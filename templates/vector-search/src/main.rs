//! Semantic search scaffold with a repository trait and an in-memory store.
//!
//! Embeddings are computed locally with a deterministic hashing scheme so the
//! scaffold runs offline without a model service. [`VectorRepository`] is the
//! storage boundary: a durable backend can be added on top of `ag-data`
//! without changing the request handlers.

use std::sync::{Arc, RwLock};

use ag_core::Shield;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

/// Dimension of the locally computed embeddings.
const DIMENSIONS: usize = 16;

/// A stored document together with its embedding.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Document {
    id: String,
    text: String,
    embedding: Vec<f32>,
}

/// A scored search result.
#[derive(Clone, Debug, Serialize)]
struct SearchHit {
    id: String,
    text: String,
    score: f32,
}

/// Storage error reported by a repository implementation.
#[derive(Debug)]
struct RepoError(String);

/// Storage boundary for vectors. Implementations may be in-memory or durable.
trait VectorRepository: Send + Sync {
    /// Insert or replace a document by id.
    fn upsert(&self, document: Document) -> Result<(), RepoError>;

    /// Return the `top_k` highest-scoring documents for a query vector.
    fn search(&self, query: &[f32], top_k: usize) -> Vec<SearchHit>;

    /// Number of stored documents.
    fn len(&self) -> usize;
}

/// In-memory repository backed by a read/write lock.
struct InMemoryVectorRepository {
    documents: RwLock<Vec<Document>>,
}

impl InMemoryVectorRepository {
    fn new() -> Self {
        Self {
            documents: RwLock::new(Vec::new()),
        }
    }
}

impl VectorRepository for InMemoryVectorRepository {
    fn upsert(&self, document: Document) -> Result<(), RepoError> {
        if document.embedding.len() != DIMENSIONS {
            return Err(RepoError(format!(
                "embedding has {} dimensions, expected {DIMENSIONS}",
                document.embedding.len()
            )));
        }
        let mut documents = self
            .documents
            .write()
            .map_err(|error| RepoError(error.to_string()))?;
        if let Some(existing) = documents.iter_mut().find(|stored| stored.id == document.id) {
            *existing = document;
        } else {
            documents.push(document);
        }
        Ok(())
    }

    fn search(&self, query: &[f32], top_k: usize) -> Vec<SearchHit> {
        let documents = match self.documents.read() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        let mut hits: Vec<SearchHit> = documents
            .iter()
            .map(|document| SearchHit {
                id: document.id.clone(),
                text: document.text.clone(),
                score: cosine_similarity(query, &document.embedding),
            })
            .collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        hits
    }

    fn len(&self) -> usize {
        self.documents.read().map(|documents| documents.len()).unwrap_or(0)
    }
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

#[derive(Clone)]
struct AppState {
    repository: Arc<dyn VectorRepository>,
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
    documents: usize,
}

#[derive(Debug, Deserialize)]
struct DocumentRequest {
    id: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_top_k")]
    top_k: usize,
}

fn default_top_k() -> usize {
    3
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
        repository: Arc::new(InMemoryVectorRepository::new()),
    };

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/health", get(health_handler))
        .route("/documents", post(upsert_handler))
        .route("/search", get(search_handler))
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
        documents: state.repository.len(),
    })
}

async fn upsert_handler(
    State(state): State<AppState>,
    Json(request): Json<DocumentRequest>,
) -> Result<Json<Document>, (StatusCode, Json<ErrorBody>)> {
    let embedding = embed(&request.text);
    let document = Document {
        id: request.id,
        text: request.text,
        embedding,
    };
    state
        .repository
        .upsert(document.clone())
        .map_err(|error| (StatusCode::UNPROCESSABLE_ENTITY, Json(ErrorBody { error: error.0 })))?;
    Ok(Json(document))
}

async fn search_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<Vec<SearchHit>> {
    let vector = embed(&query.q);
    Json(state.repository.search(&vector, query.top_k))
}
