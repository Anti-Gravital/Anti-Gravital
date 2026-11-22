# Alpha release smoke evidence

This file records a real, reproducible end-to-end execution of
`scripts/release-smoke.sh` against the alpha release candidate. No result in
this file is simulated. The script itself exits non-zero on any failed step and
prints `PASS`/`FAIL` with an exit code for each one.

## Environment

| Item | Value |
| --- | --- |
| Date (UTC) | 2026-11-22T04:20:40Z |
| Host OS | Linux 7.0.0-44-generic x86_64 (Ubuntu) |
| rustc | rustc 1.99.1 (58cf8f9c3 2026-10-13) |
| cargo | cargo 1.99.1 (5f94df478 2026-08-27) |
| Commit | a041479feca2454e87d7d54ec5fb93e6fae00e0b |
| Branch | community5/qa |
| Repository | /home/angel-nereira/Github-local-angelnereira/worktrees5/qa |
| Network | disabled for every cargo invocation (`CARGO_NET_OFFLINE=true`, `--offline --locked`) |
| External services | none (loopback HTTP only) |
| Cargo target dir | /home/angel-nereira/Github-local-angelnereira/shared-cargo-target |

Command executed (from the repository root):

```
export CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target
export CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0
bash scripts/release-smoke.sh
```

The script creates its own `mktemp -d` workdir, removes it on exit (including
on failure), and uses a PID-unique project name so that a stale server left on
the port cannot satisfy the readiness check. Before starting `ag dev` it also
refuses to continue if the probe URL already answers.

## Exact output of the authoritative run

```
==> STEP: preconditions
PASS: preconditions (exit 0)
=== Anti-Gravital alpha release smoke test ===
date:    2026-11-22T04:20:40Z
host:    Linux 7.0.0-44-generic x86_64
rustc:   rustc 1.99.1 (58cf8f9c3 2026-10-13)
cargo:   cargo 1.99.1 (5f94df478 2026-08-27)
commit:  a041479feca2454e87d7d54ec5fb93e6fae00e0b
repo:    /home/angel-nereira/Github-local-angelnereira/worktrees5/qa
bind:    127.0.0.1:8080
url:     http://127.0.0.1:8080/
target:  /home/angel-nereira/Github-local-angelnereira/shared-cargo-target
network: disabled (CARGO_NET_OFFLINE=true, --offline --locked)
workdir: /tmp/ag-release-smoke.6KS2DG

==> STEP: build workspace (cargo build --workspace --locked --offline)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
PASS: build workspace (cargo build --workspace --locked --offline) (exit 0)
PASS: ag binary present after build (exit 0)

==> STEP: scaffold project (ag new --no-interactive)
Creating project 'agsmoke1105828' with template 'rest'...

Project 'agsmoke1105828' created.

Next steps:
  cd agsmoke1105828
  ag dev
PASS: scaffold project (exit 0)

==> STEP: compile schema (ag generate)
Generating 6 artifacts from '/tmp/ag-release-smoke.6KS2DG/agsmoke1105828/schema.ag'...
  generated/clients/typescript/types.ts
  generated/migrations/0001_initial.sql
  generated/openapi.json
  generated/openapi.yaml
  generated/src/mod.rs
  generated/src/models.rs
Generation completed in 'generated'.
PASS: compile schema and emit artifacts (exit 0)

==> STEP: resolve generated project lockfile (cargo generate-lockfile --offline)
     Locking 210 packages to highest compatible versions
      Adding axum v0.7.9 (available: v0.8.9)
      Adding generic-array v0.14.7 (available: v0.14.9)
PASS: resolve generated project lockfile (cargo generate-lockfile --offline) (exit 0)

==> STEP: cargo check generated project (--locked --offline)
    Checking agsmoke1105828 v0.1.0 (/tmp/ag-release-smoke.6KS2DG/agsmoke1105828)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
PASS: cargo check generated project (--locked --offline) (exit 0)

==> STEP: cargo build generated project (--locked --offline)
   Compiling agsmoke1105828 v0.1.0 (/tmp/ag-release-smoke.6KS2DG/agsmoke1105828)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.07s
PASS: cargo build generated project (--locked --offline) (exit 0)

==> STEP: start ag dev on loopback and curl http://127.0.0.1:8080/
PASS: loopback port free before dev start (exit 0)
      response body: agsmoke1105828 corriendo con Anti-Gravital
PASS: ag dev + curl loopback hello endpoint (exit 0)

=== RESULT: 10 steps, 10 PASS, 0 FAIL ===
RESULT: PASS
```

The process launched by `ag dev` and the generated workdir were both gone after
the run: a follow-up `curl http://127.0.0.1:8080/` failed to connect and no
`agsmoke*` process or `/tmp/ag-release-smoke.*` directory remained.

The exact log is also kept outside the repository at
`/tmp/opencode/w5-qa-smoke-run-final.log` for review during this wave.

## What this proves

- The committed workspace builds with `--locked --offline`; no network is used.
- `ag new NAME --template rest --no-interactive` produces a Cargo project with
  `Cargo.toml`, `config.toml` and `src/main.rs` and writes nothing interactively.
- `ag generate` compiles a small DSL schema and emits all six artifact kinds:
  Rust models/module, SQL migration, OpenAPI JSON and YAML, and TypeScript
  client types.
- The scaffolded project resolves an offline lockfile and passes both
  `cargo check --locked --offline` and `cargo build --locked --offline`.
- `ag dev --bind 127.0.0.1:8080` starts a local server and `GET /` returns the
  generated project's hello response on loopback.

## Acceptance criteria

Satisfied by this evidence, with no external infrastructure:

1. Alpha is executable from the repository (workspace build passes).
2. Alpha is usable offline: scaffold, DSL generation and Cargo build all work
   with the network disabled.
3. A generated prototype serves a real HTTP response on loopback.
4. The smoke procedure is reproducible: one command, deterministic steps,
   honest exit code, automatic cleanup.

Not established by this smoke test (they require external infrastructure and
belong to other roles or later gates):

- The alpha installer's prebuilt-binary download path (needs a release asset
  and a checksum channel; see wave-5 installer role).
- Any database-backed template (`fullstack` needs PostgreSQL), mail delivery
  (SMTP), DNS/ACME, edge, workers with a durable backend, or multi-node Fabric
  behaviour.
- Four-platform release CI (the smoke test runs on this Linux host only).
- Production readiness: the pre-Fase5 release gate remains open.

## Honest observations found while producing this evidence

- O1 (environment, not a product defect): an attempt to run the same script
  with a brand-new private `CARGO_TARGET_DIR` failed during the workspace
  build with `Disk quota exceeded (os error 122)` from rustc/LLVM while
  compiling large binaries. The script correctly reported `FAIL` with exit
  101 and did not claim success. The authoritative run above uses the shared
  Cargo target directory prescribed by the wave-5 cargo environment and is
  unaffected. A cold-cache run must be retried when disk headroom is available.
- O2 (test integrity): an earlier manual exploration left a `smokeapp` server
  on port 8080. A first draft of the smoke script could have accepted that
  stale server's response as a pass. The final script uses a PID-unique
  project name embedded in the expected response and refuses to start when
  the probe URL already answers; the authoritative run was taken after that
  fix and after the stale process was removed. The earlier draft runs are
  therefore not used as evidence.
- O3 (defect owned by the alpha security review, not duplicated here): the
  generated `rest` template hardcodes `0.0.0.0:8080` and ignores the `BIND`
  value that `ag dev` exports, so the dev server actually listens on all
  interfaces even though the CLI reports a loopback bind and the README
  documents a loopback default. Curl to `127.0.0.1` succeeds, which is why the
  smoke step passes, but this is a real security-relevant mismatch. Tracked by
  the wave-5 security role (finding D1); this file does not open a duplicate
  issue.

## Supplementary verification on the same branch

- `bash -n scripts/release-smoke.sh` -> exit 0.
- `cargo test --locked -p ag-cli --test wave4_regression` -> 6 passed,
  0 failed, 0 ignored. No `SKIP` lines were printed, so all wave-4 features the
  suite probes are present on this tree (base a041479, after the wave-4 merges).
- `cargo test --locked --workspace` -> exit 0 (unit, integration and doc tests).
- `cargo fmt --all -- --check` -> exit 0.
- `git diff --check` -> exit 0.
- Content scans with the patterns from `.github/workflows/docs.yml` (AI
  attribution, chat URL, emoji, retired third-party brand, descriptor name
  prefix) -> no matches. The descriptor carries `Closes: none` in its required
  closing-keyword section.
