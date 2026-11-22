# {{name}} - webhook gateway

A minimal inbound webhook receiver. Its only security gate is a constant-time
HMAC-SHA256 check over the **raw** request body, so the service can safely
accept payloads from third-party senders before parsing them.

## Stack

| Concern | Choice |
| --- | --- |
| HTTP and security | `ag-core` (`Shield`), axum 0.7 |
| Signature | `hmac` 0.12 + `sha2` 0.10, compared with `Mac::verify_slice` |
| Encoding | `hex` 0.4 for the `sha256=<hex>` header format |
| Storage | in-memory event log with delivery-id deduplication |

## Run

```bash
export WEBHOOK_SECRET='a-long-random-value'
cargo run
```

Send a signed event (the signature is computed over the exact bytes of the
body, with no whitespace normalization):

```bash
BODY='{"event":"ping"}'
SIG="sha256=$(printf '%s' "$BODY" | openssl dgst -sha256 -hmac "$WEBHOOK_SECRET" -hex | awk '{print $NF}')"

curl -i -X POST http://127.0.0.1:8080/webhook \
  -H 'content-type: application/json' \
  -H "X-Signature-256: $SIG" \
  -H 'X-Delivery-Id: delivery-1' \
  --data-binary "$BODY"

curl http://127.0.0.1:8080/events
```

A missing, malformed or invalid signature returns 401 and the body is never
parsed. Reusing the same `X-Delivery-Id` returns 202 with `duplicate: true`
and does not append a second event.

## API

| Method | Path | Purpose |
| --- | --- | --- |
| POST | `/webhook` | Verify signature, deduplicate and record an event |
| GET | `/events` | List the most recent events, newest first |
| GET | `/health` | Liveness and algorithm identifier |

## Verification tests

```bash
cargo test --offline
```

Unit tests cover a valid signature, a tampered body and malformed headers.

## Limitations

- A single shared secret is used for all senders; add per-provider secrets and
  a provider identifier in the path for production.
- No timestamp or nonce window: capture `X-Timestamp` in the signed payload if
  you need replay protection beyond delivery-id dedup.
- The event log is capped at 1000 entries and is lost on restart. A real
  gateway should forward verified events to a queue (`ag-workers`) or a durable
  store (`ag-data`).
- The dedup set grows without bound in this demo; use a TTL cache in production.
- Fail fast on startup if `WEBHOOK_SECRET` is absent; do not fall back to a
  default secret.
