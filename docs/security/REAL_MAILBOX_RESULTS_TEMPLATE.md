# Real-mailbox delivery results template (issue #178)

Copy this file to the path named by `AG_MAIL_DELIVERY_RESULTS` (default
`docs/security/REAL_MAILBOX_RESULTS.md`) and fill every field from the raw
headers of the delivered message. Empty fields are intentional: they mean
"not recorded" and are never evidence. Do not infer a value. See
`docs/security/REAL_MAILBOX_VERIFICATION.md` for the procedure and the field
definitions.

Unfold folded header lines by joining continuation lines with a single space
before writing them to the one-line fields below.

## Delivery metadata

delivery.date_utc=
delivery.commit_sha=
delivery.recipient_address=
delivery.recipient_domain=
delivery.signing_domain=
delivery.selector=
delivery.helo_host=
delivery.mta_feature=mta
delivery.notes=

## Receiver verdict from Authentication-Results

auth_results.receiver=
auth_results.raw=
auth_results.dkim_result=
auth_results.header_d=
auth_results.header_s=
auth_results.header_a=
auth_results.header_b=

## Tags from the DKIM-Signature header of the delivered message

dkim_signature.raw=
dkim_signature.d=
dkim_signature.s=
dkim_signature.a=
dkim_signature.b=

## Evidence statement

evidence.source=receiver Authentication-Results header
evidence.local_verifier_used=false
evidence.gate_claimed=false

This template is unfilled by design. A checker run against it reports SKIP with
reason `no-evidence`, never PASS.
