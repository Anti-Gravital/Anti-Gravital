# Evidence — local ACME mock order flow (offline, no CA)

This document records a real, reproducible run of the local ACME mock harness
in `scripts/acme-mock-harness.py`. It exercises the same RFC 8555 order states
as the credentialed staging run of issue #87: new order, authorization
pending/valid, DNS-01 challenge, finalize and certificate download.

**Scope statement.** This is local tooling. No external network is used, no
domain control is proven, and no certification authority is contacted. The
downloaded chain is a self-signed certificate generated on this machine by
`openssl`; its subject equals its issuer. It is not a CA-issued certificate and
must never be reported as such. Nothing here closes #87 or #179: both remain
open with `blocked` until the credentialed staging run described in
`how-to/acme-staging-test.md` passes before and after the 0.8 port.

## Environment

| Field | Value |
| --- | --- |
| Base checkout | `7cbc28a35a5b0ceac033fbab4aefb5736d00fd99` |
| Harness | tip of branch `community9/dnsacme` (untracked files at run time, committed on that branch afterwards) |
| Date (UTC) | 2026-11-22 12:50 |
| Python | 3.14.4 (standard library only) |
| OpenSSL | OpenSSL 3.5.5 27 Jan 2026 |
| rustc (informational; harness is Python) | rustc 1.99.1 (58cf8f9c3 2026-10-13) |
| OS | Linux x86_64, kernel 7.0.0-44-generic |
| Bind address | `127.0.0.1`, ephemeral port, plain HTTP (no TLS) |
| Domain | `mock.example.test` (RFC 2606 reserved `.test` TLD) |
| External services contacted | none |

## Command

```
cd <repo-root>
bash -n scripts/acme-mock-harness.sh
python3 -m py_compile scripts/acme-mock-harness.py
scripts/acme-mock-harness.sh /tmp/acme-mock-run.log
```

The wrapper requires `python3` and `openssl` on `PATH` and writes the
JSON-line transcript both to stdout and to the optional log path. The harness
exits 0 on `PASS`, 1 on a failed assertion and 2 on a missing tool.

## Real output (verbatim, first of three runs)

```
{"at": "2026-11-22T12:50:01.548966+00:00", "domain": "mock.example.test", "event": "harness:start", "kind": "local-mock-only", "note": "No real CA, no credentials, no external network.", "openssl": "OpenSSL 3.5.5 27 Jan 2026 (Library: OpenSSL 3.5.5 27 Jan 2026)", "python": "3.14.4"}
{"at": "2026-11-22T12:50:01.946059+00:00", "bind": "127.0.0.1", "event": "server:ready", "tls": false, "url": "http://127.0.0.1:38539"}
{"at": "2026-11-22T12:50:01.984945+00:00", "event": "directory:ok", "new_account": true, "new_order": true}
{"at": "2026-11-22T12:50:01.988920+00:00", "event": "account:valid", "status": "valid"}
{"at": "2026-11-22T12:50:01.996677+00:00", "domain": "mock.example.test", "event": "new_order:pending", "status": "pending"}
{"at": "2026-11-22T12:50:02.003176+00:00", "challenge_status": "pending", "challenge_type": "dns-01", "event": "authorization:pending", "status": "pending"}
{"at": "2026-11-22T12:50:02.008530+00:00", "event": "challenge:valid", "status": "valid"}
{"at": "2026-11-22T12:50:02.012899+00:00", "event": "authorization:valid", "status": "valid"}
{"at": "2026-11-22T12:50:02.017852+00:00", "event": "order:ready", "status": "ready"}
{"at": "2026-11-22T12:50:02.023040+00:00", "event": "order:processing", "status": "processing"}
{"at": "2026-11-22T12:50:02.029454+00:00", "certificate_url": "http://127.0.0.1:38539/certificate/order-1", "event": "order:valid", "status": "valid"}
{"at": "2026-11-22T12:50:02.107226+00:00", "event": "certificate:downloaded", "openssl_x509": "subject=CN=mock.example.test\nissuer=CN=mock.example.test\nserial=3715BDB2FAD4BB92F6B3BF7A2A3F5E0F2CD8E0EB\nsha256 Fingerprint=17:98:AD:1C:3C:38:63:3F:93:F4:F3:CE:17:91:DA:4C:16:1F:E0:13:80:9E:E4:AA:2A:FC:F5:7E:B0:9D:B3:DA", "pem_header": "-----BEGIN CERTIFICATE-----", "self_signed": true, "subject_equals_issuer": true}
{"at": "2026-11-22T12:50:02.107695+00:00", "event": "harness:result", "result": "PASS", "sequence": ["new_order:pending", "authorization:pending", "challenge:pending", "challenge:valid", "authorization:valid", "order:ready", "order:processing", "order:valid"], "statement": "Local mock flow only. No real CA issued a certificate; the chain was self-signed on this machine."}
```

Exit code: `0`.

## State sequence asserted

The harness asserts the exact order of state records emitted by the mock, not
just the final status:

1. `new_order:pending` - order created, DNS identifier accepted.
2. `authorization:pending` - one authorization for the identifier.
3. `challenge:pending` - a `dns-01` challenge is offered.
4. `challenge:valid` - readiness signalled, mock accepts the challenge.
5. `authorization:valid` - authorization flips after the challenge.
6. `order:ready` - order ready for finalize.
7. `order:processing` - CSR accepted by the mock finalize endpoint.
8. `order:valid` - refresh publishes the certificate URL.
9. Certificate downloaded as a PEM chain and inspected with `openssl x509`.

The inspection shows `subject=CN=mock.example.test` and
`issuer=CN=mock.example.test`, confirming the chain is self-signed by the
harness. No key material is logged.

## Reproducibility

Three consecutive runs on the same base checkout with the harness files present
produced 13 transcript lines each and ended with `result: PASS` and the
identical eight-state sequence. A fourth run on the committed tree also exited
0. The ephemeral port, timestamps, serial and fingerprint differ per run by
design.

| Run | Exit | Result line |
| --- | --- | --- |
| 1 | 0 | PASS |
| 2 | 0 | PASS |
| 3 | 0 | PASS |

## What this proves and what it does not

Proves: the local harness can drive the documented order states deterministically
and offline; the PEM download parses and is self-signed; the client driver reads
the authorization/challenge/order/finalize/certificate links from the directory
and order objects instead of hardcoding them. It does not execute any Rust code
from `crates/ag-domains`, so it says nothing about the crate's own tests.

Does not prove: that a trusted CA validated a domain, that DNS propagation
works from an external resolver, that the edge serves a CA-issued chain by SNI,
or that the `instant-acme` 0.7 to 0.8 port is correct. Those require the
credentialed run in `how-to/acme-staging-test.md`.

## Exact condition to close #87 and #179

- #87 closes only when the `#[ignore]` target
  `tests/acme_staging.rs::staging_dns01_issue_and_serve_sni` exists and passes
  against the Let's Encrypt **staging** directory, with the evidence template of
  `how-to/acme-staging-test.md` filled from a real run (before and after any
  dependency change).
- #179 closes only when that staging evidence exists for the 0.8 code and
  `cargo test -p ag-domains --all-features` passes with `instant-acme` 0.8.
  The dependency stays at 0.7 until then.

Until those conditions hold, this document is supporting local evidence only
and both issues keep the `blocked` label.
