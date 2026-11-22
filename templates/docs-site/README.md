# docs-site

A documentation site on [ag-core](../../../crates/ag-core) with a sidebar
generated from the same embedded page list that drives routing. No database or
external service is required.

## Files

```
Cargo.toml.tmpl        package manifest (ag-core pinned by rev)
config.toml.tmpl       loopback Shield bind configuration
src/main.rs.tmpl       page list, sidebar renderer, routes
static/styles.css.tmpl embedded stylesheet
template.toml          catalog metadata
```

Files ending in `.tmpl` are copied by `ag new`, which replaces `{{name}}` with
the project name and strips the suffix.

## Routes

| Path | Description |
| --- | --- |
| `/` | Documentation home with sidebar |
| `/docs/:slug` | Topic page with breadcrumb; 404 for unknown slug |
| `/styles.css` | Embedded stylesheet |

## Run

```sh
ag new handbook --template docs-site
cd handbook
ag dev
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/docs/shield
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

- Pages and navigation are embedded constants; changes require a rebuild.
- Bodies are plain strings without Markdown rendering or syntax highlighting.
- Page content is trusted and inserted into HTML without escaping; escape it
  if content ever comes from user input.
- No search or versioned docs.
