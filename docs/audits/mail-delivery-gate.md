# Real-mailbox delivery gate: opt-in checker evidence (#178)

Status: the checker is implemented and its SKIP and FAIL paths are recorded.
The real-mailbox gate is NOT closed. No message was sent to a real mailbox and
no receiving-MTA `Authentication-Results` header was observed. Issue #178 stays
open with `Refs #178`.

## 1. Environment of the recorded runs

| Item | Value |
| --- | --- |
| Date (UTC) | 2026-11-22 |
| Commit | `88e232937519e5bd10cb84d2ee2bb9f9022be85b` |
| Branch / worktree | `community12/i178`, `worktrees12/i178` |
| OS / arch | Linux x86_64 |
| Bash | 5.3.9(1)-release |
| Python | 3.14.4 (available but not used for delivery) |
| Real mailbox contacted | no |
| Port 25 egress | not available from this environment |

The checker adds no Cargo dependency and does not change CI. Default CI does
not call it.

## 2. Real runs in this environment

Syntax check:

```text
$ bash -n scripts/check-mail-delivery.sh
exit=0
```

Default run, no opt-in. This is the expected state of an unconfigured host:

```text
$ bash scripts/check-mail-delivery.sh
RESULT: SKIP reason=not-opted-in detail=set AG_MAIL_DELIVERY_OPT_IN=1 to enable the delivery gate check
exit=0
```

Opt-in without a probe target:

```text
$ AG_MAIL_DELIVERY_OPT_IN=1 bash scripts/check-mail-delivery.sh
tool.bash=5.3.9(1)-release
scope=real-mailbox
results.file=docs/security/REAL_MAILBOX_RESULTS.md
RESULT: SKIP reason=probe-target-not-configured detail=set AG_MAIL_DELIVERY_PROBE_TARGET=host:25
exit=0
```

Opt-in with a port 25 probe to loopback (no external egress):

```text
$ AG_MAIL_DELIVERY_OPT_IN=1 AG_MAIL_DELIVERY_PROBE_TARGET=127.0.0.1:25 bash scripts/check-mail-delivery.sh
tool.bash=5.3.9(1)-release
scope=real-mailbox
results.file=docs/security/REAL_MAILBOX_RESULTS.md
RESULT: SKIP reason=port-25-egress-unavailable detail=TCP connect to 127.0.0.1:25 failed or timed out
exit=0
```

Opt-in wire scope with no local sink listening:

```text
$ AG_MAIL_DELIVERY_OPT_IN=1 AG_MAIL_DELIVERY_SCOPE=wire bash scripts/check-mail-delivery.sh
tool.bash=5.3.9(1)-release
scope=wire
results.file=docs/security/REAL_MAILBOX_RESULTS.md
RESULT: SKIP reason=sink-unavailable detail=TCP connect to local sink 127.0.0.1:1025 failed or timed out
exit=0
```

Every real run above is SKIP, never PASS. Exit 0 for SKIP keeps an explicitly
dispatched job green without pretending that a delivery was verified.

## 3. Controlled stub checks (not evidence)

To prove that a bad verdict is rejected and that wire scope cannot be mistaken
for gate closure, the checker was run against fabricated files in `/tmp` and a
one-shot loopback TCP listener. These files are not committed and none of them
represents a real delivery.

Fabricated `dkim=fail` evidence, with the probe pointed at the local stub
listener:

```text
tool.bash=5.3.9(1)-release
scope=real-mailbox
results.file=/tmp/opencode/i178-fake-evidence.md
connect=127.0.0.1:2525 reachable
identity.to=recipient@recipient.example
identity.domain=signing.example
identity.selector=sel
RESULT: FAIL reason=receiver-rejected detail=auth_results.dkim_result=fail is not pass
exit=1
```

Fabricated signature tags in wire scope:

```text
tool.bash=5.3.9(1)-release
scope=wire
results.file=/tmp/opencode/i178-fake-wire.md
connect=127.0.0.1:1025 reachable
wire.signature=d=signing.example s=sel a=ed25519-sha256
gate.real_mailbox=NOT_SATISFIED
RESULT: PASS scope=wire reason=local-sink-signature-tags-present
exit=0
```

The wire-scope PASS is explicitly scoped to local signature tags and prints
`gate.real_mailbox=NOT_SATISFIED`. No fabricated `PASS scope=real-mailbox` was
produced: manufacturing one would be the exact false claim this gate exists to
prevent.

## 4. Independent verification of the change

```text
$ git diff --check
exit=0
```

Content scans mirroring `.github/workflows/docs.yml` were clean: no AI-tool
attribution, no chat-session URL, no emoji in tracked `.md`/`.rs`/`.toml`/
`.yml`/`.yaml`/`.sh` files, no retired third-party brand from ADR-0011, no
prohibited descriptor prefix, and every descriptor carries a closing keyword.
`bash tools/check-file-size.sh` reported OK.

## 5. Exact condition to close #178

The external evidence still required is:

1. This branch (or its successor) merged and default CI green.
2. A maintainer with port 25 egress sends through `MtaSender` (feature `mta`)
   to a real mailbox they control and records: date (UTC), producing commit
   SHA, recipient address/domain, signing domain and selector, the full
   receiving-MTA `Authentication-Results` header with `dkim=pass` and matching
   `header.d=`, `header.s=`, `header.a=`, and the full `DKIM-Signature` header
   with matching `d=`, `s=`, `a=` and a non-empty `b=`.
3. `scripts/check-mail-delivery.sh` run with all opt-in variables set returns
   `RESULT: PASS scope=real-mailbox`, and the filled results file and checker
   output are attached to the PR or issue comment.

Only then may a maintainer post `Closes #178`. Until then the issue remains
open and no gate-pass claim is made.
