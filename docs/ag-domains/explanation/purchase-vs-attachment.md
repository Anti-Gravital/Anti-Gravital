# Explanation — purchase vs attachment

A frequent confusion: "attaching" a domain to Anti-Gravital is not the same as
"buying" a domain. `ag-domains` does the second thing only.

## Purchase (out of scope)

Buying, transferring or renewing a domain is a commercial transaction with a
registrar (the company that sells the name). `ag-domains` is **not** a registrar
and deliberately does not do this (RFC-0011 §3.2). You buy your domain wherever
you like — any registrar works.

A future, separate module (`ag-registrars`) may cover purchase/transfer/renewal
commerce; it is intentionally kept out of the attachment lifecycle so attaching
stays provider-agnostic.

## Attachment (what ag-domains does)

Attachment is the operational act of pointing a domain you already own at an
Anti-Gravital project and proving you control it:

1. declare the attachment and get the exact DNS records to publish,
2. prove ownership with the `_ag-domain` TXT record,
3. route the hostname to the edge,
4. secure it with a managed TLS certificate.

Attachment is provider-agnostic: it works the same whether you bought the domain
from one registrar or another, and whether you publish records by hand or via a
provider adapter.

## Why the separation matters

Keeping purchase out of attachment means:

- no lock-in to a particular registrar,
- the attachment lifecycle stays simple and auditable,
- the security model (ownership proof, tombstones, fail-closed routing) does not
  depend on any billing relationship.

See `reference/state-machine.md` for the attachment lifecycle and
`reference/security-model.md` for the guarantees it provides.
