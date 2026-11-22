# dns-manager

A DNS record manager scaffold built on the `DnsProvider` port from
[ag-domains](../../../crates/ag-domains). All CRUD operations go through the
provider trait, so the bundled native adapter can be replaced by a real DNS
provider adapter without changing the HTTP layer.

## Files

```
Cargo.toml           package manifest (ag-domains pinned by rev)
config.toml          server and provider settings
src/main.rs          in-memory DnsProvider adapter and CRUD routes
src/mail_records.rs  SPF/DKIM/DMARC presets via ag-domains
template.toml        catalog metadata
```

## Run

```sh
ag new mydns --template dns-manager
cd mydns
ag dev
curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/mail-presets
curl -X POST http://127.0.0.1:8080/zones/example.com/records \
  -H 'content-type: application/json' \
  -d '{"name":"api.example.com","record_type":"A","content":"203.0.113.10","ttl":300,"proxied":false}'
curl http://127.0.0.1:8080/zones/example.com/records
```

Without the CLI, replace `{{name}}` in the source files and run `cargo run`.
`BIND` overrides the listen address (default `127.0.0.1:8080`).

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/health` | Liveness probe. |
| `GET` | `/mail-presets` | Generated SPF/DKIM/DMARC record specs. |
| `GET` | `/zones/:zone/records` | List records in a zone. |
| `POST` | `/zones/:zone/records` | Create a record from a `DnsRecordSpec`. |
| `PUT` | `/zones/:zone/records/:id` | Replace a record. |
| `DELETE` | `/zones/:zone/records/:id` | Delete a record. |

## External requirements

- `AG_CLOUDFLARE_TOKEN` (catalog requirement): the token needed when the
  production deployment switches to ag-domains' adapter feature for that DNS
  provider. It is not required to build or run this scaffold.
- `MAIL_DOMAIN` (optional): domain used for the mail record presets.

## Limitations

- The bundled adapter is in-memory and non-durable: records are lost on
  restart. It exists so the template is fully offline and credential-free.
- This is a DNS zone record manager, not a domain registrar. No certificates
  are issued by this scaffold.
- `mail-presets` does not include a DKIM public key; no key is bundled.
- Stability: scaffold / alpha. Pin the `ag-*` revision before production use.
