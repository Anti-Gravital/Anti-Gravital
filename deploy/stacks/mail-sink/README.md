# Local mail sink

Captures outbound SMTP messages from `ag-mail` / `ag-auth` verification flows
in a local web UI. No message is relayed to a real recipient. This replaces a
third-party sandbox with a local capture endpoint; it is not an MTA.

## Ports

| Host port | Service | Purpose |
| --- | --- | --- |
| 1025 | mailpit | SMTP capture |
| 8025 | mailpit | Web UI and REST API |

Override with `MAIL_SMTP_PORT` and `MAIL_UI_PORT` in `.env`.

## Run

```sh
cd deploy/stacks/mail-sink
cp .env.example .env
docker compose up -d
```

Open `http://127.0.0.1:8025` to inspect captured messages.

## Point a generated project at the stack

The `ag mail test` command reads these variables:

```sh
export AG_SMTP_HOST='127.0.0.1'
export AG_SMTP_PORT='1025'
# AG_SMTP_USER and AG_SMTP_PASS are not needed by the default sink
ag mail test --from dev@example.test --to test@example.test
```

Messages persist in the named volume `maildata`. The container exposes its
own `readyz` health probe.

## Limits and security

The sink accepts any sender and recipient. Never point production traffic at
it. The web UI has no authentication; keep port 8025 on localhost.
