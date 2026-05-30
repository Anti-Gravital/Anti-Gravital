# Pre-Phase 5 Examples Audit (Stage 9)

> Stage 9 deliverable of the master audit plan. Verifies that every shipped
> example builds, runs, and behaves exactly as its documentation claims, and that
> the DSL reference example is reproducible from the current compiler.

- **Date:** 2026-05-29
- **Branch:** `audit-pre-fase5`
- **Scope:** the five examples under `examples/` plus their index docs
  (`examples/README.md`, `docs/examples/README.md`), and the three `ag new`
  templates under `templates/`.

## Inventory

| Example | Type | Crates / DSL | External requirement | Runtime verification |
| --- | --- | --- | --- | --- |
| `todo-api` | binary (workspace member) | `ag-core`, `ag-data` | PostgreSQL | build only (needs live DB) |
| `auth-mail-demo` | binary (workspace member) | `ag-auth`, `ag-mail` | none | ran end-to-end |
| `realtime-chat` | binary (workspace member) | `ag-realtime`, `ag-observe` | none | ran end-to-end (HTTP + SSE) |
| `ai-backend` | binary (workspace member) | `ag-observe` | AI API key | build only (needs key) |
| `ecommerce-api` | DSL (`schema.ag` + `generated/`) | `ag-dsl` compiler | none | regenerated + diffed |
| `templates/{rest,realtime,fullstack}` | `ag new` scaffolds | `ag-cli` (`include_str!`) | git (build only) | scaffolded all three |

All four binaries are workspace members, so the Stage 1 build gate
(`cargo clippy --workspace --all-targets`) already proves they compile. This
stage adds runtime behaviour and documentation-honesty verification.

## Method

- Built each binary: `cargo build -p <name>` → all four compile.
- Ran the self-contained binaries and exercised their documented commands.
- For `ecommerce-api`, ran `ag schema lint` + `ag generate` into a scratch dir
  and `diff -rq` against the committed `generated/` tree.
- Cross-read every example README and `//!` header against the actual code.

## Findings

### F9-1 (Blocker, fixed) — `ag-realtime` publish fails with no subscribers; realtime-chat POST returns HTTP 500

`EventBus::publish` mapped `tokio::sync::broadcast::Sender::send`'s error to
`BusError::Closed`. That `send` returns `Err` **only** when there are zero
receivers; since `EventBus` owns its `Sender` for its whole lifetime, the bus is
never actually closed. The result: every publish into a bus with no current
subscriber failed. `realtime-chat`'s `post_message` propagated the error as
`INTERNAL_SERVER_ERROR`, so the README's documented standalone command

```bash
curl -X POST http://localhost:3000/messages -H 'Content-Type: application/json' \
  -d '{"user":"alice","text":"hola desde curl"}'
```

returned **500** whenever no browser/SSE client was connected. The doc comment
also misattributed the cause to "no active senders".

This was inconsistent with the External (NATS) path, which is already
fire-and-forget (it spawns the publish and returns `Ok`).

**Fix:** `EventBus::publish` now treats "no receivers" as a successful
fire-and-forget no-op (returns `Ok(())`), aligning the InProcess and External
paths and the documented pub/sub semantics. Doc comment corrected. Added
regression tests `publish_with_no_subscribers_is_ok` and
`late_subscriber_only_sees_events_after_subscribing`
(`crates/ag-realtime/src/bus.rs`).

**Evidence:** after the fix, the documented `curl POST` returns **201**; invalid
input (empty text) still returns **422**; SSE delivery to a connected client
still works (`data: {"user":"bob",...}` received live).
`cargo test -p ag-realtime` → 25 passed.

### F9-2 (High, fixed) — `ecommerce-api/generated/` was stale and non-reproducible

The committed `generated/` tree did not match `ag generate` output. Root cause:
the tree was a mosaic of artifacts produced by different compiler versions —
English model headers from an older English `rust_gen`, Spanish migration
headers from a pre-migration `sql_gen` — and it lacked the now-emitted
`openapi.yaml`. This violates the reproducibility rule (CLAUDE.md §36): a
developer running the documented `ag generate` got different output than what was
checked in.

**Fix:** finished the ADR-0008 English migration of the two lagging codegen
modules (`rust_gen.rs`, `openapi_gen.rs`; `sql_gen.rs`/`ts_gen.rs` were already
English), then regenerated `generated/` from the current compiler. The tree is
now uniformly English and reproducible.

**Evidence:** `ag generate` run twice into separate dirs → `diff -rq`
**identical**; no Spanish remains in the generated tree; `cargo test -p ag-dsl` →
153 passed (no snapshot test regressions).

### F9-3 (High, fixed) — codegen emitted mixed-language comments in user code

Independent of the stale tree, `rust_gen.rs` and `openapi_gen.rs` emitted Spanish
doc comments / OpenAPI descriptions into generated user code, while `sql_gen.rs`
and `ts_gen.rs` already emitted English. ADR-0008 mandates English for generated
code comments. Fixed as part of F9-2 (migration applied "on touch", as ADR-0008
permits). No DSL grammar, schema semantics, or generated code structure changed —
only comment/description text language.

### F9-4 (Medium, fixed) — two examples shipped without a README

`todo-api` and `auth-mail-demo` had no README, violating the examples rule
("Cada ejemplo trae su README con instrucciones de ejecucion"). Added
`examples/todo-api/README.md` (DB requirement, env vars, endpoints, Docker) and
`examples/auth-mail-demo/README.md` (self-contained, NullSender, switch to SMTP).

### F9-5 (Medium, fixed) — index docs declared examples non-existent

`examples/README.md` said "Fase 0: vacio. El primer ejemplo (`todo-api`) llega en
Fase 2" and `docs/examples/README.md` listed every example as "Pendiente" /
"catalogo vacio", while five examples exist and build. This is an ADR-0009
honesty violation. Both files updated to the real state;
`docs/examples/README.md` also now lists `auth-mail-demo` (it was omitted
entirely).

### F9-6 (Low, debt) — example READMEs remain in Spanish

The five `examples/` READMEs are Spanish while ADR-0008 makes English canonical.
ADR-0008 allows gradual on-touch migration and does not block. Recorded as debt
(DEBT-016); not converted in this stage to keep the change scoped to behaviour
and honesty.

### F9-7 (Medium, fixed) — templates/README.md was stale and overclaimed

`ag new` scaffolds from three templates (`rest`, `realtime`, `fullstack`),
embedded in `ag-cli` via `include_str!`. `templates/README.md` was wrong on every
count:
- declared "Fase 0: vacio. Los templates llegan ... a partir de Fase 2" while all
  three templates exist and `ag new` produces clean scaffolds (ADR-0009);
- listed a fourth template `mobile-backend` that does not exist and is not a valid
  `-t` value (clap accepts only `rest|realtime|fullstack`);
- overclaimed contents: `rest` as "auth, datos y observabilidad" (actually
  `ag-core` only), `realtime` as "WebSocket y SSE" (WebSocket only, no SSE),
  `fullstack` as "SSR ligero con `ag-ui`" (actually REST + `ag-data`/PostgreSQL,
  no `ag-ui`).

**Fix:** rewrote `templates/README.md` to list the three real templates with their
actual dependencies and the non-interactive default (`rest`).

**Verification (templates):** `ag new demo-<t> -t <t>` for all three templates
produced the expected file tree (`Cargo.toml`, `config.toml`, `src/main.rs`, plus
`migrations/0001_init.sql` for `fullstack`) with **no unsubstituted `{{...}}`
placeholders**. Template source comments are already English. Scaffolded projects
pull `ag-*` crates via git, so a full `cargo build` of a scaffold needs network
and the published repo; that is by design and out of scope for this offline gate.

## Per-example documentation-honesty result

| Example | Documented behaviour matches code? |
| --- | --- |
| `todo-api` | yes — endpoints, env vars, Docker now documented in new README |
| `auth-mail-demo` | yes — ran it: 3 emails captured via NullSender, matches README |
| `realtime-chat` | yes (after F9-1) — ports, endpoints, curl examples verified live |
| `ai-backend` | yes — port 3001, env-driven provider registry, `/providers` & `/health` work without keys, `/chat` documented to return 503 with no provider |
| `ecommerce-api` | yes (after F9-2/3) — artifact table updated to include `openapi.yaml`; tree reproducible |

## Verification commands

```sh
cargo build -p todo-api -p auth-mail-demo -p realtime-chat -p ai-backend   # all compile
cargo run  -p auth-mail-demo                                               # 3 emails, exits 0
PORT=3099 ./target/debug/ai-backend &                                      # /health 200, /providers [], /chat 503 (no key)
./target/debug/ag schema lint --schema examples/ecommerce-api/schema.ag    # sin problemas
./target/debug/ag generate --schema examples/ecommerce-api/schema.ag --output <tmp>
diff -rq examples/ecommerce-api/generated <tmp>                            # identical
./target/debug/ag new demo -t rest|realtime|fullstack                      # clean scaffold, no {{}} left
cargo test -p ag-realtime -p ag-dsl -p ag-cli                              # 25 / 153 / 6 passed
cargo fmt --all -- --check                                                 # clean
cargo clippy -p ag-realtime -p ag-dsl -p ag-cli -p realtime-chat --all-targets -- -D warnings
```

## Gate impact

The `Examples` row of `PRE_FASE5_RELEASE_GATE.md` moves to `pass`: every example
builds, the self-contained ones run as documented, the DSL example is
reproducible, the three `ag new` templates scaffold cleanly, and all
documentation claims are backed by evidence. The remaining Spanish-README
migration (F9-6, DEBT-016) is tracked as non-blocking debt.
