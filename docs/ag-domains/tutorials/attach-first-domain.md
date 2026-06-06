# Tutorial — attach your first custom domain

Goal: attach a domain you bought from any registrar to an Anti-Gravital project
and produce the exact DNS records to publish. No provider credentials needed.

## 1. Attach the domain

```
ag domains attach example.com \
  --project my-site \
  --edge-host edge.my-cloud.example \
  --ip 203.0.113.10 --ip 2001:db8::10
```

Output lists the records to add at your DNS provider, including the TXT
ownership record and a `www` CNAME.

## 2. Publish the records

Copy each row into your DNS provider's dashboard (Namecheap, Hostinger,
Cloudflare, Route 53, etc.). For an apex domain, add the `A`/`AAAA` records; for
a subdomain, add the single `CNAME`.

## 3. Verify ownership

```
ag domains verify example.com
```

This checks the TXT record across public resolvers. DNS can take minutes to
propagate; re-run until it reports verified.

## 4. Check status

```
ag domains status example.com
```

You will see the four readiness dimensions. Ownership becomes `verified` after
step 3.

## 5. Re-print or export records

```
ag domains instructions example.com --edge-host edge.my-cloud.example --ip 203.0.113.10
ag domains export-zone   example.com --edge-host edge.my-cloud.example --ip 203.0.113.10
```

## Detaching

```
ag domains detach example.com
```

This stops renewal and tombstones the hostname to prevent takeover. Remember to
remove the records from your provider.

## What comes next

Native HTTPS serving (live edge listeners + certificate issuance wired end to
end) is a later phase (RFC-0009 phase B). Today this tutorial covers attachment,
instructions, ownership proof and status.
