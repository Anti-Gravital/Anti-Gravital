# RFC-0027 - chumsky 0.9 to 0.13/0.14 for the Anti-DSL parser (amendment to RFC-0003)

- Status: ACCEPTED (recommendation to DEFER). Decision recorded in ADR-0021
  (2026-11-22, BDFL). Keep chumsky 0.9.3.
- Author: Anti-Gravital community (dependency evaluation workstream), Gravital Labs
- Draft date: 2026-11-21
- Decision date: 2026-11-22 (comment period closed; `CLAUDE.md` rule 28)
- Target phase: no new phase; amends the Fase 3 compiler stack decision
- Affected modules/crates: `ag-dsl` (`parser.rs`, `diagnostics.rs`), `fuzz/`
  parser targets, `docs/rfc/RFC-0003` (amendment), `docs/benchmarks/`
- Predecessor RFC: RFC-0003 (compiler base libraries), section 3.2, 4.1, 6 and 9
- Related issue: #180
- Reviewer: bajonivel
- Comment period: closed 2026-11-22. Decision recorded by the BDFL in
  ADR-0021; see section 9.

## 1. Motivation

Issue #180 asks for a formal evaluation of replacing `chumsky` 0.9 with the
current 0.13 line in the Anti-DSL parser, and states that the change requires
an RFC amending RFC-0003. RFC-0003 section 3.2 chose chumsky 0.9 explicitly for
stability and recorded in section 6 that a migration "will be evaluated in a
future dedicated RFC"; the number was left unassigned. This RFC fills that
placeholder with evidence and a recommendation.

The evaluation is needed now for two reasons:

1. The 0.9.3 release is from 2023. The maintained 1.0-line redesign has been
   shipping stable releases since 0.12.0 (2025-12-15), and `chumsky` 0.13.0
   (2026-05-06) plus 0.14.x (2026-11) are the current line.
2. A parser migration is not a version bump. It rewrites the 1,482-line
   `crates/ag-dsl/src/parser.rs` and resets three quality gates that are still
   open or recently closed: the 24-hour fuzz gate (#156), the diagnostics
   quality tests and the byte-identical generated artifacts for
   `examples/ecommerce-api`.

Doing nothing also has a cost: the longer the DSL grammar keeps growing on the
0.9 API, the larger the eventual rewrite. This RFC decides whether that cost is
real enough today to pay the migration cost now.

## 2. Problem

### 2.1 Current state

- `Cargo.toml` (workspace) pins `chumsky = "0.9"`; `Cargo.lock` resolves
  `chumsky 0.9.3`. `crates/ag-dsl/src/parser.rs` is written against that API
  (27 unit tests, 1,482 lines).
- The parser uses `parse_recovery_verbose()` with `Simple<Token>` errors and a
  hand-written `parse_error_to_diagnostic()` (parser.rs:864-899) that reads
  `err.reason()` (`SimpleReason::{Unexpected, Unclosed, Custom}`) and
  `err.expected()` to build the "se esperaba uno de: ..." hint.
- `Stream::from_iter(end_span, tokens.into_iter())` feeds `(Token, Span)` pairs
  so diagnostics keep byte-accurate spans from the `logos` lexer.
- `#![allow(clippy::result_large_err)]` at parser.rs:11 is needed because
  `Simple<Token>` exceeds 128 bytes (HashSet of expected tokens).

### 2.2 Release and maintenance facts

Source: crates.io registry API, fetched 2026-11-21.

| Version | Published | MSRV | Edition | Default features | Rust code lines |
| --- | --- | --- | --- | --- | --- |
| 0.9.3 (current pin) | 2023-10-24 | not declared | 2018 | `ahash`, `std`, `spill-stack` | 4,035 |
| 0.12.0 | 2025-12-15 | 1.65 | 2021 | `std`, `stacker` | 12,742 |
| 0.13.0 | 2026-05-06 | 1.65 | 2021 | `std`, `stacker` | 12,878 |
| 0.14.2 (current stable) | 2026-11-17 | 1.65 | 2021 | `std`, `stacker` | 13,381 |

Additional facts:

- `1.0.0-alpha.8` (2025-01-22) is the maximum semver tag but is still alpha;
  the maintained, semver-stable line is 0.14.2. Therefore "upgrade to 0.13"
  should be read as "upgrade to the current 0.13+/1.0-line API". If a migration
  is approved, the target must be the newest stable 0.14.x at implementation
  time, not the older 0.13.0 named in the issue.
- `cargo audit` on this worktree (2026-11-21) reports no advisory for
  `chumsky` at any pinned version; the four allowed warnings concern
  `astral-tokio-tar`, `chrono`, `humantime` and `paste`. Staleness of 0.9.3 is
  a maintenance risk, not a known vulnerability.
- MSRV is not a blocker: the workspace declares `rust-version = "1.99.1"`.
- Correction to RFC-0003: section 3.2/9 says chumsky 0.9 is used "with feature
  `label`". Version 0.9.3 has no `label` feature (its features are `ahash`,
  `std`, `spill-stack`); `.labelled()` is available without one. The
  workspace manifest correctly enables no features. This RFC records the
  correction without changing behaviour.

### 2.3 API differences (verified against 0.13.0 source and a compiled spike)

| Concern | chumsky 0.9.3 | 0.13 / 0.14 line |
| --- | --- | --- |
| Parser trait | `Parser<I, O, Error = E>` | `Parser<'a, I, O, E = extra::Default>`; lifetime + GAT-based internal optimiser |
| Input construction | `Stream::from_iter(eoi, iter)` | `Stream::from_iter(iter).map(eoi, \|(t,s)\| (t,s))`, or `&[T]` with `map_span`/`split_spanned`; EoI span supplied explicitly |
| Recovery entry point | `parse_recovery_verbose()` returns `(Option<O>, Vec<E>)` | `parse(...).into_output_errors()` or `.into_result()`; recovery is configured on the parser, not selected at call time |
| Error type | `Simple<Token>` with `SimpleReason::{Unexpected, Unclosed, Custom}`, `.reason()` and `.expected()` | `Simple<'a,T,S>` keeps only `span()` and `found()`. `Rich<'a,T,S>` has `.reason()` -> `RichReason::{ExpectedFound, Custom}`, `.expected()`, `.contexts()` |
| Unclosed delimiter | dedicated `SimpleReason::Unclosed { span, delimiter }` | no dedicated variant; must be reconstructed from labels/contexts or a custom error |
| Span combinator | `.map_with_span(\|o, span\| ...)` | `.map_with(\|o, e\| ... e.span())` |
| Recovery strategy | `skip_then_retry_until([Token; N])` | `skip_then_retry_until(skip: Parser<(),..>, until: Parser<(),..>)`; both are parsers, not a token array |
| `select!` | matches owned tokens; `Token::Ident(s) => s` yields `String` | with `ValueInput` the binding is by reference; `Token::Ident(s) => s.clone()` is required. `select_ref!` was added |
| Labelling | `.labelled()` available, no feature needed | `.labelled()` available; labels are retained by `Rich`, discarded by `Simple` |
| Large error | `result_large_err` allow already needed | `Rich` embeds `Box<RichReason>` plus a context `Vec`; the allow remains necessary |

There are roughly 50 `select!` sites, 35 `.ignore_then()`, 46 `.labelled()`,
21 `.delimited_by()` and 2 `.map_with_span()` sites in `parser.rs`; all must be
revisited. The grammar itself (the AST in `ast.rs`) does not change.

### 2.4 Local evidence

**Compiled API spike.** A scratch crate outside the repository
(`/tmp/opencode/spike`, not committed) was built against `chumsky 0.13.0` with
default features using the same token shape as `ag-dsl` (a `String`-bearing
token and `Range<usize>` byte spans). It compiled and ran, confirming:

- `Stream::from_iter(toks.into_iter()).map(eoi, |(t,s): (_, _)| (t,s))`
  preserves the lexer's byte spans and accepts the EoI span.
- `select! { Tok::Ident(s) => s.clone() }` and `.labelled("identifier")` work
  with `Rich` errors.
- `.map_with(|s, e| format!("{}@{:?}", s, e.span()))` replaces
  `.map_with_span()`.
- `skip_then_retry_until(any().ignored(), just(Tok::LBrace).ignored())` is the
  replacement for the one-argument 0.9 strategy; output type must be `()`.
- A deliberate malformed input produced
  `RichReason::ExpectedFound { found: Some(Num(7)), expected: ['Comma', 'RBrace'] }`.
  Expected tokens are now rendered as quoted patterns (for example `'Comma'`),
  not as `Debug` of `Token`, so any future exact-text assertions must use the
  new formatting.

**Local baseline of the current pipeline** (informational; not a release
benchmark and not a 0.9-vs-0.13 comparison). Rule 17 metadata:

- Hardware: x86_64, 12 vCPU Intel(R) Core(TM) i7-1255U, CPU scaling 33% during
  the run, shared host with other builds.
- OS: Linux 7.0.0-43-generic x86_64 GNU/Linux.
- Rust: rustc 1.99.1 (58cf8f9c3 2026-10-13), cargo 1.99.1; profile `release`,
  `debug = false`, `CARGO_INCREMENTAL=0`.
- Commit: `0c4d808561577f30e6030c788c8feb0947be70a3`.
- Configuration: 3,728-byte `examples/ecommerce-api/schema.ag`, 20 warm-up
  iterations, 200 measured iterations of `ag_dsl::compile()` (lex + parse +
  semantic + diagnostics), result passed to `std::hint::black_box`.
- Result: mean 1,344.3 us, standard deviation 276.3 us (20.6%), p50 1,335.9 us,
  p95 1,804.3 us.
- Limitations: measures the full pipeline rather than parse only; the host was
  noisy; there is no 0.13 counterpart because a faithful port is a prerequisite
  for measuring it. No performance claim is derived from this number.

**Upstream benchmark (cited, not reproduced).** The chumsky 0.13 crate
documentation publishes a JSON benchmark on one core of an AMD Ryzen 7 3700x:
chumsky check-only 140.77 us (797 MB/s), chumsky 210.43 us (533 MB/s), winnow
178.91 us, serde_json 477.41 us, pest 1.9706 ms. It is an upstream-authored,
generic JSON workload, not an Anti-DSL measurement, and is cited only to show
that the maintained line is not known to be a performance regression.

**Quality gates at risk.** `docs/audits/pre-fase5-fuzzing.md` records a passing
60-second smoke and a pending 24-hour gate (issue #156). `fuzz/fuzz_targets/
fuzz_parser.rs` calls `ag_dsl::lint()` on arbitrary UTF-8 and is the target most
directly affected by a parser rewrite. The 27 parser unit tests mostly assert
"no errors" rather than exact diagnostic text; `diagnostics.rs` tests construct
`Diagnostic` values manually and are parser-independent. That means the current
suite does **not** actually pin diagnostics quality: if a migration is approved,
new golden diagnostics tests are required before claiming that gate still
passes.

## 3. Alternatives considered

### 3.1 Stay on chumsky 0.9 until DSL v1.0 freeze (recommended)

Keep the pinned 0.9.3 and the working parser unchanged. Re-evaluate when the
DSL grammar is frozen at v1.0 or when a concrete blocker appears (advisory,
unfixable diagnostics defect, demonstrable parse bottleneck).

- Pros: zero rewrite; all existing tests, fuzz evidence and golden artifacts
  stay valid; no schedule risk; 0.9.3 has no known advisory; the crate is
  self-contained and could be vendored if maintenance ever becomes a real
  problem.
- Cons: continuing on an unmaintained line; a later migration rewrites a
  larger grammar; no access to newer recovery, caching or Pratt features.
- Chosen as the recommendation because no functional or performance defect of
  0.9 is demonstrated in the current DSL, while the migration cost is concrete
  and the grammar is not yet frozen.

### 3.2 Migrate now to the current 0.13+/1.0-line (0.14.x at implementation time)

Port `parser.rs` rule by rule, keep the AST, and re-run every gate.

- Pros: current maintained API; better error contexts and recovery strategies;
  caching/memoisation and check-only mode available for future LSP work;
  aligns the compiler with the stack used by the rest of the ecosystem.
- Cons: full rewrite of ~1.5k lines; error model and recovery semantics change;
  resets fuzz evidence and golden-artifact verification; needs new diagnostics
  golden tests; no measured benefit for the current grammar.
- Not recommended now, but section 4 gives the complete plan in case the
  committee overrides the recommendation.

### 3.3 Keep chumsky but pin a 0.10/0.11/0.12 stepping stone

- Pros: smaller individual API jumps than 0.9 -> 0.14.
- Cons: 0.11.0 is yanked, 0.10/0.11 are superseded, and the total work is the
  same or larger. Rejected.

### 3.4 Replace the parser (hand-written recursive descent, or lalrpop/pest)

RFC-0003 already rejected these: LALR grammar ambiguity, pest's lack of error
recovery, and the maintenance cost of hand-written recovery. No new evidence
contradicts that decision. Rejected.

## 4. Design (amendment to RFC-0003)

### 4.1 What the amendment changes

If this RFC is accepted with the recommended decision:

- RFC-0003 section 3.2 keeps chumsky 0.9 as the Fase 3 choice; the "future RFC"
  placeholder in sections 6, 8 and 9 is resolved by this RFC with a decision to
  defer.
- RFC-0003 section 9's stack line "Parser: chumsky 0.9 con feature `label`" is
  corrected to "Parser: chumsky 0.9.3, no extra Cargo features (`.labelled()`
  is built in)".
- No public API of `ag-dsl` changes. `compile()`, `lint()` and `Diagnostic`
  keep their signatures. No DSL grammar version changes. No master documents
  change.

If the committee decides to migrate instead, the following design applies and
RFC-0003 sections 3.2, 4.1 and 9 are superseded on the parser line only.

### 4.2 Migration design (conditional)

- **Error type:** adopt `Rich<'a, Token, Span>` (not the 0.13 `Simple`) so the
  existing "expected tokens" hint survives. `parse_error_to_diagnostic()` maps
  `RichReason::ExpectedFound { expected, found }` to the existing message plus
  hint, and `RichReason::Custom` to the existing custom branch. The lost
  `Unclosed` variant is replaced by a labelled delimiter plus a custom error
  emitted when `delimited_by` fails to find its closer; this needs an explicit
  test because the exact message cannot be preserved automatically.
- **Input and spans:** replace
  `Stream::from_iter(end_span, tokens.into_iter())` with
  `Stream::from_iter(tokens.into_iter()).map(end_span, |(t, s): (_, _)| (t, s))`.
  The compiled spike shows this keeps the lexer's `Range<usize>` byte spans.
- **Parser signatures:** add the `'a` lifetime and the input generic
  (`I: ValueInput<'a, Token = Token, Span = Span>`), matching the upstream logos
  example. Keep all `fn xxx_parser()` shapes and the `SchemaItem` union so the
  diff stays mechanical.
- **Combinators:** `.map_with_span()` -> `.map_with()` with `e.span()`;
  `select!` String bindings get `.clone()`; `skip_then_retry_until([...])` is
  re-expressed as `skip_then_retry_until(any().ignored(), choice((...)).ignored())`.
  Recovery must be re-tuned against the existing multi-error tests, not copied
  literally.
- **Public API:** unchanged. `parse_tokens()` still returns
  `(Option<Schema>, Vec<Diagnostic>)`; only its internals change.
- **Dependencies:** workspace `chumsky = "0.9"` -> `chumsky = "0.14"`
  (or newest stable at implementation time), with default features; no new
  direct dependency. `Cargo.lock` is regenerated in the same PR. The
  `result_large_err` allow stays until clippy proves it unnecessary.
- **Tooling/CI:** no workflow change. Add a `criterion` benchmark for
  `ag_dsl::compile()` under `benches/` or `crates/ag-dsl/benches/` before the
  port, so a like-for-like before/after number exists; this is currently
  missing from `docs/benchmarks/`.
- **Documentation:** update `crates/ag-dsl/README.md`, `docs/modules/ag-dsl/`
  (if present) and the RFC-0003 index entry in the same PR. No master document
  change: the DSL version and phase are unchanged.

## 5. Migration plan (conditional, only if the committee overrides)

1. **PR 0 (prerequisite):** add the DSL parse benchmark with Rule 17 metadata
   and record the 0.9 baseline on a quiet host. No parser change.
2. **PR 1:** add golden diagnostics tests that assert message and hint text for
   unexpected token, EOF, unclosed delimiter and multi-error recovery. These
   tests must pass on 0.9 before the port and unchanged on 0.13+.
3. **PR 2:** switch the workspace dependency and port `parse_tokens()` input
   plumbing plus `parse_error_to_diagnostic()`; update only the first grammar
   rule (`config_parser`) as a vertical slice. All tests green.
4. **PR 3-N:** port the remaining grammar rules in small groups (models/fields,
   annotations, request/response/error, endpoints/events, mail/domain/worker),
   keeping all 27 existing tests unchanged.
5. **PR N+1:** re-run fuzz smoke targets (`fuzz_parser`, `fuzz_compile`,
   `fuzz_lexer`) and the 24-hour gate (#156); regenerate
   `examples/ecommerce-api/generated` and require `git diff --exit-code`.
6. **PR N+2:** update docs and CHANGELOG, close #180 with evidence.

Under the recommended "defer" decision none of these PRs are created now.

## 6. Risks

| Risk | Probability | Impact | Mitigation |
| --- | --- | --- | --- |
| Diagnostics regress silently (labels/expected formatting differ) | High if migrating | High (DX) | PR 1 golden diagnostics tests before any port; no test deletion |
| Recovery behaviour changes and multi-error tests fail | Medium if migrating | Medium | Re-tune `skip_then_retry_until` with the existing multi-error tests; never relax assertions |
| Fuzz evidence reset while #156 is still pending | High if migrating | High (release gate) | Deferral avoids it; if migrating, re-run smoke + 24h gate and record new logs |
| Generated artifacts differ byte-for-byte | Low (AST/codegen untouched) | High (issue #180 gate) | `git diff --exit-code examples/ecommerce-api/generated` in CI |
| 0.9.3 becomes a security or maintenance problem | Low today (no advisory) | Medium | Watch `cargo audit`; if an advisory appears, this RFC's deferral is void and PR 1-2 start immediately |
| Target version churn (0.13 named in issue, 0.14.x current) | High if migrating | Low | Pin the newest stable version at implementation time, not the issue's number |
| Spike-to-production gap (small spike, full grammar) | Medium | Medium | Benchmark the real `ag-dsl::compile()` before and after; treat upstream numbers as context only |

## 7. Impact

- **Scope:** none. No new crate, no new feature, no DSL grammar change.
- **Schedule:** deferral adds no work. A migration is a multi-PR rewrite plus a
  24-hour fuzz re-run and is not compatible with the current release gate.
- **Operational complexity:** none under deferral. Migration adds no runtime
  dependency (both are single crates) but adds benchmark and golden-test
  maintenance.
- **Public APIs:** unchanged in both options.
- **Documentation:** this RFC plus the RFC index and CHANGELOG entries. The
  RFC-0003 amendment is explicit so no future reader mistakes 0.9 for a
  deliberate permanent choice.
- **Diagnostics:** unchanged under deferral. Under migration, exact hint text
  changes from `Token` Debug to quoted patterns ('Comma'); golden tests decide
  whether that is acceptable.
- **Fuzzing:** unchanged under deferral. Under migration, all DSL fuzz targets
  must be re-run, including the pending 24-hour gate.
- **Benchmarks:** no DSL parse benchmark exists today; the deferral records a
  single baseline measurement with explicit limitations and recommends adding
  a tracked benchmark before any future migration.

## 8. Rollback

- If deferral is accepted and a blocker appears later: open a successor RFC
  that supersedes this one, then follow the conditional plan in sections 4-5.
- If migration is accepted and fails a gate: revert the workspace pin to
  `chumsky = "0.9"` and restore `parser.rs` from the pre-migration commit. The
  AST, semantic analysis, codegen and public API are untouched, so the rollback
  is a single-revert operation. Restore the previous fuzz logs and benchmark
  baseline as the record; do not claim the gates passed after a reverted port.
- Metrics that trigger rollback: any existing parser test failing, changed
  generated artifacts, new fuzz crash, or a measured parse-time regression
  beyond noise on the tracked benchmark.

## 9. Decision

- Decisor: BDFL (Angel Nereira, project maintainer); recorded by the
  architecture committee chair.
- Decision date: 2026-11-22 (`CLAUDE.md` rule 28).
- Result: ACCEPTED with the recommendation to **DEFER the upgrade. Stay on
  chumsky 0.9.3 until the DSL v1.0 grammar freeze or a concrete blocker
  appears** (ADR-0021). The workspace pin stays `chumsky = "0.9"` and
  `Cargo.lock` stays at 0.9.3. Issue #180 stays open until the board
  reconciles it; no gate is closed by this decision.
- Recommendation of this RFC: **DEFER the upgrade. Stay on chumsky 0.9.3 until
  the DSL v1.0 grammar freeze or a concrete blocker appears.**
- Rationale: 0.9.3 has no known advisory and the parser is working and fully
  tested; no functional limit or measured performance bottleneck of 0.9 is
  demonstrated. The maintained line is a different API that requires a full
  rewrite of ~1.5k lines, changes the error model, and resets the fuzz,
  diagnostics and golden-artifact gates while #156 is still pending. Upgrading
  to an older 0.13.0 would compound the cost because 0.14.x is already stable.
  The upgrade is worth it only when the grammar stops changing, or immediately
  if an advisory or an unfixable diagnostics defect appears.
- Review triggers: (1) DSL v1.0 grammar freeze; (2) any RUSTSEC advisory on
  chumsky 0.9.x; (3) a concrete DX defect that 0.9 cannot express; (4) a
  tracked benchmark showing a parse bottleneck attributable to the parser
  library.
- If the committee decides to migrate now, the target is the newest stable
  0.14.x and sections 4.2 and 5 become the implementation contract.

## 10. References

- Issue #180: https://github.com/Anti-Gravital/Anti-Gravital/issues/180
- Issue #156 (24h fuzz gate): https://github.com/Anti-Gravital/Anti-Gravital/issues/156
- Issue #157 (benchmark evidence): https://github.com/Anti-Gravital/Anti-Gravital/issues/157
- `docs/rfc/RFC-0003-librerias-compilador-ag-dsl.md`, sections 3.2, 4.1, 6, 9
- `crates/ag-dsl/src/parser.rs` (parse entry point, error conversion, tests)
- `crates/ag-dsl/src/diagnostics.rs` (Diagnostic contract)
- `fuzz/fuzz_targets/fuzz_parser.rs`; `docs/audits/pre-fase5-fuzzing.md`
- `Cargo.toml` workspace dependency table; `Cargo.lock` entry for chumsky
- crates.io registry API: https://crates.io/api/v1/crates/chumsky (release
  dates, MSRV, features, line counts; fetched 2026-11-21)
- chumsky 0.13 documentation, features and upstream JSON benchmark:
  https://docs.rs/chumsky/0.13.0/chumsky/
- chumsky 0.13 logos integration example (input/span pattern):
  https://docs.rs/crate/chumsky/0.13.0/source/examples/logos.rs
- chumsky changelog: https://docs.rs/crate/chumsky/0.13.0/source/CHANGELOG.md

## 11. Additive spike evidence (2026-11-22, issue #180)

This section was added after the acceptance decision. It records new evidence
only and does NOT reopen, amend or weaken section 9 or ADR-0021. The decision
remains DEFER; chumsky 0.9.3 stays pinned.

A throwaway spike outside the repository ported one cohesive slice of
`parser.rs` (`config_parser()` + `config_field_parser()`, the `parse_tokens()`
input plumbing, the error-message conversion and a `map_with_span` probe) from
0.9.3 to the current stable 0.14.2, then measured it. Full evidence, verbatim
compiler errors, raw benchmark trials and rule 17 metadata are in
`docs/audits/chumsky-migration-spike.md`.

Three findings refine section 2.3 and section 4.2 without changing the plan:

1. Two additional concrete migration items were confirmed by compilation: in
   0.14 `.repeated()` yields `()` unless `.collect()` is present, and
   `RichPattern: Display` requires `Token: Display` (production `Token` has no
   `Display`). The expected-token hint therefore needs an explicit rendering
   decision, not only a new `match` arm.
2. On the same malformed input, 0.9 produced no expected-token hint while 0.14
   produced `se esperaba uno de: valor de config` from the retained label.
   This is a DX improvement and an exact-text change; it reinforces the
   requirement for golden diagnostics tests before any port.
3. A small, noisy microbenchmark (load 16-17 on 12 logical CPUs, 5 interleaved
   trials per version, 5000 measured iterations each) showed median means
   208.1 us on 0.9.3 and 176.3 us on 0.14.2 for the slice. Standard deviation
   exceeded the mean in individual trials. This is negative evidence against
   an obvious 0.14 slowdown, NOT a performance claim and NOT ADR-0021 trigger
   (d), which requires a tracked `ag-dsl::compile()` benchmark on a quiet host.

Reopening conditions are unchanged and are listed in section 9 and in
`docs/audits/chumsky-migration-spike.md` section 7.
