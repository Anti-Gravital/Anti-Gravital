# Real-mailbox delivery verification for `ag-mail` (issue #178)

Status: procedure and opt-in checker provided; the real-mailbox gate is NOT
closed by this document or by any offline test. This is the reproducible
hand-off for the second Verification item of issue #178. The independent
offline DKIM verifier is covered separately in
`docs/security/DKIM_INDEPENDENT_VERIFICATION.md` and does not replace this gate.

## 1. What closes issue #178

Both items below are required before posting `Closes #178`:

1. The native-MTA test tooling and this verification kit are merged and default
   CI stays green.
2. A maintainer with port 25 egress delivers a message signed by `ag-mail`
   through `MtaSender` (feature `mta`) to a **real mailbox they control** and
   records the receiving MTA's `Authentication-Results` header together with
   the date, the producing commit and the sending identity.

Item 2 is external and manual. It cannot be produced by `dkim.verify()`, by
`tools/dkim_verify.py`, by the in-process loopback sink or by a local TXT
answer. A receiver that adds the header is the only valid source.

Until item 2 is recorded, issue #178 stays open with `Refs #178`.

## 2. The exact fields to capture

A receiving MTA adds an `Authentication-Results` header (RFC 7001). For a DKIM
result it reports `dkim=pass` and the following properties. They map to tags in
the `DKIM-Signature` header as follows:

| Capture key | Where it comes from | Meaning | Required |
| --- | --- | --- | --- |
| `auth_results.dkim_result` | `Authentication-Results`: `dkim=` | Receiver verdict; must be `pass` | yes |
| `auth_results.header_d` | `Authentication-Results`: `header.d=` (from `d=`) | Signing domain, must equal the configured signing domain | yes |
| `auth_results.header_s` | `Authentication-Results`: `header.s=` (from `s=`) | Selector, must equal the configured selector | yes |
| `auth_results.header_a` | `Authentication-Results`: `header.a=` (from `a=`) | Algorithm, expected `ed25519-sha256` or `rsa-sha256` | yes |
| `auth_results.header_b` | `Authentication-Results`: `header.b=` (truncated `b=`) | Correlation snippet emitted by some receivers | if present |
| `dkim_signature.d` | `DKIM-Signature`: `d=` | Full signing domain from the message | yes |
| `dkim_signature.s` | `DKIM-Signature`: `s=` | Full selector from the message | yes |
| `dkim_signature.a` | `DKIM-Signature`: `a=` | Full algorithm from the message | yes |
| `dkim_signature.b` | `DKIM-Signature`: `b=` | Full signature value used to verify | yes |

Rules for copying the evidence:

- Use the mailbox client's "show original" / "view raw headers" action. A
  summarized "DKIM: OK" badge is not evidence.
- Header lines are often folded. Unfold continuation lines by joining them with
  a single space before writing them to the one-line fields in the template.
- `header.b` is only a correlation snippet; never use it as the full signature.
  Take `dkim_signature.b` from the `DKIM-Signature` header itself.
- Never fill an empty field by inference. If the receiver omits `header.a` or
  `header.b`, leave that field empty and say so in the notes field.
- The `Authentication-Results` header must name a receiving MTA, not this
  repository or a local process.

Copy `docs/security/REAL_MAILBOX_RESULTS_TEMPLATE.md` to the path named by
`AG_MAIL_DELIVERY_RESULTS` (default `docs/security/REAL_MAILBOX_RESULTS.md`) and
fill it. Keep the filled file with the PR or issue comment as the evidence.

## 3. Environment variables used by the procedure and checker

| Variable | Used for | Requirement |
| --- | --- | --- |
| `AG_MAIL_DELIVERY_OPT_IN` | Explicit opt-in for `scripts/check-mail-delivery.sh` | must be `1`; otherwise the checker SKIPs |
| `AG_MAIL_DELIVERY_SCOPE` | `wire` (local sink) or `real-mailbox` (default) | set `wire` only for Procedure A |
| `AG_MAIL_DELIVERY_RESULTS` | Path to the filled results file | optional; defaults to `docs/security/REAL_MAILBOX_RESULTS.md` |
| `AG_MAIL_DELIVERY_TO` | Recipient mailbox address used for the live send | required for `real-mailbox` scope |
| `AG_MAIL_DELIVERY_DOMAIN` | Signing domain (`d=`) used for the live send | required for `real-mailbox` scope |
| `AG_MAIL_DELIVERY_SELECTOR` | Selector (`s=`) used for the live send | required for `real-mailbox` scope |
| `AG_MAIL_DELIVERY_PROBE_TARGET` | `host:port` of a recipient MX for the port 25 egress probe | required for `real-mailbox` scope; no default |
| `AG_MAIL_SINK_HOST`, `AG_MAIL_SINK_PORT` | Loopback sink address for `wire` scope | defaults `127.0.0.1`, `1025` |

The checker is opt-in so it never turns an unconfigured environment into a
verification claim. A missing dependency, a blocked port or absent evidence is
always reported as SKIP, never PASS.

## 4. Procedure A: local sink (offline wire check)

This proves the ESMTP/STARTTLS dialogue and that a DKIM signature is present in
the bytes submitted to a sink. It does **not** produce an
`Authentication-Results` header and does not close the gate.

1. Confirm the toolchain and feature build:

   ```sh
   cargo test --locked -p ag-mail --features mta --lib -- --list | grep -E 'submit_|dkim_'
   ```

2. Run the two loopback-sink tests. They start an in-process SMTP sink that
   advertises STARTTLS with a throwaway certificate, accept the connection with
   the opportunistic-TLS configuration and capture the `DATA` payload:

   ```sh
   CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
   CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
   cargo test --locked -p ag-mail --features mta --lib \
     sender::mta::tests::submit_speaks_esmtp_over_starttls_to_local_sink -- --exact --nocapture
   ```

   ```sh
   CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
   CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
   cargo test --locked -p ag-mail --features mta --lib \
     sender::mta::tests::submit_signs_with_dkim_on_the_wire -- --exact --nocapture
   ```

3. Optionally capture the signed bytes into a file and cross-check them with
   the independent verifier. The verifier answers the `_domainkey` query from a
   fixed local record, so this remains offline:

   ```sh
   python3 tools/dkim_verify.py /path/to/captured-message.eml
   ```

4. Run the checker in wire scope to record the outcome:

   ```sh
   AG_MAIL_DELIVERY_OPT_IN=1 AG_MAIL_DELIVERY_SCOPE=wire \
   AG_MAIL_DELIVERY_RESULTS=docs/security/REAL_MAILBOX_RESULTS.md \
   bash scripts/check-mail-delivery.sh; echo "exit=$?"
   ```

   A `RESULT: PASS scope=wire` line means only that the local sink check
   succeeded. The checker always prints `gate.real_mailbox=NOT_SATISFIED` in
   wire scope. Only `scope=real-mailbox` PASS is gate evidence.

## 5. Procedure B: real mailbox (the external gate)

1. Pick a mailbox you control at a domain whose MX you can reach on port 25.
   Confirm egress (the checker performs the same probe):

   ```sh
   AG_MAIL_DELIVERY_PROBE_TARGET=mx.recipient.example:25 \
   timeout 5 bash -c 'exec 3<>/dev/tcp/${0%:*}/${0##*:}' \
     "$AG_MAIL_DELIVERY_PROBE_TARGET" && echo "port25=reachable"
   ```

   If this fails or times out, the gate is blocked in this environment. Record
   that fact and stop; do not report PASS.

2. Publish and confirm the DKIM key. The TXT record at
   `selector._domainkey.signing.example` must contain the public key matching
   the `DkimConfig` used for signing, with `k=ed25519` (or `k=rsa` for the RSA
   path). Confirm with your DNS tool of choice before sending.

3. Send through the native MTA using the real resolver and port 25. The relevant
   API is `MtaSender` plus `MtaConfig::with_dkim(DkimConfig::...)`; keep the
   signing domain aligned with the `From` domain for DMARC. A minimal call site
   has this shape (replace the placeholders with your own key and identities):

   ```rust,ignore
   use ag_mail::{message::{Address, EmailBuilder}, sender::MailSender};
   use ag_mail::sender::mta::{DkimConfig, MtaConfig, MtaSender};

   let dkim = DkimConfig::ed25519("signing.example", "sel", private_pkcs8_der);
   let config = MtaConfig::new("mail.signing.example")
       .with_dkim(dkim);
       // port 25 and the system resolver are the defaults
   let sender = MtaSender::new(config);
   let email = EmailBuilder::new()
       .from(Address::new("noreply@signing.example"))
       .to(Address::new("recipient@recipient.example"))
       .subject("ag-mail native MTA delivery check")
       .text_body("Delivery check for issue #178")
       .build()
       .expect("valid email");
   let result = sender.send(&email).await.expect("delivery accepted by MX");
   ```

   Record the producing `git rev-parse HEAD` before the send.

4. In the destination mailbox, open the message and view the raw headers. Copy
   the full `Authentication-Results` and `DKIM-Signature` headers into the
   template from section 2. If the mailbox shows no `Authentication-Results`
   header, the gate is not satisfied: record that and stop.

5. Run the checker with all required variables set:

   ```sh
   AG_MAIL_DELIVERY_OPT_IN=1 \
   AG_MAIL_DELIVERY_SCOPE=real-mailbox \
   AG_MAIL_DELIVERY_RESULTS=docs/security/REAL_MAILBOX_RESULTS.md \
   AG_MAIL_DELIVERY_TO=recipient@recipient.example \
   AG_MAIL_DELIVERY_DOMAIN=signing.example \
   AG_MAIL_DELIVERY_SELECTOR=sel \
   AG_MAIL_DELIVERY_PROBE_TARGET=mx.recipient.example:25 \
   bash scripts/check-mail-delivery.sh; echo "exit=$?"
   ```

6. Require a `RESULT: PASS scope=real-mailbox` line and attach the filled
   results file plus the checker output to the PR or issue comment. Only after
   the change is merged, default CI is green and this recorded evidence exists
   may a maintainer post `Closes #178`.

## 6. Checker outcomes

| Outcome | Exit | Condition |
| --- | --- | --- |
| `PASS scope=wire` | 0 | Opt-in, sink reachable, results file shows a valid DKIM signature tags; prints `gate.real_mailbox=NOT_SATISFIED` |
| `PASS scope=real-mailbox` | 0 | Opt-in, identities set, port 25 probe reachable, filled results file with `dkim=pass`, matching `d=`/`s=`/`a=` and a non-empty `b=` |
| `SKIP` | 0 | Not opted in, sink or port 25 unavailable, identities absent, probe target not configured, or results file missing/unfilled |
| `FAIL` | 1 | Evidence present but inconsistent: `dkim=fail`/`dkim=error`, missing tags, or a mismatch between the receiver fields and the configured identities or the DKIM-Signature tags |

SKIP and FAIL are different: SKIP means the check could not be performed, FAIL
means it was performed and found a real defect. Read the `RESULT:` line; exit 0
alone never means a verified delivery.

## 7. References

- Issue #178: mail protocol stack upgrade and real-mailbox verification.
- `docs/security/DKIM_INDEPENDENT_VERIFICATION.md`: offline independent verifier.
- `docs/audits/dkim-independent-ci.md`: recorded offline evidence and gate
  closure condition.
- RFC-0009 sections 4.3 and 4.8: DKIM signing and the local-sink smoke test.
- RFC-0035: authorization for the opt-in DKIM CI check.
