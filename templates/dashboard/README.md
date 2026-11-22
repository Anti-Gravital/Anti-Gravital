# dashboard

An observability dashboard on [ag-core](../../../crates/ag-core) with an HTML
page and a JSON metrics endpoint. Both read the same embedded sample data, so
no metrics backend, database or external service is required.

## Files

```
Cargo.toml.tmpl        package manifest (ag-core pinned by rev)
config.toml.tmpl       loopback Shield bind configuration
src/main.rs.tmpl       sample metrics, HTML page, JSON endpoint
static/styles.css.tmpl embedded stylesheet
template.toml          catalog metadata
```

Files ending in `.tmpl` are copied by `ag new`, which replaces `{{name}}` with
the project name and strips the suffix.

## Routes

| Path | Description |
| --- | --- |
| `/` | HTML dashboard with a metrics table |
| `/api/metrics` | JSON object with service name and metrics array |
| `/styles.css` | Embedded stylesheet |
| `/health` | Plain-text health probe |

## Run

```sh
ag new ops-dash --template dashboard
cd ops-dash
ag dev
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/api/metrics
```

## Verification

```sh
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
cargo check --offline
```

Recorded evidence: `/tmp/opencode/agents/w6-web.md`.

## Limitations

- Metrics are static sample values, not live measurements. Wire the JSON
  endpoint to a real collector when deploying.
- The page does not auto-refresh and has no charts.
- Sample values are example data only; they are not a performance claim.
