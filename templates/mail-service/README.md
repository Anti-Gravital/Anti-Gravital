# mail-service

A transactional mail service scaffold built on
[ag-mail](../../../crates/ag-mail). It exposes a small HTTP surface that
validates and sends an email through the `MailSender` port. The native SMTP
relay is used when configured; otherwise the service runs in an offline
dry-run mode.

## Files

```
Cargo.toml      package manifest (ag-mail pinned by rev)
config.toml     documented server, SMTP and DKIM settings
src/main.rs     composition root: sender selection and HTTP handlers
template.toml   catalog metadata
```

## Run

```sh
ag new mymail --template mail-service
cd mymail
ag dev
curl http://127.0.0.1:8080/health
curl -X POST http://127.0.0.1:8080/send \
  -H 'content-type: application/json' \
  -d '{"to":"user@example.com","subject":"Welcome","body":"Hello"}'
```

Without the CLI, replace `{{name}}` in the source files and run Cargo directly:

```sh
sed -i 's/{{name}}/mymail/g' Cargo.toml src/main.rs
cargo run
```

`BIND` overrides the listen address (default `127.0.0.1:8080`).

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `AG_SMTP_HOST` | unset | SMTP relay host. Unset means dry-run mode. |
| `AG_SMTP_PORT` | `587` | SMTP relay port. |
| `AG_SMTP_USERNAME` / `AG_SMTP_PASSWORD` | unset | Relay credentials. Both must be set to authenticate. |
| `AG_MAIL_FROM` | `no-reply@localhost` | Default `From` address. |
| `AG_DKIM_SELECTOR` / `AG_DKIM_DOMAIN` / `AG_DKIM_KEY_PATH` | unset | DKIM configuration stub. |

## External requirements

- `AG_SMTP_HOST` (catalog requirement): a reachable SMTP relay for real
  delivery. Credentials are supplied by the deployment; none are bundled.
- DKIM signing requires a private key and the opt-in `mta` feature of
  `ag-mail`. This template only loads and logs the DKIM settings as a stub.

## Limitations

- Outbound only. This is not an IMAP/POP mail server and has no inbound
  routing.
- Dry-run mode never performs network I/O, so the template builds and starts
  with no external service. Real delivery needs the SMTP relay above.
- The DKIM block is configuration only; it does not sign messages.
- Stability: scaffold / alpha. Pin the `ag-*` revision before production use.
