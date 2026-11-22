# blog

A small blog on [ag-core](../../../crates/ag-core) with a post list, per-post
detail pages and a JSON API. All posts come from one embedded `const` slice, so
there is no database and no external service.

## Files

```
Cargo.toml.tmpl        package manifest (ag-core pinned by rev)
config.toml.tmpl       loopback Shield bind configuration
src/main.rs.tmpl       post data + routes + HTML builders
static/styles.css.tmpl embedded stylesheet
template.toml          catalog metadata
```

Files ending in `.tmpl` are copied by `ag new`, which replaces `{{name}}` with
the project name and strips the suffix.

## Routes

| Path | Description |
| --- | --- |
| `/` | HTML list of posts with links to detail pages |
| `/posts/:id` | HTML detail page, or 404 for an unknown id |
| `/api/posts` | JSON array of all posts |
| `/styles.css` | Embedded stylesheet |

## Run

```sh
ag new myblog --template blog
cd myblog
ag dev
curl http://127.0.0.1:8080/
curl http://127.0.0.1:8080/posts/hello-world
curl http://127.0.0.1:8080/api/posts
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

- Posts are static and compiled in; adding one requires a rebuild.
- Post fields are trusted constants and are inserted into HTML without
  escaping. If content becomes user-provided, add HTML escaping before
  rendering.
- No pagination, search, drafts or Markdown rendering.
- The JSON API ignores query parameters and returns all posts.
