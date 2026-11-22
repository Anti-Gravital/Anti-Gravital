# vector-search

Semantic search service built on `ag-core` (Shield + axum). It defines a
`VectorRepository` trait and ships an in-memory implementation, so it builds
and runs offline with no database.

## What it provides

- `VectorRepository`: upsert, search and count, the storage boundary.
- `InMemoryVectorRepository`: a thread-safe in-memory store.
- A deterministic local embedding function (token hashing, L2 normalised) and
  cosine-similarity ranking.
- `POST /documents` to add a document, `GET /search?q=...&top_k=...` to query,
  and `GET /health` for readiness.

## Run

```sh
ag new my-search --template vector-search --no-interactive
cd my-search
cargo run
```

The server listens on `127.0.0.1:8080` by default.

```sh
curl -s -X POST http://127.0.0.1:8080/documents \
  -H 'content-type: application/json' \
  -d '{"id":"a","text":"Rust gives memory safety without a garbage collector"}'
curl -s -X POST http://127.0.0.1:8080/documents \
  -H 'content-type: application/json' \
  -d '{"id":"b","text":"Semantic search ranks documents by vector similarity"}'
curl -s 'http://127.0.0.1:8080/search?q=memory%20safety&top_k=2'
curl -s http://127.0.0.1:8080/health
```

`top_k` defaults to 3. An upsert with a malformed embedding dimension returns
422; the HTTP API always computes the embedding itself.

## Configuration

`config.toml` documents the bind address and the store kind. `BIND` is
authoritative at runtime. The in-memory store is selected by default.

## External requirements

- None for the default build and runtime.
- A durable vector store would use `ag-data` (the sanctioned data boundary)
  and read its connection string from the environment, for example
  `VECTOR_DB_URL`. This is declared in `template.toml` metadata and is never
  hardcoded in source or config files.

## Integration with platform crates

`ag-data` is linked as the reserved durable-data boundary. The default store
is local and in-memory, so no database is contacted. To add persistence,
implement `VectorRepository` over a pool obtained from `ag-data::connect` and
select it in the composition root; handlers stay unchanged.

## Limitations

- The local embedding is a hashing stand-in, not a trained model: it captures
  token overlap, not true semantics. Replace it with a real embedding provider
  for meaningful search.
- The in-memory store is lost on restart and is not shared across processes.
- No metadata filtering, hybrid search, pagination or authentication.
- The first build resolves the pinned git revision from the local cargo cache
  when building offline.

## Status

Template, experimental. Builds and runs offline with `cargo check --offline`.
