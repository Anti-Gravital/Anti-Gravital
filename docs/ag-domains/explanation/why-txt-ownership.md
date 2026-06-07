# Explanation — why TXT ownership verification is required

A hostname must not start serving a tenant's content just because someone typed
it into a CLI or dashboard. Otherwise anyone could claim `example.com` they do
not control and intercept its traffic once DNS pointed at the edge.

`ag-domains` requires proof of DNS control before activation: the operator
publishes a TXT record at `_ag-domain.<registrable-domain>` containing an
unguessable per-attachment token (`ag-verification=<id>-<random>`). Only someone
who controls the domain's DNS can publish it.

## Why TXT (default)

TXT proves DNS control before any public traffic needs to move to the edge, so
ownership is established without downtime. CNAME/HTTP/provider methods are
possible alternatives but TXT is the safe default.

## Uniqueness and takeover

- The store enforces a single active attachment per hostname identity.
- Detaching writes a tombstone so another party cannot immediately re-claim the
  hostname; re-attaching requires a fresh ownership proof.

These mitigations follow the OWASP Subdomain Takeover Prevention guidance and
RFC-0011 §15-equivalent security requirements.
