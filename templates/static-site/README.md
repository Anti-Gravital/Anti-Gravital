# static-site

A minimal static web site served by [ag-core](../../../crates/ag-core) with
[Shield](../../../docs/manual) protection. HTML and CSS are embedded into the
binary with `include_str!`; there is no database and no external service.

## Files

```
Cargo.toml.tmpl        package manifest (ag-core pinned by rev)
config.toml.tmpl       loopback Shield bind configuration
src/main.rs.tmpl       composition root: routes + listener
static/index.html.tmpl embedded home page
static/styles.css.tmpl embedded stylesheet
template.toml          catalog metadata
```

Files ending in `.tmpl` are copied by `ag new`, which replaces `{{name}}` with
the project name and strips the suffix.

## Run

```sh
ag new mysite --template static-site
cd mysite
ag dev
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/styles.css
```

Without the CLI, substitute the placeholder and run Cargo directly:

```sh
sed 's/{{name}}/mysite/g' Cargo.toml.tmpl > Cargo.toml
# ... repeat for each .tmpl file, stripping the suffix ...
cargo run
```

A custom bind address can be supplied with `BIND=0.0.0.0:8080 cargo run`.

## Verification

```sh
# From a scaffold in /tmp with the shared Cargo target dir:
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
cargo check --offline
```

See `/tmp/opencode/agents/w6-web.md` for the exact scaffold commands and the
recorded result for this template.

## Limitations

- Assets are compile-time embedded. Editing HTML/CSS requires a rebuild
  (`cargo-watch` via `ag dev` handles this in development).
- No templating engine: `{{name}}` is substituted only at scaffold time.
- Equal to `rest` in runtime behaviour; the difference is the static asset
  layout and the HTML/CSS responses. For landing-page copy and sections use
  `marketing-site`.
