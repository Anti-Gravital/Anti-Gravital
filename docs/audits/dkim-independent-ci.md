# Independent DKIM verification: opt-in check evidence (#178)

Status: the independent-verifier Done-when item of issue #178 has reproducible
offline evidence. The real-mailbox gate is NOT closed. This branch records
`Refs #178` and issue #178 stays open.

## 1. Environment of the recorded run

| Item | Value |
| --- | --- |
| Date (UTC) | 2026-11-22 |
| Commit | `9a95e795dc5e15eff9946d5c438df76905f13913` |
| Branch / worktree | `community11/mailverify`, `worktrees11/mailverify` |
| OS / arch | Linux 7.0.0-44-generic x86_64 |
| Python | 3.14.4 |
| pip | 26.3 |
| dkimpy | 1.1.8 (BSD-like / DFSG approved) |
| dnspython | 2.9.2 (dkimpy dependency) |
| PyNaCl | 1.6.2 (Apache-2.0) |
| cargo | 1.99.1 (5f94df478 2026-08-27) |
| rustc | 1.99.1 (58cf8f9c3 2026-10-13) |
| Cargo target dir | `/home/angel-nereira/Github-local-angelnereira/shared-cargo-target` |

No new Cargo dependency was added, so `cargo deny` / `cargo audit` positions
are unchanged. dkimpy and PyNaCl are test-only Python tooling.

## 2. Commands executed

```sh
# Syntax check of the new wrapper
bash -n scripts/check-dkim-independent.sh

# Real run over the embedded vectors (stdout + stderr, exit status printed)
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
bash scripts/check-dkim-independent.sh; echo "exit=$?"
```

The wrapper itself invokes:

```sh
cargo test --locked -p ag-mail --features mta --lib \
  sender::mta::dkim::tests::dkim_signatures_verify_with_dkimpy \
  -- --exact --ignored --nocapture
```

## 3. Recorded output (real run)

```text
tool.python3=Python 3.14.4
tool.cargo=cargo 1.99.1 (5f94df478 2026-08-27)
tool.rustc=rustc 1.99.1 (58cf8f9c3 2026-10-13)
tool.dkimpy=1.1.8
tool.pynacl=1.6.2
command=cargo test --locked -p ag-mail --features mta --lib sender::mta::dkim::tests::dkim_signatures_verify_with_dkimpy -- --exact --ignored --nocapture
cargo.status=0
vectors.verified=2
RESULT: PASS reason=dkimpy-verified-recorded-0-9-and-fresh-0-15-vectors
exit=0
```

`vectors.verified=2` is the wrapper's count of `dkim=pass` lines emitted by
`tools/dkim_verify.py`: one for the signature recorded from mail-auth 0.9 and
one for a fresh mail-auth 0.15 signature. A direct run of the ignored test in
the same environment showed both verdict lines:

```text
running 1 test
dkim=pass file=/tmp/ag-mail-dkim-3448487/old-0-9.eml d=send.example s=sel a=ed25519-sha256 c=relaxed/relaxed
dkim=pass file=/tmp/ag-mail-dkim-3448487/fresh-0-15.eml d=send.example s=sel a=ed25519-sha256 c=relaxed/relaxed

test sender::mta::dkim::tests::dkim_signatures_verify_with_dkimpy ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 113 filtered out; finished in 0.17s
```

The temporary directory suffix varies per process; the two `dkim=pass` lines
and the single-test result are the evidence.

### 3.1 Controlled SKIP-path check

To prove that a missing dependency is not degraded to PASS, the wrapper was
re-run with a stub `python3` earlier in `PATH` that reports both modules as
missing. This is a controlled stub, not a real missing-dependency environment:

```text
tool.python3=Python 3.99.0 (stub)
tool.cargo=cargo 1.99.1 (5f94df478 2026-08-27)
tool.rustc=rustc 1.99.1 (58cf8f9c3 2026-10-13)
tool.dkimpy=missing
tool.pynacl=missing
RESULT: SKIP reason=missing-dependency detail=dkimpy and PyNaCl are both required; missing dependency is never PASS
exit=0
```

The result is `SKIP`, never `PASS`. Exit 0 for SKIP keeps an explicitly
dispatched job green when tooling is absent; consumers must read the result
line, not just the exit status, to distinguish evidence from an unavailable
check.

## 4. Outcome semantics

| Outcome | Condition | Exit |
| --- | --- | --- |
| `PASS` | dkimpy + PyNaCl present; cargo status 0; at least two `dkim=pass` lines; no `dkim=fail`/`dkim=error` | 0 |
| `SKIP` | python3, dkimpy or PyNaCl unavailable | 0 (never rendered as PASS) |
| `FAIL` | rejected vector, verifier error, or nonzero cargo status (including a zero-test run) | 1 |

Full test path plus `--exact` makes a zero-test run fail: without the right
features the named test is not compiled, so the two `dkim=pass` lines cannot
appear and the wrapper reports FAIL.

## 5. CI wiring

`.github/workflows/dkim-independent.yml` is `workflow_dispatch` only. It has no
`push` or `pull_request` trigger, so default CI is unchanged and stays green.
The job creates a venv, installs `dkimpy==1.1.8` and `pynacl==1.6.2` (pinned),
and runs the wrapper in the same step. Authorization: RFC-0035 (accepted,
comment period waived). See `docs/rfc/RFC-0035-independent-dkim-ci.md`.

## 6. Exact condition to close #178

The offline independent-verifier item is covered above, but issue #178 is not
closed by it. Both of the following are required before posting `Closes #178`:

1. This branch (or its successor) is merged and default CI stays green; and
2. a maintainer with **port 25 egress** sends a message through the native MTA
   (feature `mta`) to a **real mailbox** and records:
   - the delivery date and the commit SHA that produced the message,
   - the recipient domain and the sending domain/selector,
   - the full receiver `Authentication-Results` header showing `dkim=pass`
     with matching `header.d=send.example`, `header.s=sel` and the
     `ed25519-sha256` algorithm.

The header must come from a receiving MTA, not from `dkim.verify()` in a local
Python process. Port 25 egress is blocked in this environment, so item 2 is an
external, manual gate. Until it is recorded, #178 remains open with
`Refs #178`; no gate-pass claim is made here.
