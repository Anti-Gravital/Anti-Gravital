# Reference — migration and compatibility

How `ag-domains` and `ag-edge` evolve without breaking users.

## Native-first, additive features

The control plane works with no external service (ADR-0009). Everything beyond
the native default is an opt-in Cargo feature, so enabling one never changes
default behavior:

| Feature (crate) | Adds |
|---|---|
| `acme` (ag-domains, default) | ACME issuance/renewal |
| `propagation` (ag-domains, default) | resolver-backed verification |
| `cloudflare` (ag-domains) | a `DnsProvider` adapter |
| `api` (ag-domains) | the REST control-plane surface |
| `sql-store` (ag-domains) | a Postgres-backed attachment store |
| `server` / `tls` (ag-edge) | runnable HTTP / HTTPS+SNI listeners |

The default store is in-memory/JSON; the SQL store is an accelerator for
multi-node operation, not a requirement.

## Store migration

Attachments serialize as stable JSON (`store::JsonFileStore`), so moving from the
in-memory/JSON store to the SQL store (or vice versa) is a data copy, not a
schema rewrite. Tombstones migrate with attachments to preserve anti-takeover
guarantees.

## Manual flow is always available

Provider automation (adapter SDK, Domain Connect, DNS-01 wildcard automation) is
additive over the manual flow. The manual flow — print instructions, publish
records, verify — works at every provider and reaches the same verified state, so
adopting automation later does not invalidate existing attachments.

## API stability

Public APIs follow SemVer. The `DnsProvider` trait is intentionally small and
covered by contract tests so adapters do not drift; the declarative adapter SDK
(`reference/provider-adapter-sdk.md`) is layered on top without changing it.

## Backward-compatible routing

The edge router accepts a caller-supplied legacy resolver, so pre-existing
(non-custom-domain) routing keeps working while custom-domain bindings are added
(`explanation/routing-host-sni.md`).
