# Independent DKIM verification for `ag-mail`

Status: verifier available and recorded (2026-11-22); an opt-in wrapper and a
manual-only CI check were added afterwards (section 7). This document is the
reproducible evidence for the second Done-when item of issue #178: a DKIM
signature produced by the upgraded stack verifies with an implementation
outside `mail-auth`.

## 1. Why the cross-version test is not enough

The known-answer test in
`crates/ag-mail/src/sender/mta/dkim.rs` (`dkim_known_answer_old_signature_verifies_and_new_signature_verifies`)
checks two things:

- a signature recorded from `mail-auth` 0.9 verifies under `mail-auth` 0.15;
- a signature produced by `mail-auth` 0.15 verifies under `mail-auth` 0.15.

Both sides share canonicalization, hashing, header selection and header
writing lineage. The test is a useful migration regression guard, but it
cannot detect a defect that was present in the signing lineage all along.
The earlier substring-only assertions were weaker still. Neither satisfies
an independent-verifier requirement.

The check recorded here uses **dkimpy**, a separate DKIM implementation
(BSD-like licence, DFSG approved) with its **PyNaCl** ed25519 primitive
(Apache-2.0). `mailrs-dkim` was tried previously and discarded because it
hashes signed headers in literal `h=` order while `mail-auth` and deployed
MTAs hash in message order; `viadkim` is GPL-3.0 and would fail `cargo deny`.
No Rust dependency was added, so the workspace deny/audit position is
unchanged.

## 2. Vector and source locations

| Item | Location |
| --- | --- |
| Signer under test | `crates/ag-mail/src/sender/mta/dkim.rs`, `DkimConfig::sign` |
| Ed25519 private key | same file, `KA_ED25519_PKCS8_DER` |
| Published public key | same file, `KA_PUBLIC_KEY_B64` (`p=` tag in `sel._domainkey.send.example`) |
| Fixed message | same file, `KA_MESSAGE` |
| Signature recorded from 0.9 | same file, `KA_SIGNATURE_0_9` |
| Verifier tool | `tools/dkim_verify.py` |
| Ignored test that drives it | same file, `dkim_signatures_verify_with_dkimpy` |

The test writes the old vector and a freshly produced 0.15 signature to a
temporary directory and passes both files to `tools/dkim_verify.py`. The
Python tool answers the single `sel._domainkey.send.example.` TXT query
locally with the fixed `p=` record; it never touches the network.

## 3. Prerequisites

Recorded environment: Python 3.14.4, pip 26.3, dkimpy 1.1.8, dnspython
2.9.2, PyNaCl 1.6.2, Linux x86_64.

```sh
# Bootstrap pip if it is not present (this image had no pip and no ensurepip).
curl -sS -o /tmp/get-pip.py https://bootstrap.pypa.io/get-pip.py
python3 /tmp/get-pip.py --user --break-system-packages

# Permissively licensed, independent verifier and its ed25519 primitive.
python3 -m pip install --user --break-system-packages dkimpy pynacl
```

`--break-system-packages` is needed only on PEP 668 managed Python
installations such as this one; it can be omitted inside a virtual
environment. The two packages are test tooling, not build dependencies.

Important gotcha: dkimpy 1.1.8 **silently returns `False`** for ed25519
signatures when PyNaCl is not importable. `tools/dkim_verify.py` therefore
checks for `nacl` at startup and exits with a clear error rather than
reporting a false negative. This is why the earlier attempt appeared to
show that dkimpy rejected the vector.

## 4. Exact command

Run from the repository root with the shared Cargo target directory:

```sh
CARGO_TARGET_DIR=/home/angel-nereira/Github-local-angelnereira/shared-cargo-target \
CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 \
cargo test -p ag-mail --all-features \
  dkim_signatures_verify_with_dkimpy -- --ignored --nocapture
```

The test is `#[ignore]` on purpose: the verifier is an external Python tool
that CI does not install, so the normal `cargo test -p ag-mail --all-features`
run stays hermetic. The ignored test is explicit, recorded evidence, not a
silently skipped gate.

## 5. Recorded output

```text
running 1 test
dkim=pass file=/tmp/ag-mail-dkim-36419/old-0-9.eml d=send.example s=sel a=ed25519-sha256 c=relaxed/relaxed
dkim=pass file=/tmp/ag-mail-dkim-36419/fresh-0-15.eml d=send.example s=sel a=ed25519-sha256 c=relaxed/relaxed

test sender::mta::dkim::tests::dkim_signatures_verify_with_dkimpy ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 130 filtered out; finished in 0.17s
```

The temporary directory name varies per process; the two `dkim=pass` lines
and the test result are the evidence.

`dkim.verify()` returns a boolean. The tool renders that verdict in an
Authentication-Results-like form so it can be compared with what a receiving
MTA would emit:

```text
dkim=pass file=<message> d=send.example s=sel a=ed25519-sha256 c=relaxed/relaxed
```

The equivalent header a receiver would add is:

```text
Authentication-Results: mx.receiver.example; dkim=pass
  (ed25519-sha256 key; relaxed/relaxed) header.d=send.example header.s=sel
```

A `dkim=fail` or `dkim=error` line, or a non-zero exit status, means the
signature was rejected and release must stop.

## 6. Scope and remaining gap

This closes the independent-verifier Done-when item of #178. The other
verification step in that issue, sending through the native MTA to a real
mailbox and recording the receiver's `Authentication-Results` header, is
still manual because port 25 egress is blocked here. Issue #178 therefore
stays open with `Refs #178` until that gate is recorded.

## 7. Opt-in check wrapper and CI wiring

`scripts/check-dkim-independent.sh` wraps the ignored test with explicit
outcomes and exact tool versions:

- `PASS` (exit 0): dkimpy + PyNaCl are available and both embedded vectors
  verified. The wrapper requires two `dkim=pass` lines, no `dkim=fail` or
  `dkim=error` line, and a zero cargo status; full test path plus `--exact`
  prevents a zero-test run from being read as success.
- `SKIP` (exit 0): python3, dkimpy or PyNaCl is unavailable. This is never
  rendered as PASS; read the result line, not only the exit status.
- `FAIL` (exit 1): a vector was rejected, the verifier errored, or cargo
  failed.

Run it from the repository root:

```sh
bash scripts/check-dkim-independent.sh
```

`.github/workflows/dkim-independent.yml` is `workflow_dispatch` only. It is
not part of default CI and installs pinned `dkimpy==1.1.8` / `pynacl==1.6.2`
in a temporary venv before running the wrapper. The CI change is authorized
by `docs/rfc/RFC-0035-independent-dkim-ci.md` (accepted, comment period
waived). A real recorded run, the controlled SKIP-path check and the exact
condition to close #178 are in `docs/audits/dkim-independent-ci.md`.

This is still offline known-answer evidence: no receiver is contacted and no
`Authentication-Results` header is observed. It does not close #178.
