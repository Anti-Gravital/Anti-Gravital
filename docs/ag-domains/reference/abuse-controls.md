# Reference — abuse controls (blueprint section 15.6)

Status: implemented (`ag_domains::abuse`).

Beyond the per-registered-domain issuance counter (`ag_domains::issuance`), the
control plane enforces three abuse dimensions. All checks are pure, lock-based
and deterministic (no clock, no network).

| Dimension | Helper | Rejection |
|---|---|---|
| Per-tenant attachment limit | `register_attachment(tenant)` / `release_attachment(tenant)` | `TenantAttachmentLimit` |
| Per-tenant issuance limit | `record_issuance(tenant)` | `TenantIssuanceLimit` |
| Global ACME issuance queue | `acquire_acme_slot()` -> `AcmePermit` | `GlobalAcmeQueueFull` |

A limit of `0` means "unlimited" for that dimension, so a deployment enables
only the controls it needs (`AbuseLimits`).

## Global ACME queue

`acquire_acme_slot` bounds the number of *concurrent* in-flight ACME orders
across all tenants, so a burst cannot exhaust the CA rate budget or the issuance
worker pool. It returns an `AcmePermit`; the slot is held only while the permit
is alive and is freed automatically on drop (RAII), including on panic.

## REST API enforcement

The REST API enforces the per-tenant attachment limit when abuse controls are
wired in:

```rust
let state = ApiState::new(store, edge)
    .with_abuse_controls(AbuseControls::new(AbuseLimits::default()));
```

`POST /attachments` reserves a tenant slot before persisting (returning HTTP
`429 Too Many Requests` at the limit), and `POST /attachments/{id}/detach`
releases it. Without `with_abuse_controls`, attachments stay unbounded (prior
behaviour). The tenant key is the attachment's project id.
