# How-to — troubleshoot DNS and certificates

The fastest path is `ag domains diagnose`, which compares what the attachment
expects against what public resolvers actually return:

```
ag domains diagnose <domain> --edge-host <edge> [--ip <ip>]...
```

Each line is `[ok]` or `[error]` with an action. "No action required" means the
DNS side is correct.

## DNS propagation

Symptom: `ag domains verify` stays `PENDING`, or `diagnose` reports a record as
missing right after you added it.

- DNS changes take time to propagate. Re-run `verify` / `diagnose` after a few
  minutes (low TTLs help).
- Confirm the exact ownership value at the registered domain:
  `ag domains check --domain _ag-domain.<registered-domain> --expected <value>`.
- "wrong value" on a CNAME usually means an old hosting record is still present
  (`Observed: ... old-host`). Replace the value at your provider.
- "CNAME coexists with A/AAAA": remove one — a name cannot have both.
- Apex domains: a plain CNAME is invalid at the apex; use the printed `A`/`AAAA`
  or your provider's apex alias / CNAME flattening.

## Certificate issuance

Symptom: ownership and DNS are correct but the certificate does not issue.

- CAA: if the domain publishes CAA records that exclude the configured CA,
  issuance is refused. Add/adjust a CAA record allowing the CA, or configure an
  allowed CA. (The control plane runs a CAA preflight before ordering.)
- HTTP-01 (exact hostnames): the domain must resolve to the edge and reach it on
  port 80 so the CA can fetch `/.well-known/acme-challenge/<token>`. Behind a
  proxy (e.g. Cloudflare orange-cloud), disable it for the first issuance.
- DNS-01 (wildcards): a `_acme-challenge.<base>` TXT record must be published.
  With the manual flow you add it when prompted; with a provider adapter it is
  created and cleaned up automatically.
- Rate limits: the issuance layer deduplicates by SAN set and counts per
  registered domain to stay under the CA's limits; if you hit a CA limit, wait
  for the window to reset rather than retrying in a loop.

## Takeover protection

Re-attaching a recently detached hostname is blocked while its tombstone is
active. Wait for it to expire or use a different host; reactivation requires a
fresh ownership proof.
