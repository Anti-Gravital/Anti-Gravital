# Revalidation of deferred issues #178, #152 and #154 (2026-11-23)

- Date: 2026-11-23. Checker runs recorded at 2026-11-24T00:00:02Z UTC.
- Branch / worktree: `community29/i178` / `worktrees29/i178`.
- Baseline commit: `97b9402f9e5127924c39a9c75174e6444de7ad78`.
- Nature: additive audit note. It closes no issue, flips no release-gate row,
  amends no ADR/RFC and authorizes no Phase 5 work. All three issues stay OPEN.
- Scope: revalidate the recorded state of three deferred issues against the
  live board and the governing documents, and restate the exact close
  condition for each. No credential, mailbox address, private key or
  `Authentication-Results` header is reproduced here. Header field names and
  placeholder domains only.

## 1. Verified current state

Board state read with `gh api repos/Anti-Gravital/Anti-Gravital/issues/<n>`
(read-only) on 2026-11-23:

| Issue | Board state | Labels | Class | Current state |
| --- | --- | --- | --- | --- |
| #178 | OPEN | `tech-debt`, `blocked`, `p2`, `ag-mail` | Blocked (external gate) | The three mail crates were upgraded together; the independent offline DKIM verifier and the opt-in checkers exist. The external real-mailbox delivery evidence does not. |
| #152 | OPEN | `enhancement`, `tech-debt`, `p2`, `ag-mail` | Deferred by accepted decision | Design-only REST annex and DX contract exist. No route, data model, CLI command or Cargo feature is implemented. Mail C/D are deferred by the BDFL decision of 2026-11-22 (Hoja de Ruta 4.6.3). |
| #154 | OPEN | `enhancement`, `tech-debt`, `p3`, `ag-data` | Deferred by accepted decision | RFC-0025 is ACCEPTED AS DESIGN with implementation deferred by ADR-0020. No ORM, RLS or tenant-context code exists. |

Live titles match the board reconciliation: "Upgrade the mail protocol stack
(mail-send 0.7, mail-auth 0.15, mail-parser 1.0) in the native MTA" (#178),
"ag-mail native MTA: managed REST API and marketing objects (RFC-0009
§4.5/4.6, Phases 4.6-C/D)" (#152), and "ag-data: DSL-generated typed ORM,
row-level security and multi-tenancy (Phase 5+)" (#154). The latest issue
comments carry the same conditions as `docs/audits/issue-board-reconciliation.md`
(wave 4, 9, 11 and 12 addenda). No comment or body claims these issues are
closed.

The pre-Phase-5 gate in `docs/audits/PRE_FASE5_RELEASE_GATE.md` is still OPEN:
Audit and Deny are `pass` on merged commit `800b671`, while Fuzz 24h (#156),
Benchmarks (#157) and Open debt remain `pending`. Phase 5 is not open.

## 2. Prepared but not executed

Two distinct gaps must be named explicitly. Neither is closed by this note.

### 2.1 Real mailbox: PREPARED, NOT EXECUTED (#178, prerequisite for #152)

Prepared: the reproducible procedure and field list in
`docs/security/REAL_MAILBOX_VERIFICATION.md`, the empty results template
`docs/security/REAL_MAILBOX_RESULTS_TEMPLATE.md`, the opt-in checker
`scripts/check-mail-delivery.sh`, and the offline independent DKIM verifier
kit (`docs/security/DKIM_INDEPENDENT_VERIFICATION.md`,
`scripts/check-dkim-independent.sh`).

Not executed: no message was sent through `MtaSender` from a port-25-enabled
runner, no mailbox was contacted, no receiving MTA was observed, and no filled
`docs/security/REAL_MAILBOX_RESULTS.md` exists in this tree. Consequently no
genuine `Authentication-Results` header has been recorded and none is
fabricated here. Port 25 egress is unavailable in this environment (#153).

The offline dkimpy PASS recorded in section 3 verifies embedded known-answer
vectors only. It contacts no receiver and observes no `Authentication-Results`
header. A local loopback sink, a local TXT answer and `dkim.verify()` are
equally insufficient. This is the second Done-when item of #178 and the Mail A
gate for #152; both remain unsatisfied.

### 2.2 Phase 5 authorization: PREPARED, NOT EXECUTED (#154)

Prepared: RFC-0025 is accepted as design, ADR-0020 records the deferral, and
`docs/modules/ag-data/phase5-implementation-readiness.md` fixes the concrete
implementer seams (DSL `tenant`/`global` markers, `TenantContext` flow,
`ag-auth` claim hand-off, RLS drift-gate coverage). The managed-API design for
#152 is likewise prepared in `docs/architecture/ag-mail-managed-api-design.md`
with gates G1-G5 explicitly open.

Not executed: no Phase 5 phase was opened, no BDFL authorization for Mail C/D
or for RFC-0025 increment I1/I2/I3 was recorded, no release-gate row was
flipped, and no implementation PR exists. The five-PR plan in RFC-0025 section
5 and the REST/marketing implementation in the design annex are future work.
Design documents are not implementation authorization.

## 3. Checker verification in this environment (recorded output)

All commands were run from the worktree at baseline commit
`97b9402f9e5127924c39a9c75174e6444de7ad78`, date 2026-11-24T00:00:02Z UTC.
Only the `RESULT:` lines and tool versions are reproduced; no secrets.

```text
$ bash scripts/check-mail-delivery.sh
RESULT: SKIP reason=not-opted-in detail=set AG_MAIL_DELIVERY_OPT_IN=1 to enable the delivery gate check
exit=0

$ AG_MAIL_DELIVERY_OPT_IN=1 bash scripts/check-mail-delivery.sh
tool.bash=5.3.9(1)-release
scope=real-mailbox
results.file=docs/security/REAL_MAILBOX_RESULTS.md
RESULT: SKIP reason=probe-target-not-configured detail=set AG_MAIL_DELIVERY_PROBE_TARGET=host:25
exit=0

$ AG_MAIL_DELIVERY_OPT_IN=1 AG_MAIL_DELIVERY_PROBE_TARGET=127.0.0.1:25 bash scripts/check-mail-delivery.sh
tool.bash=5.3.9(1)-release
scope=real-mailbox
results.file=docs/security/REAL_MAILBOX_RESULTS.md
RESULT: SKIP reason=port-25-egress-unavailable detail=TCP connect to 127.0.0.1:25 failed or timed out
exit=0

$ bash scripts/check-dkim-independent.sh
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

Interpretation, without spin: the real-mailbox checker honestly reports SKIP
in every configuration available here, never PASS. The independent verifier
reports PASS because dkimpy and PyNaCl are installed in this host; that PASS
is offline known-answer evidence and is explicitly NOT a real-mailbox PASS,
NOT a receiver verdict and NOT gate evidence for #178 or #152. Exit 0 on SKIP
keeps an opt-in job green without claiming a delivery.

## 4. Exact close condition per issue

### 4.1 #178 - mail protocol stack upgrade

All of the following must hold before `Closes #178` is posted by a maintainer:

1. The native-MTA test tooling and the verification kit are merged and default
   CI stays green.
2. A maintainer with port 25 egress delivers a message signed by `ag-mail`
   through `MtaSender` (feature `mta`) to a real mailbox they control and
   records: UTC date, producing commit SHA, recipient address/domain, signing
   domain (`d=`) and selector (`s=`), the full receiving-MTA
   `Authentication-Results` header with `dkim=pass` and matching `header.d=`,
   `header.s=` and `header.a=`, and the full `DKIM-Signature` header with
   matching `d=`, `s=`, `a=` and a non-empty `b=`.
3. `scripts/check-mail-delivery.sh` returns `RESULT: PASS scope=real-mailbox`
   with all opt-in variables set, and the filled results file plus checker
   output are attached to the PR or issue comment.

Sources: `docs/security/REAL_MAILBOX_VERIFICATION.md` sections 1, 2, 5 and 6;
`docs/audits/mail-delivery-gate.md` section 5; `docs/audits/estado-honesto-puertas.md`
("Buzón real DKIM" row).

### 4.2 #152 - ag-mail managed REST API and marketing objects

Close requires the authorization gates and the implementation Done-when, not
just the design:

1. G1/G2: RFC-0009 Mail A evidence - real external delivery with aligned SPF,
   DKIM and DMARC, with date, commit, recipient domain and full headers - OR a
   formal RFC-0009 amendment under Hoja de Ruta 4.6.3 that moves or reorders
   C/D, with the 2026-11-22 BDFL deferral lifted for C/D.
2. G3-G5: the DX contract and the architecture annex accepted; RFC-0024
   amended with the mail/marketing tool groups and six capabilities; CLI
   decision taken; acceptance tests against local/ephemeral PostgreSQL 16 with
   `cargo test -p ag-mail --features api` and the `mail-*` CI jobs green; new
   dependencies justified.
3. Done-when: REST routes and the tenant-scoped PostgreSQL data model with
   idempotency land behind `api`; the marketing objects (broadcasts, contacts,
   audiences, segments, topics and RFC 8058 one-click unsubscribe) land behind
   `api`; tests cover them against local PostgreSQL; the native default and
   relay sender are unchanged; DEBT-021 is updated.

If the amendment path is chosen instead, this annex and the DX design feed the
RFC-0009 amendment, #152 is retargeted to a later phase, and no Phase 4.6 code
is written. Sources: `docs/architecture/ag-mail-managed-api-design.md`
sections 1 and 6; `docs/modules/ag-mail/managed-api-dx-design.md`;
`docs/audits/issue-board-reconciliation.md` rows for #152.

### 4.3 #154 - ag-data typed ORM, RLS and multi-tenancy

The deferral is lifted only when all of the following hold:

1. Every blocking row of `docs/audits/PRE_FASE5_RELEASE_GATE.md` is `pass`.
   Pending at this revalidation: Fuzz 24h (#156), Benchmarks (#157) and Open
   debt. Audit and Deny are already `pass` on merged commit `800b671`.
2. Phase 5 opens per the Hoja de Ruta, and a separate implementation PR
   records the gate closure and the phase before merging any increment
   (ADR-0020 point 3).
3. The timestamp dependency is met: the DSL maps to `time::OffsetDateTime`
   and RFC-0026 fixes the serde contract. This dependency no longer blocks.
4. If the committee treats the section 4.1 grammar or the section 4.3
   `ag-auth` addition as beyond RFC-0025, a successor RFC is accepted first.
5. The RFC-0025 section 5 five-PR plan implements I1 (typed ORM), I2 (RLS) and
   I3 (async propagation and audited admin scope), and every acceptance box in
   RFC-0025 section 9 passes: `TenantId`/`TenantContext` with no new
   `ag-core` crate dependency; a repository method without context does not
   compile; explicit tenant predicate and insert column; `USING`/`WITH CHECK`
   policies with RLS enabled and forced and no `BYPASSRLS` on the runtime role;
   `set_config('app.tenant_id', $1, true)` inside `AgTx`; two-tenant live tests
   including the reused-connection case; `ag generate --check` covering the
   generated RLS migration; golden timestamp strings; audited
   `TenantScope::All`; master, module and diagram docs updated in the same PR.

Sources: RFC-0025 sections 5, 9 and 10; ADR-0020;
`docs/modules/ag-data/phase5-implementation-readiness.md` section 8;
`docs/audits/issue-board-reconciliation.md` rows for #154.

## 5. Verification of this change

Run after the files in this branch were written; recorded results are part of
the commit evidence. No gate row, issue state or ADR/RFC status was changed.

| Check | Command | Result |
| --- | --- | --- |
| Whitespace | `git diff --check` | exit 0, no whitespace errors |
| Kaizen | `scripts/kaizen check` | exit 0; descriptores, politicas de contenido, formato, lint, tests and ratchet all green |
| Local gate | `scripts/ag-check` | exit 0; `PASS=8 FAIL=0 SKIP=0` (descriptors, fmt, clippy, test, audit, deny, masters, prohibited) |
| Content scan | grep on new files for key prefixes, private keys, `whsec_`, fabricated receiver headers and AI-tool prefixes | no matches |

## 6. References

- `docs/security/REAL_MAILBOX_VERIFICATION.md` and
  `docs/security/REAL_MAILBOX_RESULTS_TEMPLATE.md` (#178 hand-off).
- `docs/security/DKIM_INDEPENDENT_VERIFICATION.md` (#178 offline verifier).
- `docs/audits/mail-delivery-gate.md` (#178 checker evidence).
- `docs/architecture/ag-mail-managed-api-design.md` (#152 design and G1-G5).
- `docs/rfc/RFC-0025-ag-data-orm-rls.md` and ADR-0020 (#154 contract).
- `docs/modules/ag-data/phase5-implementation-readiness.md` (#154 seams).
- `docs/audits/PRE_FASE5_RELEASE_GATE.md` (current gate state).
- `docs/audits/issue-board-reconciliation.md` (board classification).

Closure keywords: `Refs #178`, `Refs #152`, `Refs #154`. No `Closes #NNN` is
used by this branch.
