# Tutorial — attach a subdomain

Goal: attach a subdomain such as `api.example.com` to an Anti-Gravital project.
Subdomains point at the edge with a single `CNAME`, which is simpler than the
apex case.

## 1. Attach

```
ag domains attach api.example.com \
  --project my-site \
  --edge-host edge.my-cloud.example
```

The output lists one routing `CNAME` (`api.example.com -> edge.my-cloud.example`)
and the `_ag-domain` TXT ownership record. No apex IPs are needed.

## 2. Publish the records

Add the `CNAME` and the TXT ownership record at your DNS provider.

## 3. Verify ownership

```
ag domains verify api.example.com
```

## 4. Status and diagnosis

```
ag domains status api.example.com
ag domains diagnose api.example.com --edge-host edge.my-cloud.example
```

## Wildcards

A wildcard such as `*.example.com` is a subdomain attachment with one
difference: TLS must use the ACME DNS-01 challenge, so the attachment is created
with `tls_mode = managed_dns01`. See `how-to/configure-wildcard.md` and
`explanation/http01-vs-dns01.md`.

## Exact vs wildcard precedence

At the edge, an exact hostname binding always wins over a wildcard, so attaching
`api.example.com` and `*.example.com` together routes `api` to its exact target.
See `explanation/routing-host-sni.md`.

See `tutorials/attach-apex.md` for the apex (`A`/`AAAA`) case.
