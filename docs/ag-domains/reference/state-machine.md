# Reference — attachment state machine

An attachment tracks four independent readiness dimensions plus a derived
lifecycle. A single boolean is deliberately avoided (RFC-0009 §6).

## Readiness dimensions

- `ownership_status`: `pending | verified | failed | expired`
- `dns_status`: `pending | routable | wrong_target | conflicting_records | unknown | failed`
- `tls_status`: `disabled | pending | active | renewal_due | failed | expired | retired`
- `routing_status`: `disabled | shadow | ready | active | suspended`

## Lifecycle (derived)

```
draft
  -> pending_ownership
  -> pending_dns
  -> pending_tls
  -> active

misconfigured   (any failure/conflict dimension)
detached        (after detach; terminal until re-attach)
```

The lifecycle is recomputed from the dimensions by
`DomainAttachment::recompute_lifecycle`. `detached` is terminal: it is not
recomputed away.

## Activation rule

```
active = ownership_verified
      && dns_routable
      && (tls_active || tls_mode == disabled)
      && routing_ready
```

A hostname must not serve traffic until activation holds.

## Tombstones

Detaching writes a tombstone (default 30 days) that blocks re-claiming the
hostname until it expires (subdomain-takeover prevention, RFC-0009 §7).
