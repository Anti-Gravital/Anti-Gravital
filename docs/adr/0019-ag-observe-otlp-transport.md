# ADR-0019 - ag-observe OTLP transport without protoc

- Status: accepted
- Date: 2026-11-21
- Decider: project maintainer (issue #149 resolution)

## Context

`ag-observe` kept its OTLP exporter deferred. Two `TECH-DEBT` markers claimed
that `opentelemetry-otlp` needs `protoc` at build time through `grpc-tonic`,
that this breaks `ci/build windows-x64`, and that `opentelemetry_sdk` needs an
active Tokio runtime. Both markers described an older release, not the
versions actually pinned in the workspace (`opentelemetry-otlp` 0.33.x,
`opentelemetry-proto` 0.33.x, `opentelemetry_sdk` 0.33.x).

Inspection of those crate sources shows:

1. `opentelemetry-otlp` 0.33.x does not enable `grpc-tonic` by default. Its
   defaults are `http-proto`, `reqwest-blocking-client`, `trace`, `metrics`,
   `logs` and `internal-logs`.
2. `opentelemetry-proto` 0.33.x has no `build.rs`. The generated Rust modules
   are committed to the crate and `tonic-prost-build` is a dev-dependency only.
   No feature runs a protobuf compiler at build time.
3. `opentelemetry_sdk` 0.33.x uses a thread-based `BatchSpanProcessor` by
   default. Its own documentation states that `reqwest-blocking-client` works
   with a regular `main` as well as with `#[tokio::main]`; only `grpc-tonic`
   requires the provider to be created inside a Tokio runtime.

The `protoc` and runtime constraints were therefore real for the historical
gRPC path but are not present in the pinned versions. Keeping the markers was
inaccurate technical debt.

## Decision

1. Re-integrate the OTLP exporter behind an opt-in Cargo feature named `otlp`,
   disabled by default.
2. Use the HTTP/protobuf transport: `opentelemetry-otlp` with
   `default-features = false` and `features = ["http-proto",
   "reqwest-blocking-client"]`. The `grpc-tonic` feature is never enabled.
3. `init()` adds a `tracing-opentelemetry` layer backed by an
   `SdkTracerProvider` with a batch exporter only when the feature is enabled
   and `OTEL_EXPORTER_OTLP_ENDPOINT` is set. Without the feature, setting the
   endpoint still returns `ObserveError::OtlpSetup`.
4. No CI change is needed: multiplatform CI does not provision `protoc` because
   no build step needs it. This is the documented equivalent of the
   build-time-arrangement condition in issue #149.
5. Both `TECH-DEBT` markers (workspace and `ag-observe` manifests) are removed.
6. The `otlp` feature installs the `ring` rustls provider idempotently before
   building the exporter client. reqwest 0.13 features are unified across the
   graph, so a workspace using `rustls-no-provider` (issue #177) would otherwise
   panic when the OTLP client is built. This follows the convention introduced
   for the native mail sender in issue #181.

## Consequences

- The native logging and Prometheus exporters remain the default and need no
  external collector, preserving ADR-0009 rule 2.
- Enabling `otlp` adds `opentelemetry-otlp` and its `reqwest` 0.13 blocking
  client transitively. `cargo deny` reports the second `reqwest` major version
  as a warning (`multiple-versions = "warn"`), not an error. The blocking
  client is required by the thread-based batch processor; async clients
  (`reqwest-client`, `hyper-client`) are incompatible with it and are not used.
- Traces are exported on a background thread; a collector must be reachable at
  the configured endpoint, but the exporter is built without contacting it.
- Pinning policy: do not enable `grpc-tonic` without re-evaluating both the
  `protoc` availability in every CI runner and the Tokio runtime requirement.

## Alternatives

- **Enable `grpc-tonic` and provision `protoc` in every runner:** rejected, it
  adds Windows setup and a Tokio runtime requirement for no functional gain
  in this phase.
- **Keep the exporter deferred and the markers in place:** rejected, the
  markers are factually wrong for the pinned versions and the issue cannot be
  closed with an honest status.
- **Make OTLP export the default:** rejected, it would make an external
  collector part of the default experience and violate ADR-0009.

## Status

Accepted and implemented in `crates/ag-observe` for issue #149.
