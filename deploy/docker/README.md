# Reusable Docker packaging for template families

This directory provides optional container packaging for projects generated
from Anti-Gravital templates. Docker is a deployment target, not a
requirement: `cargo run` and `ag dev` remain the default development path
(CLAUDE.md rule 12, integrate and never replace; ADR-0009 rule 2, keep a
native mode by default).

Nothing here is started by the CLI, required by a template, or baked into a
generated project. Copy the files you need into a generated project and adapt
the build arguments.

## Layout

```
deploy/docker/
  Dockerfile.template   multi-stage builder + minimal scratch runtime
  README.md             this file
  VERIFICATION.md       real verification evidence, including the build SKIP
  verify.sh             reproducible static checks (no daemon required)
  compose/
    .env.example        placeholder variables; copy to .env, never commit it
    web.yml             web / UI family
    api.yml             HTTP API family with PostgreSQL
    realtime.yml        realtime family with a cache dependency
    infrastructure.yml  background worker family
    ai.yml              AI backend family
```

## Dockerfile.template

Three stages:

1. `builder` on `rust:1.99.1` adds the musl target, copies the context and
   builds a fully static release binary with `--locked`. Cargo registry and
   target directories use BuildKit cache mounts.
2. `probe` on `busybox:1.36-musl` supplies the static `sh` and `wget` needed
   by `HEALTHCHECK`, because the final `scratch` image has no shell.
3. `scratch` runtime holds only the service binary, the CA certificate
   bundle, the tiny probe files and `/tmp`. It runs as numeric UID/GID
   `65532:65532`, exposes a port, declares a `HEALTHCHECK`, and has no entry
   that reads a secret.

Build arguments:

| Argument | Default | Purpose |
| --- | --- | --- |
| `RUST_IMAGE` | `rust:1.99.1` | builder toolchain image |
| `RUST_TARGET` | `x86_64-unknown-linux-musl` | static build target |
| `PACKAGE` | `todo-api` | Cargo package passed to `-p` |
| `BINARY` | `todo-api` | executable name in `target/<target>/release/` |

Runtime environment: `PORT` (default `8080`) and `HEALTH_PATH` (default
`/health`) drive the health probe. `EXPOSE 8080` is documentation only; set
`PORT` / `BIND` and the Compose `ports` mapping for the real port.

### Adapting it to a generated project

1. Copy `Dockerfile.template` to the project root as `Dockerfile`, or keep it
   under `deploy/docker/` and point `build.dockerfile` at it.
2. Set `PACKAGE` and `BINARY` to the package and binary of your project.
3. Set `PORT` and `BIND` consistently (for example `PORT=8080` and
   `BIND=0.0.0.0:8080`); containers must bind to all interfaces. Update
   `EXPOSE` to match.
4. If the health endpoint differs, set `HEALTH_PATH` to that path. Workers
   without HTTP should disable the healthcheck, as in
   `compose/infrastructure.yml`.
5. Add a `.dockerignore` at the build-context root with at least:
   `.git`, `target`, `.env`, `*.log`. The template uses `COPY . .` so it also
   works in a single-crate generated project.
6. For another architecture, change `RUST_TARGET` (for example
   `aarch64-unknown-linux-musl`) and install the matching musl toolchain in
   the builder stage.

### Optional build and run

```sh
docker build \
  --build-arg PACKAGE=todo-api \
  --build-arg BINARY=todo-api \
  -f deploy/docker/Dockerfile.template \
  -t ag-api:local .

docker run --rm -p 8080:8080 \
  -e BIND=0.0.0.0:8080 \
  -e DATABASE_URL='postgresql://user:password@host:5432/db' \
  ag-api:local
```

No credential is placed in an image layer. Pass secrets with `-e`, an
`--env-file`, or a mounted secret file.

## Compose files

Each file is a family pattern with environment-variable interpolation and
named volumes for persistent data. Build context `../..` is the repository or
generated project root. The defaults point at existing workspace examples so
the files are concrete; a generated project overrides the `*_PACKAGE` and
`*_BINARY` variables.

| File | Service plus dependency | Default package | Port | Named volumes |
| --- | --- | --- | --- | --- |
| `web.yml` | web service | `realtime-chat` | 3000 | `web-assets` |
| `api.yml` | api + PostgreSQL | `todo-api` | 8080 | `pg-data`, `api-uploads` |
| `realtime.yml` | realtime + cache | `realtime-chat` | 3000 | `redis-data`, `realtime-state` |
| `infrastructure.yml` | background worker | `workers-basic` | none | `infra-spool` |
| `ai.yml` | AI backend | `ai-backend` | 3001 | `ai-cache` |

Start a family with a local env file:

```sh
cp deploy/docker/compose/.env.example deploy/docker/compose/.env
# edit .env: set POSTGRES_PASSWORD and AI_API_KEY to real values
docker compose -f deploy/docker/compose/api.yml --env-file deploy/docker/compose/.env up --build
```

PostgreSQL and Redis are integrated dependencies, not replacements for the
native in-memory paths. `infrastructure.yml` runs the in-memory worker
backend and needs no external service.

## Verification

Run `bash deploy/docker/verify.sh`. It validates all five compose files with
`docker compose config -q`, parses them with PyYAML, checks the compose
contract, lints the Dockerfile statically, and attempts a real build only if
a daemon is reachable. Unavailable checks are reported as `SKIP`.

In this worktree the CLI (29.10.0) and Compose plugin (v5.5.1) are installed,
but the daemon is not reachable, so the build is an honest **SKIP**. The
static checks returned `PASS=23 FAIL=0 SKIP=2`. Exact commands and output are
in `VERIFICATION.md`.

## Limitations

- The scratch runtime execs a static Rust binary; it is intended for the
  HTTP and worker examples in this repository, not for arbitrary generated
  code that needs a system shell or runtime data files at startup.
- `HEALTHCHECK` assumes an HTTP endpoint that returns success on `/health`;
  non-HTTP services must disable or replace it.
- BuildKit cache mounts require BuildKit; without it the build still works
  but is slower.
- No image is published and no registry, signature or SBOM is produced. This
  is packaging guidance for teams that deploy with Docker or Compose.
