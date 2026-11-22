//! Retrieval-augmented generation scaffold: ingest, retrieve, answer.
//!
//! Every stage runs locally and deterministically so the scaffold works
//! offline. [`AnswerGenerator`] is the boundary where a remote model provider
//! can later be plugged in. `ag-ai` is linked as the reserved home for those
//! capabilities; at the pinned revision it has no public API.

use std::sync::{Arc, RwLock};

use ag_core::Shield;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

/// Dimension of the locally computed embeddings.
const DIMENSIONS: usize = 16;
/// Default words per chunk during ingest.
const CHUNK_WORDS: usize = 40;

/// A stored text chunk with its embedding.
#[derive(Clone, Debug, Serialize)]
struct Chunk {
    id: String,
    text: String,
    embedding: Vec<f32>,
}

/// Ingest input: one source document.
#[derive(Debug, Deserialize)]
struct DocumentInput {
    id: String,
    text: String,
}

/// Ingest request body.
#[derive(Debug, Deserialize)]
struct IngestRequest {
    documents: Vec<DocumentInput>,
}

/// Ask request body.
#[derive(Debug, Deserialize)]
struct AskRequest {
    question: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
}

fn default_top_k() -> usize {
    3
}

/// Answer response body.
#[derive(Debug, Serialize)]
struct AnswerResponse {
    generator: &'static str,
    question: String,
    answer: String,
    contexts: usize,
}

/// JSON error body returned together with a non-2xx status code.
#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    service: &'static str,
    chunks: usize,
    generator: &'static str,
}

/// Deterministic local embedding: token hashes summed into fixed buckets and
/// L2-normalised. This is a stand-in, not a trained model.
fn embed(text: &str) -> Vec<f32> {
    let mut vector = vec![0.0f32; DIMENSIONS];
    for token in text.split_whitespace() {
        let mut hash: u32 = 2_166_136_261;
        for byte in token.to_ascii_lowercase().bytes() {
            hash ^= u32::from(byte);
            hash = hash.wrapping_mul(16_776_919);
        }
        vector[(hash as usize) % DIMENSIONS] += 1.0;
    }
    let norm = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }
    vector
}

/// Cosine similarity for normalised vectors reduces to a dot product.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Split a document into fixed-size word chunks with stable ids.
fn chunk_document(id: &str, text: &str) -> Vec<Chunk> {
    let words: Vec<&str> = text.split_whitespace().collect();
    words
        .chunks(CHUNK_WORDS)
        .enumerate()
        .map(|(index, group)| {
            let chunk_text = group.join(" ");
            Chunk {
                id: format!("{id}-{index}"),
                embedding: embed(&chunk_text),
                text: chunk_text,
            }
        })
        .collect()
}

/// Local chunk store with the two operations the pipeline needs.
struct ChunkStore {
    chunks: RwLock<Vec<Chunk>>,
}

impl ChunkStore {
    fn new() -> Self {
        Self {
            chunks: RwLock::new(Vec::new()),
        }
    }

    fn ingest(&self, documents: &[DocumentInput]) -> Result<usize, String> {
        let mut created = Vec::new();
        for document in documents {
            if document.id.is_empty() {
                return Err("document id must not be empty".to_owned());
            }
            created.extend(chunk_document(&document.id, &document.text));
        }
        let count = created.len();
        let mut chunks = self.chunks.write().map_err(|error| error.to_string())?;
        chunks.extend(created);
        Ok(count)
    }

    fn retrieve(&self, question: &str, top_k: usize) -> Vec<Chunk> {
        let chunks = match self.chunks.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        let query = embed(question);
        let mut scored: Vec<(f32, Chunk)> = chunks
            .iter()
            .map(|chunk| (cosine_similarity(&query, &chunk.embedding), chunk.clone()))
            .collect();
        scored.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.into_iter().take(top_k).map(|(_, chunk)| chunk).collect()
    }

    fn len(&self) -> usize {
        self.chunks.read().map(|chunks| chunks.len()).unwrap_or(0)
    }
}

/// Answer-generation boundary. A remote model provider can implement this
/// trait without changing the ingest or retrieve stages.
trait AnswerGenerator: Send + Sync {
    fn name(&self) -> &'static str;
    fn generate(&self, question: &str, contexts: &[Chunk]) -> Result<String, String>;
}

/// Local extractive generator: returns the retrieved context verbatim with
/// stable labels. It does not call any model or external service.
struct ExtractiveGenerator;

impl AnswerGenerator for ExtractiveGenerator {
    fn name(&self) -> &'static str {
        "extractive"
    }

    fn generate(&self, question: &str, contexts: &[Chunk]) -> Result<String, String> {
        if contexts.is_empty() {
            return Err("no context chunks available; ingest documents first".to_owned());
        }
        let parts: Vec<String> = contexts
            .iter()
            .enumerate()
            .map(|(index, chunk)| format!("[{}] {}", index + 1, chunk.text))
            .collect();
        Ok(format!(
            "Question: {question}\nUsing {} local context chunk(s):\n{}",
            contexts.len(),
            parts.join("\n")
        ))
    }
}

#[derive(Clone)]
struct AppState {
    store: Arc<ChunkStore>,
    generator: Arc<dyn AnswerGenerator>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state = AppState {
        store: Arc::new(ChunkStore::new()),
        generator: Arc::new(ExtractiveGenerator),
    };

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/health", get(health_handler))
        .route("/ingest", post(ingest_handler))
        .route("/ask", post(ask_handler))
        .with_state(state);
    let app = shield.apply(router);

    // Bind to loopback by default; `ag dev` sets BIND to its --bind value.
    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("listening on http://{bind}");
    shield.serve(listener, app).await?;
    Ok(())
}

async fn health_handler(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        service: "{{name}}",
        chunks: state.store.len(),
        generator: state.generator.name(),
    })
}

async fn ingest_handler(
    State(state): State<AppState>,
    Json(request): Json<IngestRequest>,
) -> Result<Json<IngestReport>, (StatusCode, Json<ErrorBody>)> {
    let chunks = state
        .store
        .ingest(&request.documents)
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorBody { error })))?;
    Ok(Json(IngestReport {
        documents: request.documents.len(),
        chunks,
    }))
}

#[derive(Debug, Serialize)]
struct IngestReport {
    documents: usize,
    chunks: usize,
}

async fn ask_handler(
    State(state): State<AppState>,
    Json(request): Json<AskRequest>,
) -> Result<Json<AnswerResponse>, (StatusCode, Json<ErrorBody>)> {
    let contexts = state.store.retrieve(&request.question, request.top_k);
    let answer = state
        .generator
        .generate(&request.question, &contexts)
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorBody { error })))?;
    Ok(Json(AnswerResponse {
        generator: state.generator.name(),
        question: request.question,
        answer,
        contexts: contexts.len(),
    }))
}
