# Issue board reconciliation - wave 4 (2026-11-22)

Scope: reconcile the 13 issues that remained open after the wave 3 merges so
the board reflects the real state of the repository. Source of truth for each
classification is the issue body, its latest comments and the accepted
ADR/RFC/roadmap decision that governs it. No issue was closed by this work.

This document keeps the wave-4 matrix below as the historical baseline. The
wave-9 addendum at the end records the current owner, status, evidence command
and exact closure condition for every open issue. It does not override any
closure condition below; it only adds current cross-links.

## Reconciliation matrix

Legend:

- **Blocked (external gate)**: an external service, credential, runner,
  artifact or environment is required before the issue can be verified.
- **Deferred by accepted decision**: a decision already recorded in an
  accepted ADR/RFC or roadmap decision postpones the work; no external gate
  alone prevents it.
- **Tracking parent**: links children only; never closed while a child is open.

| Issue | Short title | Class | Governing decision | Exact unblock condition | Labels | Status |
| --- | --- | --- | --- | --- | --- | --- |
| #76 | [tracking] ag-domains remaining work | Tracking parent | RFC-0011 phases E/F; ADR-0012 | All 16 children (#78-#93) closed; #87 is the only open child | `tech-debt`, `ag-domains` | OPEN, 15 CLOSED / 1 OPEN |
| #87 | Live ACME staging end-to-end test | Blocked (external gate) | RFC-0011 phases E/F; `docs/ag-domains/how-to/acme-staging-test.md` | Controlled domain + DNS write access + outbound HTTPS to Let's Encrypt staging; run `tests/acme_staging.rs::staging_dns01_issue_and_serve_sni` | `tech-debt`, `ag-domains`, `p3`, `blocked` | OPEN, blocked |
| #112 | ag-edge producer-only enqueue wiring | Deferred by accepted decision | RFC-0012 section 17.4 / S6; ADR-0013 | A concrete ag-edge use case needs durable enqueue; wire `ag-edge -> ag-workers` with `producer` only plus an integration test | `enhancement`, `tech-debt`, `p3`, `ag-edge`, `ag-workers` | OPEN, deferred |
| #141 | Installer integrity trust anchor | Blocked (external gate) | RFC-0030 section 11 (design); ADR-0009 rule 4 | All six RFC-0030 section 11 conditions: accepted RFC, offline key in two out-of-band channels, tag-published signed manifest, clean-machine verification, CI parity, SECURITY.md limits | `enhancement`, `p3`, `security`, `blocked` | OPEN, `blocked` added this wave |
| #142 | [tracking] Repository audit through Phase 4.6 | Tracking parent | CLAUDE.md rule 29; audit PR #159/#163 | All 31 children closed; 5 remain open (#141, #152, #154, #156, #157); #149 is now closed | `tech-debt` | OPEN, 26 CLOSED / 5 OPEN |
| #152 | ag-mail managed REST API and marketing objects | Deferred by accepted decision | RFC-0009 Mail A gate; Hoja de Ruta 4.6.3 BDFL decision 2026-11-22; `docs/modules/ag-mail/managed-api-dx-design.md` | RFC-0009 Mail A evidence (real external delivery with aligned SPF/DKIM/DMARC) or an RFC-0009 amendment moving C/D out; port 25 egress blocked (#153) | `enhancement`, `tech-debt`, `p2`, `ag-mail` | OPEN, deferred |
| #154 | ag-data DSL-generated ORM, RLS, tenancy | Deferred by accepted decision | ADR-0020 (RFC-0025 accepted as design) | Phase 5 opens AND every blocking row of the pre-Phase-5 release gate is `pass` (Audit/Deny currently `fail`) | `enhancement`, `tech-debt`, `p3`, `ag-data` | OPEN, deferred |
| #156 | 24h fuzz manual gate | Blocked (external gate) | `docs/fuzz/24h-gate-verification.md`; `docs/audits/pre-fase5-fuzzing.md` section 4.5 | Funded Linux x86-64 runner with a 24h budget runs all SEVEN fuzz targets, archives logs and seguridad reviews crash triage | `tech-debt`, `p2`, `blocked` | OPEN, blocked |
| #157 | Final benchmark evidence (Stage 6) | Blocked (external gate) | `docs/audits/PRE_FASE5_RELEASE_GATE.md`; `docs/benchmarks/verification-package.md` | Reference hardware + load generator + stabilized code; run criterion/oha suites, write `docs/benchmarks/pre-fase5-performance.md` with rule-17 metadata, flip gate row, reconcile README | `tech-debt`, `p2`, `blocked` | OPEN, blocked |
| #178 | Mail protocol stack upgrade | Blocked (external gate) | RFC-0009 Mail A gate; `docs/security/DKIM_INDEPENDENT_VERIFICATION.md` | Dependency upgrade and independent DKIM verification are satisfied; a runner with port 25 egress sends through the native MTA to a real mailbox and records the `Authentication-Results` header | `tech-debt`, `p2`, `ag-mail`, `blocked` | OPEN, blocked |
| #179 | instant-acme 0.7 to 0.8 upgrade | Blocked (external gate) | #87; RFC-0011; `docs/ag-domains/reference/instant-acme-0.8-migration.md` | Live ACME staging E2E (#87) passes before and after the 0.7 -> 0.8 order-flow port; no unverified rewrite of renewal | `tech-debt`, `ag-domains`, `p3`, `blocked` | OPEN, blocked |
| #180 | chumsky 0.9 to 0.13 evaluation | Deferred by accepted decision | ADR-0021 (RFC-0027 accepted: keep 0.9.3) | DSL v1.0 grammar freeze or a concrete blocker (advisory, unfixable DX defect, measured bottleneck); target 0.14.x, not 0.13 | `ag-dsl`, `tech-debt`, `p3` | OPEN, deferred |
| #185 | Replace unmaintained chrono | Blocked (external gate) | RFC-0029 + RFC-0026 (both in-repo roots removed) | `object_store` drops its non-optional chrono or a newer compatible release lands; drop the RUSTSEC-2026-0543 ignore and pass `cargo deny --all-features check` and `cargo tree -i chrono` | `tech-debt`, `p2`, `security`, `blocked` | OPEN, blocked |

## Labels applied

- Added `blocked` to **#141**. It is the only issue in the externally gated
  set that did not already carry the label. The gate is an out-of-band trust
  anchor: an offline signing key, two independent publication channels and a
  tagged release artifact (RFC-0030 section 11). None exist yet.
- Kept `blocked` on #87, #156, #157, #178, #179 and #185 after confirming each
  gate is still real and unresolved.
- Kept `p1`/`p2`/`p3`, area and type labels unchanged on every issue.
- Did NOT add `blocked` to the four decision-deferred issues (#112, #152,
  #154, #180). They are postponed by an accepted decision, not by a missing
  external gate; adding `blocked` there would misstate the board.

## Status comments posted

A short status comment with the exact unblock condition and the authorizing
document was posted on all 13 issues on 2026-11-22. Comment URLs:

| Issue | Comment |
| --- | --- |
| #76 | https://github.com/Anti-Gravital/Anti-Gravital/issues/76#issuecomment-7073192860 |
| #87 | https://github.com/Anti-Gravital/Anti-Gravital/issues/87#issuecomment-7073193069 |
| #112 | https://github.com/Anti-Gravital/Anti-Gravital/issues/112#issuecomment-7073193307 |
| #141 | https://github.com/Anti-Gravital/Anti-Gravital/issues/141#issuecomment-7073193538 |
| #142 | https://github.com/Anti-Gravital/Anti-Gravital/issues/142#issuecomment-7073193819 |
| #152 | https://github.com/Anti-Gravital/Anti-Gravital/issues/152#issuecomment-7073194063 |
| #154 | https://github.com/Anti-Gravital/Anti-Gravital/issues/154#issuecomment-7073194430 |
| #156 | https://github.com/Anti-Gravital/Anti-Gravital/issues/156#issuecomment-7073194700 |
| #157 | https://github.com/Anti-Gravital/Anti-Gravital/issues/157#issuecomment-7073194977 |
| #178 | https://github.com/Anti-Gravital/Anti-Gravital/issues/178#issuecomment-7073195200 |
| #179 | https://github.com/Anti-Gravital/Anti-Gravital/issues/179#issuecomment-7073195459 |
| #180 | https://github.com/Anti-Gravital/Anti-Gravital/issues/180#issuecomment-7073195909 |
| #185 | https://github.com/Anti-Gravital/Anti-Gravital/issues/185#issuecomment-7073196238 |

A correction to the #76 child count was posted at
https://github.com/Anti-Gravital/Anti-Gravital/issues/76#issuecomment-7073205646
(#76 tracks 16 children, 15 CLOSED and 1 OPEN; the first comment said 15/14).

## Tracking parent checklists

Both parent bodies were updated with verified counts from the live board.

### #76 - ag-domains remaining work (RFC-0011 E/F)

16 children (#78-#93): 15 CLOSED, 1 OPEN.

- [x] #78, #79, #80, #81, #82, #83, #84, #85, #86 (correctness + phase E)
- [ ] #87 live ACME staging E2E (OPEN, `blocked` on domain/DNS/egress)
- [x] #88, #89, #90, #91, #92, #93 (observability, security, edge, docs, phase F)

The parent stays OPEN while #87 is open. Never close a parent with open children.

### #142 - repository audit through Phase 4.6

31 children: 26 CLOSED, 5 OPEN. #149 is now CLOSED (community2/otlp merged),
so the previous "six children open" statement was stale.

Open children, all linked from the parent body:

- [ ] #141 installer integrity trust anchor (`blocked`, RFC-0030)
- [ ] #152 ag-mail managed REST API and marketing objects (deferred, RFC-0009 Mail A gate)
- [ ] #154 ag-data DSL ORM/RLS/tenancy (deferred, ADR-0020)
- [ ] #156 24h fuzz manual gate (`blocked`)
- [ ] #157 final benchmark evidence (`blocked`)

The parent stays OPEN until all five are closed.

## Closure instructions for the coordinator

No issue was closed by this reconciliation and no `Closes #NNN` keyword was
used in this branch. Every closure remains conditional on verifiable evidence.
Use the exact keywords below only after the stated evidence exists and is
recorded in the merge PR.

| Issue | Exact keyword to use when verified | Evidence required before using it |
| --- | --- | --- |
| #76 | `Closes #76` | `Closes #87` merged first; all 16 children CLOSED |
| #87 | `Closes #87` | Credentialed staging run passes and logs are linked; then update parent #76 |
| #112 | `Closes #112` | Concrete ag-edge consumer exists; `producer`-only wiring and integration test land; dependency cycle check green |
| #141 | `Closes #141` | All six RFC-0030 section 11 conditions hold; key ceremony, signed release manifest, clean-machine run and CI parity recorded |
| #142 | `Closes #142` | All five open children (#141, #152, #154, #156, #157) CLOSED |
| #152 | `Closes #152` | RFC-0009 Mail A gate evidence recorded, or an RFC-0009 amendment moves C/D out; REST/marketing surface tested |
| #154 | `Closes #154` | Phase 5 open, pre-Phase-5 gate rows `pass`, generated ORM/RLS/tenancy code tested |
| #156 | `Closes #156` | 24h logs for all seven targets archived with no unresolved crashes; crash triage reviewed |
| #157 | `Closes #157` | Rule-17 benchmark evidence on reference hardware; gate row and README reconciled |
| #178 | `Closes #178` | `Authentication-Results` header recorded from a real-mailbox send with port 25 egress |
| #179 | `Closes #179` | Staging E2E (#87) passes before and after the 0.8 port |
| #180 | `Closes #180` | RFC decision to migrate (supersedes ADR-0021), tests unchanged, generated artifacts byte-identical, fuzz smoke green |
| #185 | `Closes #185` | `cargo tree -i chrono` empty under all features, RUSTSEC-2026-0543 ignore removed, deny/audit/test green |

### Wave-4 branch closure handoff

No wave-4 branch resolves any of the 13 issues. Guardrails (RFC-0022) and hooks
address CI/hook tooling, not a board issue; their descriptors declare
`Closes: none`. The guardrails allowlist refers to #142 only as context for a
file-size exception, not as a closure. There is therefore no wave-4
`Closes #NNN` to hand off at this point. If a later branch fully resolves one
of these issues, the coordinator must use the exact `Closes #NNN` from that
branch's own descriptor at merge time, not from this document, and must not
close the issue before that merge is verified.

## New reviewer issues

No new issues were opened by reviewers before this reconciliation. The open
board was exactly the 13 issues listed above. Cross-role notes were checked:
`w4-review.md` records no findings and no issues opened, and `w4-guardrails.md`
defers its runtime security issue to workflow execution rather than opening one
now. No reviewer issue had appeared at final verification either.

## Verification performed

- All 13 issue bodies, labels and latest comments read via `gh`.
- Child states for both parents verified individually via `gh issue view`:
  #76 = 16 children (15 CLOSED, 1 OPEN); #142 = 31 children (26 CLOSED, 5 OPEN).
- All 13 status comments are <= 256 characters, ASCII only, no emoji and no
  AI-tool attribution.
- Label changes limited to adding `blocked` to #141; all other labels preserved.
- No issue was closed, reassigned or edited outside the two parent bodies.
- Master documents in `docs/master/` were not touched.

## Wave 9 addendum (2026-11-22) - owner and evidence matrix

Scope: cross-link every open issue with the wave-9 role that touches it, or
state explicitly that no role does. The wave-4 classification above remains the
authority for why an issue is blocked or deferred. The evidence commands below
are the exact reproducible checks an owner or coordinator must run; a short
fuzz window, a local ACME mock, a reduced benchmark sample or a partial file
split is NOT gate evidence and never closes an issue.

| Issue | Wave-9 owner | Status (2026-11-22) | Evidence command | Exact closure condition |
| --- | --- | --- | --- | --- |
| #76 | none; parent monitored by trackers | OPEN, 16 children (15 CLOSED / 1 OPEN) | `gh issue list --state open` filtered to children #78-#93 | `Closes #87` first, then all 16 children CLOSED; only then `Closes #76` |
| #87 | dnsacme (local ACME mock harness and staging docs ONLY) | OPEN, `blocked` (external gate) | `cargo test -p ag-domains --features acme --test acme_staging -- --ignored --exact staging_dns01_issue_and_serve_sni --nocapture` | Credentialed staging run passes and real logs are linked; then update #76 |
| #112 | none; deferred by RFC-0012 S6 / ADR-0013 | OPEN, deferred | `cargo test --workspace --all-features` plus dependency-cycle check | Concrete ag-edge consumer exists; `producer`-only wiring and integration test land; acyclic dependency check green |
| #141 | none; needs a human key ceremony | OPEN, `blocked` | Read RFC-0030 section 11 and verify each of the six conditions | All six RFC-0030 section 11 conditions hold: key ceremony, signed tagged manifest, clean-machine run, CI parity |
| #142 | none; parent monitored by trackers | OPEN, 31 children (26 CLOSED / 5 OPEN) | `gh issue list --state open` filtered to its 31 children | All five open children (#141, #152, #154, #156, #157) CLOSED |
| #152 | none; deferred by RFC-0009 Mail A gate / BDFL decision | OPEN, deferred | RFC-0009 Mail A gate evidence; real delivery with aligned SPF/DKIM/DMARC | Mail A evidence recorded, or an amendment moving C/D out; REST/marketing surface tested |
| #154 | none; deferred by ADR-0020 | OPEN, deferred | Read `docs/audits/PRE_FASE5_RELEASE_GATE.md`: Audit and Deny rows are still `fail` | Phase 5 open, all blocking gate rows `pass`, generated ORM/RLS/tenancy tested |
| #156 | qafuzz (fuzz automation and short-window evidence ONLY) | OPEN, `blocked` (external gate) | `cargo +nightly fuzz run <target> -max_total_time=86400` per each of seven targets; archived logs under `docs/fuzz/results/YYYY-MM-DD/24h/` | 24h logs for all seven targets archived, no unresolved crashes, crash triage reviewed |
| #157 | qabench (benchmark procedure and reduced-sample evidence ONLY) | OPEN, `blocked` (external gate) | `cargo bench -p ag-core -p ag-workers --no-run`, then criterion and `oha` runs per `docs/benchmarks/verification-package.md` | Rule-17 benchmark evidence on reference hardware, `pre-fase5-performance.md` written, gate row and README reconciled |
| #178 | none; security analysis explicitly NOT in wave 9 | OPEN, `blocked` | Real-mailbox send through native MTA with port 25 egress; capture `Authentication-Results` header | Header recorded from a real-mailbox send with port 25 egress |
| #179 | dnsacme (0.8 migration notes and local harness ONLY) | OPEN, `blocked` by #87 | Same staging E2E command as #87, run before and after the 0.7 -> 0.8 order-flow port | Staging E2E passes before and after the port; no unverified renewal rewrite |
| #180 | none; deferred by ADR-0021 (RFC-0027 accepted) | OPEN, deferred | DSL grammar freeze or concrete blocker; generated artifacts byte-identical; fuzz smoke green | RFC superseding ADR-0021, tests unchanged, artifacts byte-identical, fuzz smoke green |
| #185 | none; security analysis explicitly NOT in wave 9 | OPEN, `blocked` | `cargo tree -i chrono` (empty under all features); `cargo deny --all-features check` | `object_store` drops non-optional chrono, RUSTSEC-2026-0543 ignore removed, deny/audit/test green |
| #189 | refactor (commands/deploy.rs and commands/dns.rs ONLY) | OPEN, in progress in wave 9 | `bash tools/check-file-size.sh`; `cargo test -p ag-cli -p ag-core`; `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Every allowlisted file at or below 600 lines or split into focused modules, its allowlist entry removed, checks green |
| #198 | trackers (filed this wave) | OPEN, no owner assigned yet | `cargo audit`; `cargo deny check advisories`; `cargo tree -i url` | Gate Audit/Deny rows cite current RUSTSEC-2026-0592 blocker and an OPEN tracker; stale #173/#174 text corrected |

### Ownership notes

- #189 is a multi-owner issue. `refactor` owns only `commands/deploy.rs` and
  `commands/dns.rs`. `clidev` owns `cli.rs`, `config.rs` and `main.rs`;
  `coredev` owns `crates/ag-core/**`, including `env.rs`. The issue body's file
  list was extended by a later comment that added the wave-7/8 modules. No
  single wave-9 role can close #189: the coordinator must combine all owners'
  splits and the allowlist cleanup before using `Closes #189`.
- #185 and #178 have no wave-9 owner. Security analysis for both is explicitly
  outside this wave; their blocked status and conditions are unchanged.
- #141 needs a human. The RFC-0030 section 11 key ceremony cannot be performed
  by an automation agent, so no agent owner is assigned.
- #112, #152, #154 and #180 are deferred by accepted decisions. Wave 9 adds no
  work to them and does not add a `blocked` label.
- #156, #157, #87 and #179 have a wave-9 owner only for local automation,
  evidence or documentation. The external gate still belongs to the
  coordinator; none of these issues may be closed from wave-9 branch evidence.

### Wave-9 cross-link comments

Short status comments were posted on the five issues with a new assignment.
Each is ASCII only and at most 256 characters. No other issue was commented on
because the wave-4 comments already state its status.

| Issue | Comment |
| --- | --- |
| #189 | https://github.com/Anti-Gravital/Anti-Gravital/issues/189#issuecomment-7079363459 |
| #156 | https://github.com/Anti-Gravital/Anti-Gravital/issues/156#issuecomment-7079363705 |
| #157 | https://github.com/Anti-Gravital/Anti-Gravital/issues/157#issuecomment-7079363952 |
| #87 | https://github.com/Anti-Gravital/Anti-Gravital/issues/87#issuecomment-7079364282 |
| #179 | https://github.com/Anti-Gravital/Anti-Gravital/issues/179#issuecomment-7079364623 |

### Board inconsistency filed this wave

Issue #198 (https://github.com/Anti-Gravital/Anti-Gravital/issues/198) records a
real false-status inconsistency. The release gate Audit and Deny rows are
`fail` with 2026-11-10 evidence that names #173 and #174 and three advisories
that are now resolved in the lockfile (`crypto-bigint` 0.7.11,
`libsqlite3-sys` 0.39.0, `astral-tokio-tar` 0.6.4 under a dated `deny.toml`
ignore). Both #173 and #174 are CLOSED. The gate is still red for a different
and untracked reason: RUSTSEC-2026-0592 (`url` 2.5.8, fixed in 2.5.9), published
2026-11-20. Verified here with `cargo audit` (1 vulnerability),
`cargo deny check advisories` (`advisories FAILED`, one error) and
`cargo tree -i url` (direct dependency of `ag-core` and `ag-auth`).

This addendum closes nothing and adds no `Closes #NNN` keyword. Parent counts
are unchanged: #76 still has one open child (#87) and #142 still has five open
children (#141, #152, #154, #156, #157), so neither child checklist was edited.
