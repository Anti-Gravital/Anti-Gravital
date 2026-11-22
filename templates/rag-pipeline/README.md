# rag-pipeline

Retrieval-augmented generation scaffold built on `ag-core` (Shield + axum).
It implements the full ingest -> retrieve -> answer flow locally, so it builds
and runs offline with no model API key and no vector database.

## What it provides

- **Ingest**: `POST /ingest` splits documents into fixed-size word chunks and
  stores each chunk with a deterministic local embedding.
- **Retrieve**: the question is embedded locally and chunks are ranked by
  cosine similarity (`top_k`, default 3).
- **Answer**: `AnswerGenerator` is a provider-agnostic trait. The bundled
  `ExtractiveGenerator` returns the retrieved chunks verbatim with stable
  labels, clearly a stand-in rather than a language model.
- `GET /health` reports status, chunk count and generator name.

## Run

```sh
ag new my-rag --template rag-pipeline --no-interactive
cd my-rag
cargo run
```

The server listens on `127.0.0.1:8080` by default.

```sh
curl -s -X POST http://127.0.0.1:8080/ingest \
  -H 'content-type: application/json' \
  -d '{"documents":[{"id":"doc1","text":"Anti-Gravital is a modular backend toolkit written in Rust."},{"id":"doc2","text":"The DSL is the source of truth and generates Rust SQL and SDKs."}]}'
curl -s -X POST http://127.0.0.1:8080/ask \
  -H 'content-type: application/json' \
  -d '{"question":"What is the source of truth?","top_k":2}'
curl -s http://127.0.0.1:8080/health
```

`POST /ask` before any ingest returns 422 with a JSON error body.

## Configuration

`config.toml` documents the bind address, `chunk_words` and `top_k`, plus the
generator kind. `BIND` is authoritative at runtime. The values in `config.toml`
are informative starting points; the current scaffold uses the compiled-in
defaults (40 words, 3 chunks, extractive generator).

## External requirements

- None for the default build and runtime.
- A remote answer generator would read credentials from the environment, for
  example `AG_MODEL_API_KEY`. This value is declared in `template.toml`
  metadata and is never hardcoded in source or config files.

## Integration with platform crates

`ag-ai` is linked as the reserved home for AI capabilities. At the pinned
revision it is a Phase 0 skeleton with no public API, so all three stages are
implemented locally. When `ag-ai` exposes a client, implement
`AnswerGenerator` on top of it and replace `ExtractiveGenerator` in the
composition root; the ingest and retrieve stages stay unchanged.

## Limitations

- The local embedding is a hashing stand-in: it captures token overlap, not
  meaning. The extractive generator concatenates retrieved text; it does not
  reason or synthesise new content.
- The chunk store is in-memory and lost on restart; it is not shared across
  processes.
- No metadata filtering, reranking, streaming or authentication.
- The first build resolves the pinned git revision from the local cargo cache
  when building offline.

## Status

Template, experimental. Builds and runs offline with `cargo check --offline`.
