# ADR-0018 - reqwest 0.13 keeps the ring crypto provider

- Status: accepted
- Date: 2026-11-21
- Decider: BDFL (Angel Nereira)
- RFC origin: none (dependency upgrade recorded in issue #177)

## Context

`reqwest` 0.12 exposed the `rustls-tls` feature, which selected the `ring`
provider. Version 0.13 removes that feature. Its `rustls` feature selects
`aws-lc-rs`, and its `rustls-no-provider` feature leaves provider selection to
the application. The feature `default-tls` now aliases `rustls`, so leaving
Cargo defaults on also selects `aws-lc-rs`.

The workspace already standardizes on `ring`: `rustls` and `tokio-rustls` are
declared with the `ring` feature, `sqlx` uses `tls-rustls-ring-webpki`, and
`ag-mail`'s native MTA installs `ring` as the process default (issue #181).
With two providers linked, code that relies on implicit process-default
selection can panic at runtime. `reqwest` 0.13 also moves
`RequestBuilder::form` and `RequestBuilder::query` behind the `form` and
`query` features; `ag-auth` and `ag-domains` call both.

Under `cargo test --workspace --all-features`, `object_store` (through
`ag-storage/s3`) enables `reqwest`'s `rustls` feature independently of our
workspace declaration, so `aws-lc-rs` is still linked in that configuration.
The provider that actually signs and verifies is the one installed explicitly:
`reqwest` 0.13 checks `CryptoProvider::get_default()` first and only falls
back to `aws-lc-rs` when no default was installed.

## Decision

1. Pin `reqwest` 0.13 with `default-features = false` and the features
   `rustls-no-provider`, `json`, `stream`, `form` and `query`. Never enable
   `rustls`, `default-tls` or a native TLS feature through the workspace
   declaration.
2. Install `ring` explicitly with the idempotent pattern before any
   `reqwest::Client` is built:
   `let _ = rustls::crypto::CryptoProvider::install_default(rustls::crypto::ring::default_provider());`
   The returned error is ignored on purpose so repeated calls at different
   composition points are safe.
3. Apply that install at every composition point that builds a client: OAuth
   in `ag-auth`, the DNS provider adapters in `ag-domains`, the AI and
   auth-mail examples, the native benchmark, and every test binary that
   constructs a `reqwest` client. `AgAuth::new` also installs it, but callers
   that build the injected client themselves must install before building the
   client: installing inside `AgAuth::new` is too late for a client that was
   already constructed.
4. Keep a test that performs a verified TLS handshake through `reqwest`
   against a local rustls server (`crates/ag-core/tests/shield_tls.rs`). With
   `rustls-no-provider`, an un-installed provider makes client construction
   panic, so a misconfiguration fails CI. The test also asserts that a
   process-default provider is present.

## Consequences

- No `aws-lc-rs` is linked in the default workspace build: the default feature
  set pulls only `rustls-no-provider`. A second provider can still be linked
  under `--all-features` through `object_store`, but it never becomes the
  process default because our install runs first.
- Client construction fails closed (panic) if a code path forgets the install.
  This is intentional and covered by the TLS handshake test.
- The `ring` install is a process-global side effect. A different crate that
  installs another default before ours would win; the install is therefore
  repeated at each composition point instead of being centralized in a single
  late initializer. This is a small, explicit duplication that avoids adding
  heavy cross-crate dependencies (`ag-core` is not a dependency of `ag-auth`).
- 30-second request timeouts on the DNS adapters are unchanged.

## Alternatives considered

- `reqwest` feature `rustls` (aws-lc-rs): rejected. It links a second provider
  in the default build and silently changes the provider used by clients that
  do not install one, reintroducing the failure fixed in issue #181.
- Move the whole workspace to `aws-lc-rs`: rejected. It changes TLS behavior in
  every crate, needs a C toolchain on some targets, and contradicts the
  documented ring standard.
- Centralize the helper in `ag-core`: rejected for now. `ag-auth`, `ag-edge`,
  `ai-backend` and the integration tests do not depend on `ag-core`; adding
  that dependency purely for a three-line helper would be a heavier change to
  the dependency graph than the explicit local install.
- OTLP transport coupling (issue #149): `opentelemetry-otlp` 0.33.1 with the
  `reqwest-client` feature requires `reqwest` 0.13.1 and would enable the
  `rustls` feature, pulling `aws-lc-rs` into the process. Prefer the
  `hyper-client` feature, which reuses the workspace `hyper` stack. If
  `reqwest-client` is chosen anyway, it must not enable `rustls` and the
  application must install `ring` per this ADR.

## Notes

- Issue #177 (upgrade and decision), issue #181 (two-provider panic in the
  native MTA), ADR-0009 rule 2 (native default, external dependency behind a
  feature), ADR-0008 (English for technical documentation).
- Security review by the seguridad role endorsed `rustls-no-provider` and the
  idempotent explicit install pattern.
