# How-to — connect a domain at common DNS providers

The flow is the same everywhere (manual, no credentials needed):

```
ag domains attach <domain> --project <p> --edge-host <edge> [--ip <A/AAAA>]...
ag domains instructions <domain> --edge-host <edge> [--ip <ip>]...
# paste the printed records at your provider, then:
ag domains verify <domain>
```

`ag domains instructions` prints three kinds of record: routing (`A`/`AAAA` for
an apex, `CNAME` for a subdomain/wildcard), the recommended `www` CNAME for an
apex, and the `_ag-domain` TXT ownership record. Below is where each provider
exposes DNS records. Capability facts: `reference/provider-capability-matrix.md`.

> Apex note: a plain CNAME is invalid at the apex. Use the printed `A`/`AAAA`
> records, or the provider's apex alias / CNAME flattening when available
> (Cloudflare, Route 53, Azure).

## Namecheap

Domain List -> Manage -> **Advanced DNS** -> Host Records. Add records with
`Host` = the record name (`@` for apex, `www`, `_ag-domain`, or the subdomain
label) and the printed value. Set TTL to Automatic or 5 min. If Namecheap
BasicDNS is not active, switch nameservers to Namecheap BasicDNS first.

## Hostinger

hPanel -> Domains -> **DNS / Nameservers** -> DNS records. Add A/AAAA/CNAME/TXT
rows; `Name` `@` is the apex. Hostinger has no apex alias, so use the printed
`A`/`AAAA` for the apex.

## Squarespace (third-party domain)

Use Squarespace only as the DNS host: Settings -> Domains -> your domain -> **DNS
Settings** -> Custom Records. Add the printed records. Squarespace shows a
detected/required state; rely on `ag domains diagnose`/`verify` as the source of
truth.

## Cloudflare

DNS -> Records. Cloudflare supports **CNAME flattening at the apex**, so you may
add a CNAME at `@` pointing to the edge host instead of A/AAAA. Keep the proxy
(orange cloud) **off** while issuing the first certificate (HTTP-01 needs the
origin reachable). Cloudflare also has a first-party adapter
(`--features cloudflare`) for `ag domains sync`.

## Amazon Route 53

Hosted zones -> your zone -> Create record. For the apex use an **Alias** record
to the edge host (Route 53 alias), or the printed A/AAAA. Add the `www` CNAME and
the `_ag-domain` TXT. Changes apply as a change batch (a few seconds).

## Google Cloud DNS

Cloud DNS -> your zone -> Add record set. Add A/AAAA at the apex (no apex alias),
the `www` CNAME and the `_ag-domain` TXT. You can also import a zone file; export
one with `ag domains export-zone <domain> --edge-host <edge> --ip <ip>`.

## Azure DNS

DNS zone -> Record sets. Azure supports an **alias record** at the apex pointing
to the edge, or the printed A/AAAA. Add the `www` CNAME and the `_ag-domain` TXT.

## After pasting

```
ag domains verify <domain>     # confirms the TXT ownership record
ag domains diagnose <domain>   # expected vs observed (when available)
ag domains status <domain>     # readiness dimensions
```
