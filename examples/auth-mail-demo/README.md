# auth-mail-demo

Demonstrates how to connect `ag-auth` with `ag-mail` for the three main
transactional flows:

1. Sign-up with email verification.
2. Password recovery.
3. Passwordless authentication (magic link).

It is a one-shot program: it prints the result of each flow and exits. It opens
no ports and needs no external services.

## External requirements

None. The example uses `NullSender`, which captures the emails in memory
instead of sending them over SMTP.

## Running

```bash
cargo run -p auth-mail-demo
```

Expected output (the tokens are random on each run):

```
ag-auth + ag-mail connected.

--- Flow 1: email verification ---
...
Demo complete: 3 emails sent without needing real SMTP.
```

## Using real SMTP

Replace `NullSender` with `SmtpSender` in `src/main.rs`:

```rust
use ag_mail::sender::smtp::{SmtpConfig, SmtpSender};
let config = SmtpConfig::new("smtp.example.com", 587, "user", "pass");
let sender = SmtpSender::new(config)?;
```

## Crates demonstrated

- `ag-auth`: token generation and orchestration of the auth flows.
- `ag-mail`: transactional sending (feature `smtp`) and `NullSender`
  (feature `test-utils`) for testing without SMTP.
