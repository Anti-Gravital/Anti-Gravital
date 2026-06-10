# Provider adapter SDK (plan / diff / apply / verify / rollback)

Status: implemented (`ag-domains::provider::sdk`). Blueprint section 11.

The base `DnsProvider` trait is a thin CRUD surface (`zone_id`, `list_records`,
`create_record`, `update_record`, `delete_record`). Real provider automation
needs a declarative contract on top of it. The provider adapter SDK adds that
seam **without changing `DnsProvider`** and **without changing the default**:
manual DNS instructions remain the default path (ADR-0009); the SDK runs only
when a provider adapter is explicitly configured.

## Model

| Type | Role |
|---|---|
| `ZoneRef` | A resolved zone: registrable `domain` + provider `zone_id`. |
| `ZoneChange` | One declarative change: `Create`, `Update { record_id, from, to }`, `Delete { record_id, record }`. |
| `ZonePlan` | An ordered set of `ZoneChange`. A dry-run artifact first, an execution unit second. |
| `ChangeRef` | Returned by `apply`; carries the inverse plan so `rollback` is `apply(inverse)`. |
| `VerifyOutcome` | `Applied`, or `Drift { detail }` if a change did not take effect. |

## The pure diff

`diff(desired: &[DnsRecordSpec], observed: &[DnsRecord]) -> ZonePlan` is pure and
deterministic (no I/O), and is the core of the seam. Matching rules:

- `A` / `AAAA` / `CNAME` are **single-valued** per name: a changed value is an
  `Update` of the same record identity.
- `TXT` / `MX` are **multi-valued**: a changed value is a `Delete` of the old
  value plus a `Create` of the new one.
- Names are compared case-insensitively and with any trailing dot stripped.

### Deletion-scope contract

`diff` treats `desired` as the **authoritative set for the scope that
`observed` represents**. Observed records absent from `desired` are scheduled
for deletion. Callers that do not own the whole zone MUST scope `observed` to
the records `ag-domains` manages (for example, the `_acme-challenge` subtree)
before calling `diff`; otherwise unrelated records would be planned for
deletion. The boundary is explicit at the call site by design.

## The adapter trait

```rust
#[async_trait]
pub trait ZoneAdapter: Send + Sync {
    fn name(&self) -> &'static str;
    async fn discover(&self, domain: &str) -> Result<ZoneRef, AgDomainsError>;
    async fn read(&self, zone: &ZoneRef) -> Result<Vec<DnsRecord>, AgDomainsError>;
    async fn apply(&self, zone: &ZoneRef, plan: &ZonePlan) -> Result<ChangeRef, AgDomainsError>;
    async fn verify(&self, zone: &ZoneRef, plan: &ZonePlan) -> Result<VerifyOutcome, AgDomainsError>;
    async fn rollback(&self, zone: &ZoneRef, change: &ChangeRef) -> Result<(), AgDomainsError>;
}
```

Adapters rarely implement this by hand. `ProviderAdapter<P: DnsProvider>` wraps
any existing CRUD provider and gets the full seam for free: the plan is executed
through the CRUD trait, and the inverse plan is built from the records the
provider returns. The Cloudflare adapter participates with zero adapter-specific
code.

```rust
use ag_domains::{ProviderAdapter, ZoneAdapter, zone_diff};

let adapter = ProviderAdapter::new(provider); // any DnsProvider
let zone = adapter.discover("example.com").await?;
let observed = adapter.read(&zone).await?;
let plan = zone_diff(&desired, &observed); // review plan.summary() before applying
let change = adapter.apply(&zone, &plan).await?;
assert!(matches!(adapter.verify(&zone, &plan).await?, VerifyOutcome::Applied));
// adapter.rollback(&zone, &change).await?; // if needed
```

## Tests

The pure `diff` cases and the apply/verify/rollback round-trip are unit-tested
against an in-memory mock `DnsProvider` (no network) in
`crates/ag-domains/src/provider/sdk.rs`.
