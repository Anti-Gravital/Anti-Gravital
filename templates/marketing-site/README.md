# marketing-site

A landing page with hero, features and call-to-action sections, served by
[ag-core](../../../crates/ag-core) with embedded HTML/CSS. Adds an about page
and a plain-text health probe on top of the `static-site` layout.

## Files

```
Cargo.toml.tmpl         package manifest (ag-core pinned by rev)
config.toml.tmpl        loopback Shield bind configuration
src/main.rs.tmpl        composition root: routes + listener
static/index.html.tmpl  landing page (hero, features, CTA)
static/about.html.tmpl  about page
static/styles.css.tmpl  embedded stylesheet
template.toml           catalog metadata
```

Files ending in `.tmpl` are copied by `ag new`, which replaces `{{name}}` with
the project name and strips the suffix.

## Run

```sh
ag new campaign --template marketing-site
cd campaign
ag dev
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/about
curl http://127.0.0.1:8080/health
```

Set `BIND=0.0.0.0:8080` to listen beyond loopback.

## Verification

```sh
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
cargo check --offline
```

Recorded evidence: `/tmp/opencode/agents/w6-web.md`.

## Limitations

- Copy is static and embedded; editing it requires a rebuild.
- No form handling or CMS: the CTA does not submit anywhere.
- No analytics or tracking is included by design.
- For a content list with detail pages use `blog`; for navigable reference
  material use `docs-site`.
