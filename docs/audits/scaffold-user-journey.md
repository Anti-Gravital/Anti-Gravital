# Wave 10 real-terminal journey: scaffold to HTTP prototype

Audit of the complete normal-user path for `0.1.0-alpha.2`, executed as a
real terminal user in a dedicated worktree. Every command below was run
against a freshly built `ag` binary; outputs and exit codes are verbatim
captures, not reconstructions.

## Environment and method

| Item | Value |
| --- | --- |
| Date (UTC) | 2026-11-22 13:17 - 13:40 |
| Host | Linux 7.0.0-44-generic, x86_64, 12th Gen Intel i7-1255U, 12 CPUs |
| Rust / Cargo | 1.99.1 (58cf8f9c3 2026-10-13) / 1.99.1 |
| Repo commit | `72fa9d8`, branch `community10/scaffold` |
| `ag` binary | `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag` |
| Private build target | `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target` |
| Generated-project targets | `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-gen-target/<project>` |
| Fresh work dir | `/tmp/opencode/scaffold-journey/work` |
| Timing method | `date +%s.%N` before and after each command; elapsed via awk, 3 decimals |

The `ag` binary was built with
`CARGO_TARGET_DIR=<private> CARGO_INCREMENTAL=0 cargo build -p ag-cli --bin ag --offline`.
A private target dir was used so no other wave-10 agent's binary could be
overwritten. `TMPDIR` was pointed at the private target because the default
tmpfs `/tmp` rejected a compiler temporary with `Disk quota exceeded` on the
first build attempt (7.86 s, exit 101, `ring` build script). That is a host
constraint, not a project defect; the retry with `TMPDIR` set succeeded in
297.96 s. Failed excerpt:

```text
error writing to /tmp/ccr4aJRH.s: Disk quota exceeded
error: failed to run custom build command for `ring v0.17.14`
```

Generated projects were checked with `CARGO_TARGET_DIR` set to a separate
per-project directory in the home filesystem. This changes only where
artifacts are written; resolution, dependency sources and the `--offline`
semantics are exactly those of the documented `cd demo && cargo check --offline`.

## Summary

| # | Command (abbreviated) | Exit | Elapsed (s) | Result |
| --- | --- | --- | --- | --- |
| 1 | `ag --version` | 0 | 0.066 | ok |
| 2 | `ag new demo --template rest --no-interactive` | 0 | 0.059 | ok |
| 3 | `cd demo && cargo check --offline` | 0 | 145.749 | ok, cold check |
| 4 | `ag schema lint` | 0 | 0.120 | ok, no issues |
| 5 | `ag generate` | 0 | 0.098 | ok, 10 artifacts |
| 6 | `ag build` | 0 | 228.197 | ok, release binary |
| 7 | `cargo build --offline` (dev warm-up) | 0 | 15.265 | ok |
| 8 | `ag dev` + `curl` (warm debug build) | 0 | ~2 | ok, HTTP 200 |
| 9 | `ag new website --template static-site --no-interactive` | 0 | 0.018 | ok |
| 10 | `ag new hooks --template webhook-gateway --no-interactive` | 0 | 0.019 | ok |
| 11 | `ag new layeredmod --template layered --with auth --with cache --no-interactive` | 0 | 0.019 | ok |
| 12 | `ag new badwith --template rest --with auth --no-interactive` | 1 | 0.020 | expected error |
| 13 | `cd website && cargo check --offline` | 0 | 67.379 | ok |
| 14 | `cd hooks && cargo check --offline` | 0 | 52.609 | ok |
| 15 | `cd layeredmod && cargo check --offline` | 0 | 96.264 | ok |

Observed wall-clock for the whole journey was roughly 10 minutes, of
which over 9 minutes were dependency compilation. All CLI operations
(`new`, `schema lint`, `generate`, version) completed in under 0.12 s.

## Step 1: Print the CLI version

- Command: `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag --version`
- Working directory: `/tmp/opencode/scaffold-journey/work`
- Exit code: `0`
- Elapsed: `0.066 s`
- Note: Instant. No filesystem or network access.

Standard output:

```text
ag 0.1.0-alpha.2
```

Standard error:

_(empty)_

## Step 2: Scaffold the default REST project

- Command: `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag new demo --template rest --no-interactive`
- Working directory: `/tmp/opencode/scaffold-journey/work`
- Exit code: `0`
- Elapsed: `0.059 s`
- Note: Embedded templates; no network needed.

Standard output:

```text
Creating project 'demo' with template 'rest'...

Project 'demo' created.

Next steps:
  cd demo
  ag dev
```

Standard error:

_(empty)_

## Step 3: Offline cargo check of the generated REST project

- Command: `env CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-gen-target/demo cargo check --offline`
- Working directory: `/tmp/opencode/scaffold-journey/work/demo`
- Exit code: `0`
- Elapsed: `145.749 s`
- Note: A fresh Cargo.lock with 210 packages was created. The pinned git dependency `ag-core` at rev `91a95ff6` was resolved from the pre-warmed `~/.cargo/git` cache. Cold semantic check of ~210 crates dominates the time.

Standard output:

_(empty)_

Standard error:

```text
     Locking 210 packages to highest compatible versions
      Adding axum v0.7.9 (available: v0.8.9)
      Adding generic-array v0.14.7 (available: v0.14.9)
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.27
   Compiling quote v1.0.47
   Compiling libc v0.2.191
    Checking cfg-if v1.0.5
   Compiling version_check v0.9.5
    Checking subtle v2.6.1
    Checking typenum v1.20.1
    Checking const-oid v0.9.6
    Checking pin-project-lite v0.2.18
    Checking itoa v1.0.19
    Checking base64ct v1.8.3
    Checking bytes v1.12.2
    Checking once_cell v1.21.4
   Compiling serde_core v1.0.229
    Checking futures-core v0.3.34
   Compiling zerocopy v0.8.64
   Compiling serde v1.0.229
    Checking smallvec v1.16.6
    Checking futures-sink v0.3.34
    Checking slab v0.4.12
   Compiling libm v0.2.16
    Checking pem-rfc7468 v0.7.0
   Compiling autocfg v1.5.1
    Checking cpufeatures v0.2.17
   Compiling generic-array v0.14.7
    Checking tracing-core v0.1.36
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.15
    Checking equivalent v1.0.3
   Compiling httparse v1.10.1
    Checking base16ct v0.2.0
    Checking http v1.5.0
   Compiling cc v1.11.3
    Checking futures-task v0.3.34
    Checking hashbrown v0.17.2
   Compiling zmij v1.0.23
   Compiling num-traits v0.2.19
    Checking futures-util v0.3.34
   Compiling semver v1.0.28
   Compiling syn v2.0.119
   Compiling syn v3.0.10
    Checking spin v0.9.9
    Checking indexmap v2.14.2
    Checking lazy_static v1.5.1
   Compiling rustc_version v0.4.1
    Checking http-body v1.1.0
   Compiling serde_json v1.0.154
    Checking scopeguard v1.2.0
   Compiling parking_lot_core v0.9.12
    Checking tower-service v0.3.3
    Checking getrandom v0.2.17
    Checking errno v0.3.15
    Checking socket2 v0.6.5
    Checking signal-hook-registry v1.4.8
    Checking mio v1.2.5
   Compiling crossbeam-utils v0.8.23
    Checking rand_core v0.6.4
    Checking ff v0.13.1
    Checking log v0.4.35
    Checking group v0.13.0
   Compiling thiserror v1.0.69
    Checking lock_api v0.4.14
   Compiling curve25519-dalek v4.1.3
    Checking bitflags v2.13.2
    Checking sync_wrapper v1.0.2
    Checking memchr v2.8.3
    Checking atomic-waker v1.1.2
   Compiling time-core v0.1.9
    Checking fnv v1.0.7
   Compiling rustversion v1.0.23
    Checking untrusted v0.9.0
   Compiling thiserror v2.0.21
   Compiling num-conv v0.2.2
    Checking num-integer v0.1.47
   Compiling getrandom v0.4.3
    Checking tower-layer v0.3.3
   Compiling num-bigint-dig v0.8.6
   Compiling time-macros v0.2.32
    Checking num-iter v0.1.46
    Checking futures-channel v0.3.34
    Checking powerfmt v0.2.1
    Checking base64 v0.22.1
    Checking httpdate v1.0.3
   Compiling portable-atomic v1.15.0
   Compiling rustls v0.23.46
    Checking percent-encoding v2.3.2
    Checking deranged v0.5.9
    Checking regex-syntax v0.8.12
   Compiling ring v0.17.14
    Checking utf-8 v0.7.6
    Checking data-encoding v2.11.1
    Checking byteorder v1.5.0
    Checking form_urlencoded v1.2.3
    Checking num-bigint v0.4.8
    Checking raw-cpuid v11.6.0
    Checking http-body-util v0.1.5
    Checking ryu v1.0.23
    Checking hashbrown v0.14.5
    Checking winnow v0.7.15
    Checking toml_write v0.1.2
    Checking mime v0.3.18
    Checking uuid v1.29.3
    Checking regex-automata v0.4.19
    Checking pem v3.0.6
    Checking parking_lot v0.12.5
    Checking dashmap v6.2.1
    Checking serde_path_to_error v0.1.20
    Checking spinning_top v0.3.1
    Checking tracing-log v0.2.0
    Checking sharded-slab v0.1.7
    Checking time v0.3.55
    Checking thread_local v1.1.10
    Checking futures-timer v3.0.4
    Checking nu-ansi-term v0.50.3
    Checking nonzero_ext v0.3.0
    Checking no-std-compat v0.4.1
    Checking matchit v0.7.3
    Checking quanta v0.12.6
   Compiling tokio-macros v2.7.2
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling async-trait v0.1.92
    Checking ppv-lite86 v0.2.21
    Checking rand_chacha v0.3.1
    Checking axum-core v0.4.5
    Checking matchers v0.2.0
    Checking tokio v1.54.1
    Checking rand v0.8.8
   Compiling zeroize_derive v1.5.0
   Compiling tracing-attributes v0.1.31
   Compiling thiserror-impl v1.0.69
   Compiling curve25519-dalek-derive v0.1.1
    Checking zeroize v1.9.2
    Checking governor v0.7.0
    Checking der v0.7.10
    Checking rustls-pki-types v1.16.0
    Checking tracing v0.1.44
    Checking rustls-webpki v0.103.17
    Checking block-buffer v0.10.4
    Checking crypto-common v0.1.7
    Checking crypto-bigint v0.5.5
    Checking digest v0.10.7
    Checking hmac v0.12.1
    Checking signature v2.2.0
    Checking sha2 v0.10.9
    Checking sha1 v0.10.7
    Checking hkdf v0.12.4
    Checking rfc6979 v0.4.0
    Checking spki v0.7.3
    Checking tungstenite v0.24.0
    Checking pkcs8 v0.10.2
    Checking sec1 v0.7.3
    Checking pkcs1 v0.7.5
    Checking ed25519 v2.2.3
    Checking simple_asn1 v0.6.4
    Checking rsa v0.9.10
    Checking ed25519-dalek v2.2.0
    Checking elliptic-curve v0.13.8
    Checking toml_datetime v0.6.11
    Checking serde_spanned v0.6.9
    Checking serde_urlencoded v0.7.1
    Checking tracing-serde v0.2.0
    Checking primeorder v0.13.6
    Checking ecdsa v0.16.9
    Checking toml_edit v0.22.27
    Checking tracing-subscriber v0.3.23
    Checking p384 v0.13.1
    Checking p256 v0.13.2
    Checking jsonwebtoken v10.4.0
    Checking tokio-util v0.7.20
    Checking tower v0.5.3
    Checking tokio-tungstenite v0.24.0
    Checking tokio-rustls v0.26.6
    Checking tower-http v0.6.11
    Checking toml v0.8.23
    Checking h2 v0.4.20
    Checking hyper v1.12.0
    Checking hyper-util v0.1.21
    Checking axum v0.7.9
    Checking ag-core v0.0.0 (https://github.com/anti-gravital/anti-gravital?rev=91a95ff61ed6ef4c9d3f1c9069357e97389d1382#91a95ff6)
    Checking demo v0.1.0 (/tmp/opencode/scaffold-journey/work/demo)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 25s
```

## Step 4: Lint a small valid schema.ag

- Command: `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag schema lint`
- Working directory: `/tmp/opencode/scaffold-journey/work/demo`
- Exit code: `0`
- Elapsed: `0.120 s`
- Note: Fixture copied from `tests/generated-rust-fixture/schema.ag`. Instant.

Standard output:

```text
'schema.ag': no issues found.
```

Standard error:

_(empty)_

## Step 5: Generate artifacts from schema.ag

- Command: `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag generate`
- Working directory: `/tmp/opencode/scaffold-journey/work/demo`
- Exit code: `0`
- Elapsed: `0.098 s`
- Note: Ten artifacts written under `generated/`.

Standard output:

```text
Generating 10 artifacts from 'schema.ag'...
  generated/clients/typescript/client.ts
  generated/clients/typescript/types.ts
  generated/migrations/0001_initial.sql
  generated/openapi.json
  generated/openapi.yaml
  generated/src/handlers.rs
  generated/src/mod.rs
  generated/src/models.rs
  generated/src/router.rs
  generated/src/types.rs
Generation completed in 'generated'.
```

Standard error:

_(empty)_

## Step 6: Release build through `ag build`

- Command: `env CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-gen-target/demo timeout 600 /home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag build`
- Working directory: `/tmp/opencode/scaffold-journey/work/demo`
- Exit code: `0`
- Elapsed: `228.197 s`
- Note: Runs `cargo build --release` with no `--offline` flag; all crates were already cached, so no registry update message appeared. ~3m48s of pure compilation.

Standard output:

```text
Building in release mode...
Build completed successfully.
```

Standard error:

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.27
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling version_check v0.9.5
   Compiling subtle v2.6.1
   Compiling typenum v1.20.1
   Compiling const-oid v0.9.6
   Compiling itoa v1.0.19
   Compiling pin-project-lite v0.2.18
   Compiling once_cell v1.21.4
   Compiling bytes v1.12.2
   Compiling base64ct v1.8.3
   Compiling serde_core v1.0.229
   Compiling futures-core v0.3.34
   Compiling zerocopy v0.8.64
   Compiling serde v1.0.229
   Compiling futures-sink v0.3.34
   Compiling generic-array v0.14.7
   Compiling smallvec v1.16.6
   Compiling libm v0.2.16
   Compiling slab v0.4.12
   Compiling autocfg v1.5.1
   Compiling cpufeatures v0.2.17
   Compiling tracing-core v0.1.36
   Compiling pem-rfc7468 v0.7.0
   Compiling find-msvc-tools v0.1.15
   Compiling shlex v2.0.1
   Compiling cc v1.11.3
   Compiling http v1.5.0
   Compiling num-traits v0.2.19
   Compiling futures-task v0.3.34
   Compiling httparse v1.10.1
   Compiling syn v2.0.119
   Compiling syn v3.0.10
   Compiling equivalent v1.0.3
   Compiling getrandom v0.2.17
   Compiling errno v0.3.15
   Compiling mio v1.2.5
   Compiling socket2 v0.6.5
   Compiling signal-hook-registry v1.4.8
   Compiling rand_core v0.6.4
   Compiling ff v0.13.1
   Compiling base16ct v0.2.0
   Compiling hashbrown v0.17.2
   Compiling group v0.13.0
   Compiling ring v0.17.14
   Compiling http-body v1.1.0
   Compiling indexmap v2.14.2
   Compiling futures-util v0.3.34
   Compiling zmij v1.0.23
   Compiling semver v1.0.28
   Compiling num-integer v0.1.47
   Compiling spin v0.9.9
   Compiling lazy_static v1.5.1
   Compiling rustc_version v0.4.1
   Compiling log v0.4.35
   Compiling thiserror v1.0.69
   Compiling serde_json v1.0.154
   Compiling scopeguard v1.2.0
   Compiling parking_lot_core v0.9.12
   Compiling tower-service v0.3.3
   Compiling crossbeam-utils v0.8.23
   Compiling lock_api v0.4.14
   Compiling curve25519-dalek v4.1.3
   Compiling tower-layer v0.3.3
   Compiling num-bigint-dig v0.8.6
   Compiling rustversion v1.0.23
   Compiling num-conv v0.2.2
   Compiling getrandom v0.4.3
   Compiling bitflags v2.13.2
   Compiling fnv v1.0.7
   Compiling thiserror v2.0.21
   Compiling sync_wrapper v1.0.2
   Compiling memchr v2.8.3
   Compiling atomic-waker v1.1.2
   Compiling time-core v0.1.9
   Compiling untrusted v0.9.0
   Compiling num-iter v0.1.46
   Compiling futures-channel v0.3.34
   Compiling time-macros v0.2.32
   Compiling byteorder v1.5.0
   Compiling powerfmt v0.2.1
   Compiling portable-atomic v1.15.0
   Compiling httpdate v1.0.3
   Compiling base64 v0.22.1
   Compiling rustls v0.23.46
   Compiling utf-8 v0.7.6
   Compiling data-encoding v2.11.1
   Compiling percent-encoding v2.3.2
   Compiling deranged v0.5.9
   Compiling regex-syntax v0.8.12
   Compiling form_urlencoded v1.2.3
   Compiling tokio-macros v2.7.2
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling async-trait v0.1.92
   Compiling ppv-lite86 v0.2.21
   Compiling time v0.3.55
   Compiling raw-cpuid v11.6.0
   Compiling tokio v1.54.1
   Compiling rand_chacha v0.3.1
   Compiling num-bigint v0.4.8
   Compiling rand v0.8.8
   Compiling regex-automata v0.4.19
   Compiling zeroize_derive v1.5.0
   Compiling tracing-attributes v0.1.31
   Compiling zeroize v1.9.2
   Compiling der v0.7.10
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling crypto-bigint v0.5.5
   Compiling thiserror-impl v1.0.69
   Compiling digest v0.10.7
   Compiling tracing v0.1.44
   Compiling hmac v0.12.1
   Compiling signature v2.2.0
   Compiling hkdf v0.12.4
   Compiling rfc6979 v0.4.0
   Compiling sha2 v0.10.9
   Compiling sha1 v0.10.7
   Compiling spki v0.7.3
   Compiling rustls-pki-types v1.16.0
   Compiling pkcs8 v0.10.2
   Compiling tokio-util v0.7.20
   Compiling sec1 v0.7.3
   Compiling elliptic-curve v0.13.8
   Compiling h2 v0.4.20
   Compiling curve25519-dalek-derive v0.1.1
   Compiling primeorder v0.13.6
   Compiling ecdsa v0.16.9
   Compiling tungstenite v0.24.0
   Compiling pkcs1 v0.7.5
   Compiling ed25519 v2.2.3
   Compiling tower v0.5.3
   Compiling toml_datetime v0.6.11
   Compiling serde_spanned v0.6.9
   Compiling http-body-util v0.1.5
   Compiling winnow v0.7.15
   Compiling toml_write v0.1.2
   Compiling ryu v1.0.23
   Compiling hashbrown v0.14.5
   Compiling mime v0.3.18
   Compiling ed25519-dalek v2.2.0
   Compiling axum-core v0.4.5
   Compiling serde_urlencoded v0.7.1
   Compiling dashmap v6.2.1
   Compiling tokio-tungstenite v0.24.0
   Compiling p384 v0.13.1
   Compiling rustls-webpki v0.103.17
   Compiling rsa v0.9.10
   Compiling hyper v1.12.0
   Compiling toml_edit v0.22.27
   Compiling hyper-util v0.1.21
   Compiling p256 v0.13.2
   Compiling matchers v0.2.0
   Compiling tracing-serde v0.2.0
   Compiling simple_asn1 v0.6.4
   Compiling quanta v0.12.6
   Compiling uuid v1.29.3
   Compiling pem v3.0.6
   Compiling parking_lot v0.12.5
   Compiling spinning_top v0.3.1
   Compiling serde_path_to_error v0.1.20
   Compiling tracing-log v0.2.0
   Compiling sharded-slab v0.1.7
   Compiling thread_local v1.1.10
   Compiling futures-timer v3.0.4
   Compiling nonzero_ext v0.3.0
   Compiling no-std-compat v0.4.1
   Compiling nu-ansi-term v0.50.3
   Compiling matchit v0.7.3
   Compiling toml v0.8.23
   Compiling governor v0.7.0
   Compiling tracing-subscriber v0.3.23
   Compiling axum v0.7.9
   Compiling jsonwebtoken v10.4.0
   Compiling tower-http v0.6.11
   Compiling tokio-rustls v0.26.6
   Compiling ag-core v0.0.0 (https://github.com/anti-gravital/anti-gravital?rev=91a95ff61ed6ef4c9d3f1c9069357e97389d1382#91a95ff6)
   Compiling demo v0.1.0 (/tmp/opencode/scaffold-journey/work/demo)
    Finished `release` profile [optimized] target(s) in 3m 48s
```

## Step 7: Warm the debug build before `ag dev`

- Command: `env CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-gen-target/demo timeout 600 cargo build --offline`
- Working directory: `/tmp/opencode/scaffold-journey/work/demo`
- Exit code: `0`
- Elapsed: `15.265 s`
- Note: Added as a measurement aid after the first cold `ag dev` attempt was still compiling at the 65 s cutoff. Not a user-visible command in the quickstart.

Standard output:

_(empty)_

Standard error:

```text
   Compiling hyper v1.12.0
   Compiling hyper-util v0.1.21
   Compiling axum v0.7.9
   Compiling ag-core v0.0.0 (https://github.com/anti-gravital/anti-gravital?rev=91a95ff61ed6ef4c9d3f1c9069357e97389d1382#91a95ff6)
   Compiling demo v0.1.0 (/tmp/opencode/scaffold-journey/work/demo)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.19s
```

## Step 8: Scaffold a web template (static-site)

- Command: `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag new website --template static-site --no-interactive`
- Working directory: `/tmp/opencode/scaffold-journey/work`
- Exit code: `0`
- Elapsed: `0.018 s`
- Note: Instant, embedded template.

Standard output:

```text
Creating project 'website' with template 'static-site'...

Project 'website' created.

Next steps:
  cd website
  ag dev
```

Standard error:

_(empty)_

## Step 9: Scaffold an API template (webhook-gateway)

- Command: `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag new hooks --template webhook-gateway --no-interactive`
- Working directory: `/tmp/opencode/scaffold-journey/work`
- Exit code: `0`
- Elapsed: `0.019 s`
- Note: Instant, embedded template.

Standard output:

```text
Creating project 'hooks' with template 'webhook-gateway'...

Project 'hooks' created.

Next steps:
  cd hooks
  ag dev
```

Standard error:

_(empty)_

## Step 10: Scaffold layered workspace with `--with auth --with cache`

- Command: `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag new layeredmod --template layered --with auth --with cache --no-interactive`
- Working directory: `/tmp/opencode/scaffold-journey/work`
- Exit code: `0`
- Elapsed: `0.019 s`
- Note: Manifest rewritten to `auth = true` and `cache = true`.

Standard output:

```text
Creating project 'layeredmod' with template 'layered'...

Project 'layeredmod' created.

Next steps:
  cd layeredmod
  ag dev
```

Standard error:

_(empty)_

## Step 11: Negative case: `--with` on a template without ag.toml

- Command: `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag new badwith --template rest --with auth --no-interactive`
- Working directory: `/tmp/opencode/scaffold-journey/work`
- Exit code: `1`
- Elapsed: `0.020 s`
- Note: Fails fast, no directory created, actionable message.

Standard output:

_(empty)_

Standard error:

```text
error: --with requires a template that generates ag.toml (for example --template layered); template 'rest' has no manifest
```

## Step 12: Offline cargo check of the web template

- Command: `env CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-gen-target/website cargo check --offline`
- Working directory: `/tmp/opencode/scaffold-journey/work/website`
- Exit code: `0`
- Elapsed: `67.379 s`
- Note: Cold check of a separate target dir.

Standard output:

_(empty)_

Standard error:

```text
     Locking 210 packages to highest compatible versions
      Adding axum v0.7.9 (available: v0.8.9)
      Adding generic-array v0.14.7 (available: v0.14.9)
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.27
   Compiling quote v1.0.47
   Compiling libc v0.2.191
    Checking cfg-if v1.0.5
   Compiling version_check v0.9.5
    Checking subtle v2.6.1
    Checking const-oid v0.9.6
    Checking typenum v1.20.1
    Checking pin-project-lite v0.2.18
    Checking itoa v1.0.19
    Checking bytes v1.12.2
    Checking once_cell v1.21.4
    Checking base64ct v1.8.3
   Compiling serde_core v1.0.229
    Checking futures-core v0.3.34
   Compiling zerocopy v0.8.64
    Checking smallvec v1.16.6
    Checking futures-sink v0.3.34
    Checking pem-rfc7468 v0.7.0
   Compiling serde v1.0.229
   Compiling autocfg v1.5.1
    Checking cpufeatures v0.2.17
   Compiling libm v0.2.16
    Checking slab v0.4.12
   Compiling generic-array v0.14.7
    Checking tracing-core v0.1.36
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.15
    Checking base16ct v0.2.0
    Checking hashbrown v0.17.2
    Checking futures-task v0.3.34
   Compiling cc v1.11.3
    Checking equivalent v1.0.3
    Checking http v1.5.0
   Compiling httparse v1.10.1
    Checking futures-util v0.3.34
   Compiling num-traits v0.2.19
   Compiling syn v2.0.119
   Compiling syn v3.0.10
   Compiling zmij v1.0.23
   Compiling semver v1.0.28
    Checking indexmap v2.14.2
    Checking spin v0.9.9
    Checking http-body v1.1.0
    Checking lazy_static v1.5.1
   Compiling rustc_version v0.4.1
   Compiling parking_lot_core v0.9.12
    Checking tower-service v0.3.3
   Compiling crossbeam-utils v0.8.23
    Checking log v0.4.35
   Compiling thiserror v1.0.69
    Checking getrandom v0.2.17
    Checking errno v0.3.15
    Checking socket2 v0.6.5
    Checking signal-hook-registry v1.4.8
    Checking rand_core v0.6.4
    Checking mio v1.2.5
   Compiling serde_json v1.0.154
    Checking ff v0.13.1
    Checking scopeguard v1.2.0
    Checking lock_api v0.4.14
    Checking group v0.13.0
   Compiling curve25519-dalek v4.1.3
    Checking bitflags v2.13.2
    Checking fnv v1.0.7
    Checking sync_wrapper v1.0.2
   Compiling thiserror v2.0.21
    Checking memchr v2.8.3
    Checking atomic-waker v1.1.2
   Compiling rustversion v1.0.23
    Checking num-integer v0.1.47
   Compiling time-core v0.1.9
    Checking untrusted v0.9.0
   Compiling num-bigint-dig v0.8.6
    Checking tower-layer v0.3.3
   Compiling num-conv v0.2.2
   Compiling getrandom v0.4.3
    Checking num-iter v0.1.46
    Checking futures-channel v0.3.34
   Compiling time-macros v0.2.32
    Checking powerfmt v0.2.1
   Compiling ring v0.17.14
   Compiling portable-atomic v1.15.0
    Checking httpdate v1.0.3
    Checking base64 v0.22.1
   Compiling rustls v0.23.46
    Checking percent-encoding v2.3.2
    Checking utf-8 v0.7.6
    Checking data-encoding v2.11.1
    Checking deranged v0.5.9
    Checking regex-syntax v0.8.12
    Checking byteorder v1.5.0
    Checking form_urlencoded v1.2.3
    Checking num-bigint v0.4.8
    Checking raw-cpuid v11.6.0
    Checking http-body-util v0.1.5
    Checking ryu v1.0.23
    Checking hashbrown v0.14.5
    Checking winnow v0.7.15
    Checking toml_write v0.1.2
    Checking mime v0.3.18
    Checking ppv-lite86 v0.2.21
    Checking regex-automata v0.4.19
    Checking uuid v1.29.3
    Checking dashmap v6.2.1
    Checking rand_chacha v0.3.1
    Checking time v0.3.55
    Checking pem v3.0.6
    Checking serde_path_to_error v0.1.20
    Checking rand v0.8.8
    Checking quanta v0.12.6
    Checking parking_lot v0.12.5
    Checking spinning_top v0.3.1
    Checking tracing-log v0.2.0
    Checking sharded-slab v0.1.7
    Checking thread_local v1.1.10
    Checking futures-timer v3.0.4
    Checking nonzero_ext v0.3.0
    Checking no-std-compat v0.4.1
    Checking nu-ansi-term v0.50.3
    Checking matchit v0.7.3
    Checking governor v0.7.0
   Compiling tokio-macros v2.7.2
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling async-trait v0.1.92
   Compiling zeroize_derive v1.5.0
   Compiling tracing-attributes v0.1.31
   Compiling thiserror-impl v1.0.69
   Compiling curve25519-dalek-derive v0.1.1
    Checking matchers v0.2.0
    Checking tokio v1.54.1
    Checking zeroize v1.9.2
    Checking axum-core v0.4.5
    Checking der v0.7.10
    Checking rustls-pki-types v1.16.0
    Checking simple_asn1 v0.6.4
    Checking rustls-webpki v0.103.17
    Checking tracing v0.1.44
    Checking block-buffer v0.10.4
    Checking crypto-common v0.1.7
    Checking crypto-bigint v0.5.5
    Checking digest v0.10.7
    Checking hmac v0.12.1
    Checking signature v2.2.0
    Checking sha2 v0.10.9
    Checking sha1 v0.10.7
    Checking hkdf v0.12.4
    Checking rfc6979 v0.4.0
    Checking spki v0.7.3
    Checking tungstenite v0.24.0
    Checking pkcs8 v0.10.2
    Checking sec1 v0.7.3
    Checking pkcs1 v0.7.5
    Checking ed25519 v2.2.3
    Checking rsa v0.9.10
    Checking ed25519-dalek v2.2.0
    Checking toml_datetime v0.6.11
    Checking serde_spanned v0.6.9
    Checking serde_urlencoded v0.7.1
    Checking tracing-serde v0.2.0
    Checking tracing-subscriber v0.3.23
    Checking toml_edit v0.22.27
    Checking elliptic-curve v0.13.8
    Checking primeorder v0.13.6
    Checking ecdsa v0.16.9
    Checking p256 v0.13.2
    Checking p384 v0.13.1
    Checking jsonwebtoken v10.4.0
    Checking toml v0.8.23
    Checking tokio-util v0.7.20
    Checking tower v0.5.3
    Checking tokio-tungstenite v0.24.0
    Checking tokio-rustls v0.26.6
    Checking tower-http v0.6.11
    Checking h2 v0.4.20
    Checking hyper v1.12.0
    Checking hyper-util v0.1.21
    Checking axum v0.7.9
    Checking ag-core v0.0.0 (https://github.com/anti-gravital/anti-gravital?rev=91a95ff61ed6ef4c9d3f1c9069357e97389d1382#91a95ff6)
    Checking website v0.1.0 (/tmp/opencode/scaffold-journey/work/website)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 07s
```

## Step 13: Offline cargo check of the API template

- Command: `env CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-gen-target/hooks cargo check --offline`
- Working directory: `/tmp/opencode/scaffold-journey/work/hooks`
- Exit code: `0`
- Elapsed: `52.609 s`
- Note: Cold check of a separate target dir.

Standard output:

_(empty)_

Standard error:

```text
     Locking 211 packages to highest compatible versions
      Adding axum v0.7.9 (available: v0.8.9)
      Adding generic-array v0.14.7 (available: v0.14.9)
      Adding hmac v0.12.1 (available: v0.13.0)
      Adding sha2 v0.10.9 (available: v0.11.1)
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.27
   Compiling libc v0.2.191
    Checking cfg-if v1.0.5
   Compiling version_check v0.9.5
    Checking subtle v2.6.1
    Checking typenum v1.20.1
    Checking const-oid v0.9.6
   Compiling serde_core v1.0.229
    Checking pin-project-lite v0.2.18
    Checking itoa v1.0.19
    Checking once_cell v1.21.4
    Checking bytes v1.12.2
    Checking base64ct v1.8.3
    Checking futures-core v0.3.34
   Compiling zerocopy v0.8.64
   Compiling serde v1.0.229
    Checking futures-sink v0.3.34
    Checking smallvec v1.16.6
   Compiling libm v0.2.16
    Checking slab v0.4.12
    Checking pem-rfc7468 v0.7.0
   Compiling autocfg v1.5.1
    Checking cpufeatures v0.2.17
   Compiling generic-array v0.14.7
    Checking tracing-core v0.1.36
   Compiling find-msvc-tools v0.1.15
   Compiling shlex v2.0.1
    Checking futures-task v0.3.34
    Checking equivalent v1.0.3
   Compiling httparse v1.10.1
   Compiling cc v1.11.3
    Checking base16ct v0.2.0
    Checking http v1.5.0
    Checking hashbrown v0.17.2
    Checking futures-util v0.3.34
   Compiling zmij v1.0.23
   Compiling num-traits v0.2.19
   Compiling semver v1.0.28
    Checking spin v0.9.9
   Compiling rustc_version v0.4.1
    Checking lazy_static v1.5.1
   Compiling syn v2.0.119
   Compiling syn v3.0.10
    Checking log v0.4.35
   Compiling thiserror v1.0.69
   Compiling serde_json v1.0.154
    Checking indexmap v2.14.2
    Checking http-body v1.1.0
    Checking scopeguard v1.2.0
   Compiling parking_lot_core v0.9.12
    Checking tower-service v0.3.3
   Compiling crossbeam-utils v0.8.23
    Checking lock_api v0.4.14
    Checking getrandom v0.2.17
    Checking errno v0.3.15
    Checking signal-hook-registry v1.4.8
    Checking rand_core v0.6.4
    Checking socket2 v0.6.5
    Checking mio v1.2.5
   Compiling curve25519-dalek v4.1.3
    Checking ff v0.13.1
    Checking tower-layer v0.3.3
    Checking group v0.13.0
   Compiling num-bigint-dig v0.8.6
    Checking fnv v1.0.7
    Checking sync_wrapper v1.0.2
    Checking bitflags v2.13.2
    Checking memchr v2.8.3
   Compiling thiserror v2.0.21
   Compiling rustversion v1.0.23
   Compiling num-conv v0.2.2
    Checking num-integer v0.1.47
    Checking atomic-waker v1.1.2
   Compiling time-core v0.1.9
   Compiling getrandom v0.4.3
    Checking untrusted v0.9.0
   Compiling time-macros v0.2.32
    Checking futures-channel v0.3.34
    Checking num-iter v0.1.46
    Checking deranged v0.5.9
   Compiling ring v0.17.14
    Checking byteorder v1.5.0
    Checking base64 v0.22.1
    Checking powerfmt v0.2.1
    Checking httpdate v1.0.3
   Compiling portable-atomic v1.15.0
    Checking percent-encoding v2.3.2
   Compiling rustls v0.23.46
    Checking regex-syntax v0.8.12
    Checking utf-8 v0.7.6
    Checking data-encoding v2.11.1
    Checking form_urlencoded v1.2.3
    Checking num-bigint v0.4.8
    Checking raw-cpuid v11.6.0
    Checking http-body-util v0.1.5
    Checking winnow v0.7.15
    Checking mime v0.3.18
    Checking toml_write v0.1.2
    Checking ryu v1.0.23
    Checking time v0.3.55
    Checking hashbrown v0.14.5
    Checking uuid v1.29.3
    Checking ppv-lite86 v0.2.21
    Checking regex-automata v0.4.19
    Checking pem v3.0.6
    Checking serde_path_to_error v0.1.20
    Checking parking_lot v0.12.5
    Checking spinning_top v0.3.1
    Checking dashmap v6.2.1
    Checking tracing-log v0.2.0
    Checking rand_chacha v0.3.1
    Checking sharded-slab v0.1.7
    Checking thread_local v1.1.10
    Checking futures-timer v3.0.4
    Checking rand v0.8.8
    Checking nu-ansi-term v0.50.3
    Checking nonzero_ext v0.3.0
    Checking no-std-compat v0.4.1
    Checking matchit v0.7.3
    Checking hex v0.4.3
    Checking quanta v0.12.6
   Compiling tokio-macros v2.7.2
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling async-trait v0.1.92
   Compiling zeroize_derive v1.5.0
   Compiling tracing-attributes v0.1.31
   Compiling thiserror-impl v1.0.69
   Compiling curve25519-dalek-derive v0.1.1
    Checking matchers v0.2.0
    Checking governor v0.7.0
    Checking tokio v1.54.1
    Checking zeroize v1.9.2
    Checking der v0.7.10
    Checking rustls-pki-types v1.16.0
    Checking axum-core v0.4.5
    Checking simple_asn1 v0.6.4
    Checking block-buffer v0.10.4
    Checking crypto-common v0.1.7
    Checking crypto-bigint v0.5.5
    Checking tracing v0.1.44
    Checking digest v0.10.7
    Checking hmac v0.12.1
    Checking signature v2.2.0
    Checking sha2 v0.10.9
    Checking sha1 v0.10.7
    Checking rustls-webpki v0.103.17
    Checking spki v0.7.3
    Checking hkdf v0.12.4
    Checking rfc6979 v0.4.0
    Checking tungstenite v0.24.0
    Checking pkcs8 v0.10.2
    Checking sec1 v0.7.3
    Checking pkcs1 v0.7.5
    Checking ed25519 v2.2.3
    Checking rsa v0.9.10
    Checking elliptic-curve v0.13.8
    Checking ed25519-dalek v2.2.0
    Checking primeorder v0.13.6
    Checking ecdsa v0.16.9
    Checking p384 v0.13.1
    Checking p256 v0.13.2
    Checking toml_datetime v0.6.11
    Checking serde_spanned v0.6.9
    Checking tracing-serde v0.2.0
    Checking serde_urlencoded v0.7.1
    Checking toml_edit v0.22.27
    Checking tracing-subscriber v0.3.23
    Checking jsonwebtoken v10.4.0
    Checking tokio-util v0.7.20
    Checking tower v0.5.3
    Checking tokio-tungstenite v0.24.0
    Checking tokio-rustls v0.26.6
    Checking tower-http v0.6.11
    Checking toml v0.8.23
    Checking h2 v0.4.20
    Checking hyper v1.12.0
    Checking hyper-util v0.1.21
    Checking axum v0.7.9
    Checking ag-core v0.0.0 (https://github.com/anti-gravital/anti-gravital?rev=91a95ff61ed6ef4c9d3f1c9069357e97389d1382#91a95ff6)
    Checking hooks v0.1.0 (/tmp/opencode/scaffold-journey/work/hooks)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 52.41s
```

## Step 14: Offline cargo check of the layered workspace

- Command: `env CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-gen-target/layeredmod cargo check --offline`
- Working directory: `/tmp/opencode/scaffold-journey/work/layeredmod`
- Exit code: `0`
- Elapsed: `96.264 s`
- Note: Includes `ag-data`/`sqlx` from the pinned git rev. Slowest check.

Standard output:

_(empty)_

Standard error:

```text
     Locking 300 packages to highest compatible versions
      Adding axum v0.7.9 (available: v0.8.9)
      Adding crypto-common v0.1.6 (available: v0.1.7)
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.27
   Compiling libc v0.2.191
   Compiling serde_core v1.0.229
   Compiling version_check v0.9.5
   Compiling serde v1.0.229
    Checking cfg-if v1.0.5
    Checking subtle v2.6.1
    Checking typenum v1.20.1
    Checking const-oid v0.9.6
   Compiling autocfg v1.5.1
    Checking pin-project-lite v0.2.18
   Compiling zerocopy v0.8.64
    Checking bytes v1.12.2
    Checking itoa v1.0.19
   Compiling find-msvc-tools v0.1.15
    Checking once_cell v1.21.4
   Compiling shlex v2.0.1
    Checking futures-core v0.3.34
   Compiling generic-array v0.14.9
   Compiling stable_deref_trait v1.2.1
   Compiling cc v1.11.3
   Compiling icu_properties_data v2.4.0
   Compiling parking_lot_core v0.9.12
   Compiling icu_normalizer_data v2.4.0
   Compiling zmij v1.0.23
    Checking base64ct v1.8.3
    Checking futures-sink v0.3.34
   Compiling libm v0.2.16
   Compiling num-traits v0.2.19
   Compiling thiserror v2.0.21
    Checking cpufeatures v0.2.17
    Checking log v0.4.35
    Checking slab v0.4.12
   Compiling syn v3.0.10
   Compiling syn v2.0.119
    Checking pem-rfc7468 v0.7.0
    Checking memchr v2.8.3
   Compiling serde_json v1.0.154
    Checking tracing-core v0.1.36
    Checking equivalent v1.0.3
   Compiling rustls v0.23.46
    Checking http v1.5.0
    Checking getrandom v0.2.17
    Checking errno v0.3.15
    Checking rand_core v0.6.4
    Checking socket2 v0.6.5
    Checking signal-hook-registry v1.4.8
    Checking mio v1.2.5
   Compiling litemap v0.8.3
   Compiling writeable v0.6.4
   Compiling utf_types v1.0.0
    Checking futures-task v0.3.34
    Checking hashbrown v0.17.2
    Checking futures-io v0.3.34
   Compiling utf8_iter v1.1.0
    Checking futures-util v0.3.34
    Checking scopeguard v1.2.0
   Compiling crossbeam-utils v0.8.23
    Checking lock_api v0.4.14
   Compiling zeroize v1.9.2
   Compiling ring v0.17.14
    Checking indexmap v2.14.2
   Compiling rustls-pki-types v1.16.0
   Compiling crypto-common v0.1.6
   Compiling block-buffer v0.10.4
   Compiling digest v0.10.7
    Checking ff v0.13.1
   Compiling getrandom v0.4.3
   Compiling httparse v1.10.1
    Checking percent-encoding v2.3.2
    Checking untrusted v0.9.0
    Checking base16ct v0.2.0
    Checking form_urlencoded v1.2.3
    Checking group v0.13.0
    Checking num-integer v0.1.47
    Checking http-body v1.1.0
    Checking base64 v0.22.1
    Checking spin v0.9.9
    Checking bitflags v2.13.2
   Compiling semver v1.0.28
    Checking lazy_static v1.5.1
   Compiling rustc_version v0.4.1
    Checking futures-channel v0.3.34
    Checking byteorder v1.5.0
   Compiling allocator-api2 v0.2.21
   Compiling thiserror v1.0.69
   Compiling synstructure v0.14.0
    Checking tower-service v0.3.3
   Compiling foldhash v0.1.5
   Compiling curve25519-dalek v4.1.3
   Compiling hashbrown v0.15.5
    Checking uuid v1.29.3
    Checking ppv-lite86 v0.2.21
   Compiling webpki-roots v1.0.12
   Compiling tokio v1.54.1
    Checking tower-layer v0.3.3
    Checking rand_chacha v0.3.1
    Checking fnv v1.0.7
   Compiling tinyvec v1.13.3
    Checking sync_wrapper v1.0.2
   Compiling rustversion v1.0.23
   Compiling num-conv v0.2.2
    Checking rand v0.8.8
    Checking atomic-waker v1.1.2
   Compiling crc-catalog v2.5.0
   Compiling iana-time-zone v0.1.65
   Compiling parking v2.2.1
   Compiling time-core v0.1.9
   Compiling num-bigint-dig v0.8.6
   Compiling time-macros v0.2.32
   Compiling unicode-normalization v0.1.26
   Compiling sha2 v0.10.9
   Compiling event-listener v5.4.2
   Compiling chrono v0.4.45
   Compiling serde_derive v1.0.229
   Compiling zerofrom-derive v0.1.8
   Compiling yoke-derive v0.8.4
   Compiling zeroize_derive v1.5.0
   Compiling zerovec-derive v0.11.7
   Compiling displaydoc v0.2.7
   Compiling tracing-attributes v0.1.31
   Compiling tokio-macros v2.7.2
    Checking der v0.7.10
   Compiling thiserror-impl v2.0.21
   Compiling zerofrom v0.1.8
   Compiling yoke v0.8.4
    Checking zerotrie v0.2.5
    Checking crypto-bigint v0.5.5
    Checking hmac v0.12.1
   Compiling zerovec v0.11.9
    Checking hkdf v0.12.4
    Checking signature v2.2.0
    Checking rustls-webpki v0.103.17
    Checking rfc6979 v0.4.0
    Checking tracing v0.1.44
   Compiling thiserror-impl v1.0.69
    Checking spki v0.7.3
    Checking tinystr v0.8.5
    Checking potential_utf v0.1.6
    Checking pkcs8 v0.10.2
    Checking icu_locale_core v2.4.0
    Checking sec1 v0.7.3
    Checking icu_collections v2.4.0
    Checking elliptic-curve v0.13.8
    Checking sha1 v0.10.7
   Compiling curve25519-dalek-derive v0.1.1
   Compiling crc v3.4.0
    Checking icu_provider v2.4.0
    Checking primeorder v0.13.6
    Checking ecdsa v0.16.9
    Checking smallvec v1.16.6
    Checking icu_properties v2.4.0
    Checking icu_normalizer v2.4.0
   Compiling either v1.19.0
    Checking parking_lot v0.12.5
   Compiling tokio-stream v0.1.19
   Compiling hashlink v0.10.0
   Compiling crossbeam-queue v0.3.14
   Compiling webpki-roots v0.26.11
    Checking num-iter v0.1.46
    Checking deranged v0.5.9
   Compiling futures-intrusive v0.5.0
    Checking powerfmt v0.2.1
    Checking httpdate v1.0.3
    Checking data-encoding v2.11.1
   Compiling portable-atomic v1.15.0
    Checking idna_adapter v1.2.2
   Compiling unicode-properties v0.1.5
   Compiling idna v1.1.0
   Compiling unicode-bidi v0.3.18
    Checking utf-8 v0.7.6
    Checking regex-syntax v0.8.12
    Checking tungstenite v0.24.0
    Checking tokio-util v0.7.20
    Checking tower v0.5.3
    Checking time v0.3.55
   Compiling url v2.5.9
   Compiling stringprep v0.1.5
    Checking regex-automata v0.4.19
    Checking toml_datetime v0.6.11
    Checking serde_spanned v0.6.9
    Checking pkcs1 v0.7.5
    Checking h2 v0.4.20
    Checking ed25519 v2.2.3
   Compiling async-trait v0.1.92
   Compiling atoi v2.0.0
    Checking raw-cpuid v11.6.0
    Checking num-bigint v0.4.8
    Checking http-body-util v0.1.5
   Compiling md-5 v0.10.6
    Checking winnow v0.7.15
    Checking ryu v1.0.23
    Checking toml_write v0.1.2
    Checking mime v0.3.18
   Compiling home v0.5.12
    Checking hashbrown v0.14.5
   Compiling whoami v1.6.1
   Compiling dotenvy v0.15.7
   Compiling hex v0.4.3
   Compiling sqlx-core v0.8.6
    Checking simple_asn1 v0.6.4
    Checking matchers v0.2.0
    Checking axum-core v0.4.5
    Checking dashmap v6.2.1
    Checking serde_urlencoded v0.7.1
    Checking ed25519-dalek v2.2.0
    Checking rsa v0.9.10
    Checking tokio-tungstenite v0.24.0
    Checking quanta v0.12.6
    Checking p256 v0.13.2
    Checking toml_edit v0.22.27
    Checking p384 v0.13.1
    Checking tracing-serde v0.2.0
    Checking pem v3.0.6
    Checking sharded-slab v0.1.7
    Checking spinning_top v0.3.1
    Checking serde_path_to_error v0.1.20
    Checking tracing-log v0.2.0
    Checking thread_local v1.1.10
    Checking no-std-compat v0.4.1
    Checking nu-ansi-term v0.50.3
    Checking matchit v0.7.3
   Compiling heck v0.5.0
    Checking futures-timer v3.0.4
    Checking nonzero_ext v0.3.0
    Checking jsonwebtoken v10.4.0
    Checking tower-http v0.6.11
    Checking tokio-rustls v0.26.6
    Checking tracing-subscriber v0.3.23
    Checking governor v0.7.0
    Checking hyper v1.12.0
    Checking layeredmod-domain v0.1.0 (/tmp/opencode/scaffold-journey/work/layeredmod/crates/domain)
    Checking layeredmod-app v0.1.0 (/tmp/opencode/scaffold-journey/work/layeredmod/crates/app)
    Checking toml v0.8.23
    Checking hyper-util v0.1.21
   Compiling sqlx-postgres v0.8.6
    Checking axum v0.7.9
    Checking ag-core v0.0.0 (https://github.com/anti-gravital/anti-gravital?rev=91a95ff61ed6ef4c9d3f1c9069357e97389d1382#91a95ff6)
    Checking layeredmod-api v0.1.0 (/tmp/opencode/scaffold-journey/work/layeredmod/crates/api)
   Compiling sqlx-macros-core v0.8.6
   Compiling sqlx-macros v0.8.6
    Checking sqlx v0.8.6
    Checking ag-data v0.0.0 (https://github.com/anti-gravital/anti-gravital?rev=91a95ff61ed6ef4c9d3f1c9069357e97389d1382#91a95ff6)
    Checking layeredmod-infra v0.1.0 (/tmp/opencode/scaffold-journey/work/layeredmod/crates/infra)
    Checking layeredmod v0.1.0 (/tmp/opencode/scaffold-journey/work/layeredmod)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 35s
```

## Step 8 detail: `ag dev` and the HTTP probe

The first cold `ag dev` attempt (no prebuilt debug binary) printed the
startup banner and then spent more than 65 s compiling dependencies; it
was stopped at the cutoff before the listener opened. After a debug
build, the same command served the first request 2 s after launch.

Exact commands:

```bash
cd /tmp/opencode/scaffold-journey/work/demo
CARGO_TARGET_DIR=<gen-target>/demo TMPDIR=<gen-target>/tmp \
  timeout 90 /home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-private-target/debug/ag dev --bind 127.0.0.1:8080 >dev.out 2>dev.err &
curl -sS --max-time 2 -o curl.out -w '%{http_code}' http://127.0.0.1:8080/
```

- Startup banner: ready after 2 s (warm debug build), HTTP code `200`.
- `curl` body: `demo corriendo con Anti-Gravital`.
- Listener check before the run: port 8080 free; after stopping: free.
- Exit code of the curl probe: `0`; `ag dev` was terminated intentionally.

`dev.out` verbatim (ANSI colour sequences are present even though the
stream was redirected to a file; shown as captured):

```text
Starting development server (bind: 127.0.0.1:8080)...
cargo-watch not found. Running without hot reload.
To enable hot reload: cargo install cargo-watch
[2m2026-11-22T13:32:26.003582Z[0m [32m INFO[0m [2mdemo[0m[2m:[0m escuchando en http://127.0.0.1:8080
[2m2026-11-22T13:32:26.751507Z[0m [32m INFO[0m [2mag_core::shield::logging[0m[2m:[0m request completed [3mmethod[0m[2m=[0mGET [3mpath[0m[2m=[0m/ [3mstatus[0m[2m=[0m200 [3mlatency_ms[0m[2m=[0m0.040422

demo corriendo con Anti-Gravital
```

`dev.err` tail (the build was already warm):

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running `/home/angel-nereira/Github-local-angelnereira/worktrees10/scaffold-gen-target/demo/debug/demo`
```

## Findings

### F1. Documentation defect (opened as issue #205)

`README.md` still warns that generated scaffolds hardcode `0.0.0.0:8080`
and ignore `BIND`, and cites issue #191 as if it were open. Issue #191
is CLOSED. Direct observation contradicts the README: the `rest`
template reads `BIND` and defaults to `127.0.0.1:8080`; `ag dev --bind
127.0.0.1:8080` produced a loopback-only listener; no template under
`templates/` contains a `0.0.0.0` bind literal. The stale claims are at
README lines 118, 135-136, 331 and the Spanish mirror at 677, 695-696,
899. `docs/examples/alpha-prototype.md` already documents the correct
behaviour. Issue #205 was opened with `documentation`, `p2`, `ag-cli`
and `good first issue` labels. No issue was closed.

### F2. Cold `ag dev` is slow and gives no startup ETA

On a fresh scaffold, `ag dev` invokes plain `cargo run` (cargo-watch is
not installed here) and compiles roughly 210 crates before the listener
opens. The cold attempt was still compiling after 65 s; the complete
debug build then took about 2.5 minutes in total. The only guidance is
`cargo-watch not found. Running without hot reload.` There is no message
that a first-time compile is in progress or how long it will take.
Not a functional failure, but a measurable first-run friction.

### F3. Pinned git dependency and a fresh 210-package lock

Every generated `Cargo.toml` pins `ag-core` (and siblings) to git rev
`91a95ff61ed6ef4c9d3f1c9069357e97389d1382`, which resolves as
`ag-core v0.0.0`, not the shipped `0.1.0-alpha.2` workspace crates.
`cargo check --offline` succeeded here only because that checkout was
already in `~/.cargo/git`; `docs/examples/alpha-prototype.md` explicitly
notes that the first build needs network or a warm cache. The lock also
selected `axum v0.7.9` while `v0.8.9` was available. Expected for a
pinned alpha, but it means a generated prototype is not built from the
release being tested.

### F4. Coloured log output when redirected

`dev.out` contains ANSI escape sequences and a Spanish log message
(`escuchando en http://...`) even when stdout/stderr are files, not a
terminal. Harmless, but it complicates log scraping in scripts.

### F5. What worked cleanly

No functional defect was found in `ag --version`, `ag new` (four
templates plus `--with`), `ag schema lint`, `ag generate`, `ag build`,
`ag dev` or the HTTP response. The `--with` misuse error is explicit and
fails before writing any directory. All four generated projects passed
`cargo check --offline`. Only issue #205 was opened; nothing was closed.

## Verification performed in the worktree

```bash
git diff --check
# prohibited-content scans: emojis, AI-tool attribution strings,
# third-party mail-brand names and IA branch prefixes in tracked paths
```

Results are recorded in the commit that introduces this document.
