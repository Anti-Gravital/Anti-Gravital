# workers-mail-integration

Routing `ag-mail` delivery through the shared `ag-workers` background engine
(RFC-0012 S7/M2 adapter). Only the generic queue mechanics — enqueue,
retry/backoff, dead-lettering — move to the `ag-workers` substrate; the
mail-specific delivery path stays behind `ag-mail`'s `MailSender`. The example
uses a no-op sender, so no SMTP server or external service is required
(native-first, ADR-0009).

## Run

```bash
cargo run -p workers-mail-integration
```

Expected output:

```text
delivering mail to user0@example.com
delivering mail to user1@example.com
delivering mail to user2@example.com
done: mail drained through the ag-workers substrate
```

## What it shows

- `ag_mail::workers::WorkersMailQueue` — an `ag-mail` `MailQueue` implementation
  that enqueues `Email` jobs (`kind=mail.delivery`) onto any `ag-workers`
  `QueueBackend` (producer side).
- `ag_mail::workers::mail_delivery_registry` — builds the closed worker
  registry binding a `MailSender` to the `JobHandler` contract (consumer side).
- A `WorkerPool` draining the queue and delivering through the bound sender.
- The in-memory `MemoryQueue` backend; with `ag-mail`'s `workers-postgres`
  feature the same adapter rides the durable PostgreSQL backend
  (RFC-0012 S7/M3, parity verification tracked in Issue #109).

## Decision

`RFC-0012` (S7 migration plan) / `ADR-0013`. Feature docs:
`crates/ag-mail/README.md` (features `workers` / `workers-postgres`);
module docs: `docs/modules/ag-workers/README.md`.
