# Mail and workers data flow

How outbound email and background jobs move through `ag-mail` and
`ag-workers`. `ag-mail` sends either through the native `SmtpSender`
(external SMTP endpoint) or the native `MtaSender` (direct MX delivery,
feature `mta`). Durable job execution flows into `ag-workers`, whose
default backend is in-memory and whose durable backend is PostgreSQL via
`ag-data`.

```mermaid
graph TD
  app[Application or ag-auth] --> mail[ag-mail]
  mail --> smtp[SmtpSender]
  mail --> mta[MtaSender native MTA, feature mta]
  smtp --> ext[External SMTP endpoint]
  mta --> mx[Destination MX, ESMTP STARTTLS DKIM]
  mail --> queue[ag-mail queue with retries]
  queue --> workers[ag-workers engine]
  workers --> mem[In-memory backend default]
  workers --> pg[(PostgreSQL durable via ag-data)]
  edge[ag-edge] -. producer feature, issue 112 .-> workers
```

`ag-auth` consumes `ag-mail` for verification, recovery and magic-link
emails; `ag-mail` never depends on `ag-auth`. There are no brand-named mail
adapters (ADR-0011): external providers are reached by pointing `SmtpSender`
at their SMTP endpoint.
