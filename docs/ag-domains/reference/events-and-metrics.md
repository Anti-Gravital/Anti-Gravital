# Reference — events and metrics

## Domain events (`ag_domains::events`)

Control-plane operations emit `DomainEvent`s to an `EventSink`. The default sink
discards events (`NullEventSink`); use `InMemoryEventSink` (native, ordered) or
`TracingEventSink`. No external broker is required.

| Event type | Variant | Emitted by |
|---|---|---|
| `domain.attachment.created` | `AttachmentCreated { id, hostname }` | REST API create |
| `domain.ownership.verified` | `OwnershipVerified { id, hostname }` | reserved (verification flow) |
| `domain.detached` | `Detached { id, hostname }` | REST API detach |

Wire a sink into the REST API with `ApiState::new(store, edge).with_events(sink)`.
Events serialize to JSON with a `event` tag, e.g.
`{"event":"detached","id":"dom_...","hostname":"example.com"}`.

Additional blueprint events (`domain.dns.routable`, `domain.tls.active`,
`domain.activated`, `domain.tombstoned`, `domain.dangling_dns_detected`) are
emitted as their operations land (DEBT-018); they are intentionally not modelled
before they can fire.

## Metrics (`ag_domains::metrics`)

Counters/gauges/histograms via the `metrics` crate (exported through
`ag-observe`). Calls are no-ops when no recorder is registered.

| Metric | Kind | Helper |
|---|---|---|
| `ag_domains_attachments_total` | counter | `record_attachment_created` |
| `ag_domains_detached_total` | counter | `record_attachment_detached` |
| `ag_domains_verification_failures_total` | counter | `record_verification_failure` |
| `ag_domains_records_upsert_total` | counter | `record_dns_upsert` |
| `ag_domains_acme_renewal_total` | counter | `record_acme_renewal` |
| `ag_domains_cert_days_until_expiry` | gauge | `set_cert_days_until_expiry` |
| `ag_domains_propagation_latency_seconds` | histogram | `record_propagation_latency` |

The full blueprint §16.1 set (active gauge, TLS/DNS state counters, edge cache
hit-ratio, route-resolution latency) is partially wired; see DEBT-018.
