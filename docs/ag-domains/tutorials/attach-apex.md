# Tutorial — attach an apex domain

Goal: attach an apex (root) domain such as `example.com` to an Anti-Gravital
project. Apex domains cannot use a CNAME, so they point at the edge with
`A`/`AAAA` records (or provider CNAME-flattening / ALIAS).

## 1. Attach

```
ag domains attach example.com \
  --project my-site \
  --edge-host edge.my-cloud.example \
  --ip 203.0.113.10 --ip 2001:db8::10
```

The output lists the records to publish: the apex `A`/`AAAA` records, a
recommended `www` `CNAME`, and the `_ag-domain` TXT ownership record.

## 2. Publish the records

Add the `A` (and `AAAA`, if you use IPv6) records at the apex, plus the `www`
`CNAME`. If your provider offers CNAME flattening or ALIAS at the apex, you may
use it instead of `A`/`AAAA`.

## 3. Verify ownership

```
ag domains verify example.com
```

This checks the `_ag-domain` TXT record across public resolvers. Re-run until it
reports verified (DNS can take minutes to propagate).

## 4. Status and diagnosis

```
ag domains status example.com
ag domains diagnose example.com --edge-host edge.my-cloud.example --ip 203.0.113.10
```

`status` shows the four readiness dimensions; `diagnose` compares expected vs
observed records and points out missing or wrong ones.

## Notes

- No apex IPs configured means the attach output emits a note: apex domains need
  `A`/`AAAA` (or flattening/ALIAS) to route.
- TLS for an exact apex hostname uses the ACME HTTP-01 challenge
  (`explanation/http01-vs-dns01.md`).

See `tutorials/attach-subdomain.md` for the subdomain (CNAME) case.
