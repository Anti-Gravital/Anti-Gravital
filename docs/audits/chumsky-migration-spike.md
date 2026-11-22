# Chumsky migration cost spike: 0.9.3 to 0.14.2 (issue #180)

- Date: 2026-11-22
- Branch: `community12/i180` (wave 12)
- Language: English (ADR-0008)
- Scope: read-only measurement and API evidence. The production parser, the
  workspace manifest, the lockfile, the fuzz targets and CI were NOT changed.
  The spike crates live outside the repository and are not committed.
- Outcome: the ADR-0021 deferral is NOT overturned. Issue #180 stays OPEN.
  This document uses `Refs #180`, never `Closes #180`.

## 1. Purpose and method

RFC-0027 and ADR-0021 defer the chumsky upgrade and name four review triggers.
Two of those triggers are factual questions that can be informed by a small
experiment: the real size of the API change (section 4.2 of RFC-0027) and the
performance question (rule 17 of CLAUDE.md). This spike ports ONE cohesive
small unit of `crates/ag-dsl/src/parser.rs` to the current maintained release
and measures it, so the deferral rests on measured cost rather than on an
unverified estimate.

Two throwaway crates were built in `/tmp/opencode/spike-chumsky/`:

- `spike09`: `chumsky = "=0.9.3"` (the pinned production version).
- `spike14`: `chumsky = "=0.14.2"` (the current stable release, published
  2026-11-17; `crates.io` max stable version confirmed 2026-11-22). RFC-0027
  already records that 0.13.0 named in issue #180 is not the right target.

Both crates port the same vertical slice:

1. `config_parser()` + `config_field_parser()` (parser.rs:108-143), simplified
   only in that fields are returned as `Vec<Vec<(String, String)>>` instead of
   being folded into the `Config` AST struct. The grammar shape, combinators,
   labels and error type are unchanged.
2. The `parse_tokens()` input plumbing (parser.rs:30-43): a token vector with
   `Range<usize>` byte spans and an explicit end-of-input span.
3. The `parse_error_to_diagnostic()` message construction (parser.rs:864-899),
   reduced to a `String` because the `Diagnostic` wrapper is irrelevant to the
   migration question.
4. A focused `spanned_ident()` probe that exercises the
   `.map_with_span()` -> `.map_with(|x, e| ... e.span())` change
   (parser.rs:151, 156, 181). It is compiled but not executed.

The token type is identical in both crates and mirrors `crate::lexer::Token`:
a `String`-bearing enum plus `Range<usize>` spans. A small hand-written
tokenizer builds the same input vector in both crates, so lexer cost and token
data are held constant and are not part of the measurement. The malformed
input used for the diagnostics comparison is identical in both crates.

## 2. Verified API differences, 0.9.3 versus 0.14.2

The differences marked as compile errors in section 3 were observed directly
while producing a compiling `spike14`. The two rows that were not exercised by
the spike (`select_ref!` availability and the `result_large_err` allow) are
explicitly marked as source-checked or not re-tested.

| Concern | chumsky 0.9.3 (production) | chumsky 0.14.2 (spike) |
| --- | --- | --- |
| Parser signature | `impl Parser<Token, O, Error = ParseErr>` | `impl Parser<'a, I, O, extra::Err<ParseErr<'a>>>` with `I: ValueInput<'a, Token = Token, Span = Span>`; explicit lifetime and input generic on every parser function |
| Error alias | `type ParseErr = Simple<Token>` | `type ParseErr<'a> = Rich<'a, Token, Span>`; the error is wrapped as `extra::Err<Rich<...>>` in the parser return type |
| Error reason | `SimpleReason::{Unexpected, Unclosed, Custom}` via `err.reason()` | `RichReason::{ExpectedFound, Custom}` via `err.reason()`; there is no `Unclosed` variant |
| Found token | `err.found()` directly on the error | `match err.reason()` then `found.as_deref()`; `err.found()` also exists on `Rich` but grouping with the reason is natural for the port |
| Expected tokens | `err.expected()` of `Option<&Token>`, formatted with `{t:?}` (Debug) | `err.expected()` of `&RichPattern<'a, Token>`; `RichPattern: Display` requires `Token: Display`, so the spike implements `Display` for `Token` to keep Debug-like text. Patterns are labelled (see section 4), not raw token Debug |
| Input construction | `Stream::from_iter(end_span, tokens.into_iter())` | `Stream::from_iter(tokens.into_iter()).map(end_span, \|(t, s)\| (t, s))`; EoI span supplied to `.map()`, not to `from_iter` |
| Recovery entry point | `schema_parser().parse_recovery_verbose(stream)` returns `(Option<O>, Vec<E>)` directly | `config_parser().parse(stream).into_output_errors()`; `parse()` returns `ParseResult` and recovery is configured on the parser, not at the call site |
| `select!` binding | `Token::Ident(s) => s` owns the `String` | `Token::Ident(s) => s.clone()`; with `ValueInput` the binding is by reference. `select_ref!` exists but was not needed |
| `repeated()` output | `repeated()` yields `Vec<O>`; `.collect::<Vec<_>>()` is a no-op-style clarification | `repeated()` yields `()` unless `.collect::<Vec<_>>()` is present; omitting it is a compile error (section 3) |
| Span capture | `.map_with_span(|o, span: Span| ...)` | `.map_with(|o, e| ... e.span())`, with `MapExtra` supplied by the combinator |
| Labels | `.labelled("...")` retained by `Simple` | `.labelled("...")` retained by `Rich`; in the malformed case the label becomes the expected pattern text (section 4) |
| `result_large_err` allow | needed for `Simple<Token>` (128+ byte `HashSet`) | likely still needed for `Rich` (boxed reason plus context `Vec`); not re-tested in the spike because the parser does not return `Result` |

Scale implication: the spike touched roughly 30 lines of parser body plus the
plumbing and error conversion. The production parser has about 50 `select!`
sites, 35 `.ignore_then()`, 46 `.labelled()`, 21 `.delimited_by()` and 2
`.map_with_span()` sites (RFC-0027 section 2.3). Those counts are consistent
with a mechanical but large rewrite; the spike does not reduce that estimate.

## 3. Compile errors encountered and fixed

The first port attempt (`/tmp/opencode/spike-chumsky/compile-14-attempt1.log`,
throwaway) produced two errors. Both are reproduced here verbatim in their
essential parts.

Error 1 - `repeated()` output type:

```
error[E0277]: the trait bound `Repeated<Labelled<_, _>, _, I, _>: Parser<'_, I, _, _>` is not satisfied
   --> src/main.rs:35:30
    |
 35 |   fn config_parser<'a, I>() -> impl Parser<'a, I, Vec<Vec<(String, String)>>, extra::Err<ParseErr<'a>>>
    |                                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsatisfied trait bound
    |
    = note: expected `()`, found `Vec<Vec<(String, String)>>`
```

Cause: in 0.14 `Repeated` implements `Parser<..., ()>` unless a collection is
requested; the 0.9 code relied on `repeated()` returning a `Vec`. Fix: add
`.collect::<Vec<_>>()` after `.repeated()` in `config_parser()`.

Error 2 - expected-token rendering:

```
error[E0277]: `Token` doesn't implement `std::fmt::Display`
   --> src/main.rs:69:65
    |
 69 |     let expected: Vec<_> = err.expected().map(|p| format!("{}", p)).collect();
    |                                                                 --   ^ `Token` cannot be formatted with the default formatter
    |
 help: the trait `std::fmt::Display` is not implemented for `Token`
 help: the trait `std::fmt::Display` is implemented for `RichPattern<'_, T>`
    |     where T: fmt::Display,
```

Cause: 0.9 formatted expected tokens with Debug (`{t:?}`); 0.14 exposes them
as `RichPattern<Token>`, whose `Display` requires `Token: Display`. Fix:
implement `Display for Token` in the spike (delegating to Debug). Production
`Token` has no `Display` today, so a real migration must either add one or
format `RichPattern` with Debug and accept a different hint text. This is a
new, concrete item for RFC-0027 section 4.2: the diagnostics port needs an
explicit token-rendering decision, not just a new match arm.

After those two fixes, `cargo build --release` succeeded for both crates with
`CARGO_INCREMENTAL=0`. A second build after adding the `map_with` probe also
succeeded. No other compile errors occurred.

## 4. Observable diagnostics difference

Both crates parsed the same malformed input
(`config { project_name "x" database 42 retries } config {}`) through the
ported error-message function. Verbatim output:

```
# spike09 (chumsky 0.9.3)
malformed: ast=false diagnostics=["token inesperado: RBrace"]

# spike14 (chumsky 0.14.2)
malformed: ast=false diagnostics=["token inesperado: RBrace | se esperaba uno de: valor de config"]
```

Valid-input assertions passed in both crates (`ast.is_some()`, no diagnostics).

Interpretation: 0.9 `Simple` produced an empty expected set for this error, so
the hint was absent; 0.14 `Rich` preserves the `.labelled("valor de config")`
pattern and emits the hint. This is a DX improvement, but it is also an exact
text change in a function that many tests reach through `parse_tokens`. It
confirms the RFC-0027 finding that golden diagnostics tests must exist BEFORE
a port and that "all tests unchanged" is not a safe acceptance criterion for
diagnostic text. No such golden tests exist today.

## 5. Microbenchmark (rule 17 metadata)

**What is measured.** One parse of a 64-block config corpus
(`config { project_name "shop" database main_db retries 3 } ` repeated 64
times) from an already-built `Vec<(Token, Span)>`. The included work is exactly
the ported slice: combinator construction, parse with recovery and error-map
construction. Excluded: lexing (pre-built tokens), semantic analysis, codegen,
process startup and binary loading. This is a microbenchmark of a slice, NOT
`ag_dsl::compile()` and NOT a release benchmark.

- Hardware: x86_64, 12 logical CPUs, 12th Gen Intel(R) Core(TM) i7-1255U,
  2 threads/core, 10 cores/socket, 1 socket, scaling governor `powersave`,
  CPU scaling 33% at measurement time, max 4700 MHz / min 400 MHz. Shared host
  with other builds; load average 17.00 -> 16.29 during the five trials (more
  runnable work than logical CPUs).
- OS: Linux 7.0.0-44-generic x86_64 GNU/Linux.
- Rust: rustc 1.99.1 (58cf8f9c3 2026-10-13), cargo 1.99.1
  (5f94df478 2026-08-27), host x86_64-unknown-linux-gnu.
- Commit (source of the ported code): `72b97adbdccf8cab3fbd5f411f54343beab40672`
  in the `community12/i180` worktree. Spike lockfiles resolve `chumsky 0.9.3`
  and `chumsky 0.14.2` from crates.io.
- Configuration: `--release`, `debug = false` in both spike manifests,
  `CARGO_INCREMENTAL=0`, no `RUSTFLAGS`, no `lto`, no `codegen-units` override.
  Each trial: 500 warm-up iterations then 5000 measured iterations of the
  parse function; result passed to `std::hint::black_box`; per-iteration
  `Instant` elapsed time converted to microseconds.
- Methodology: ten trials total, five per version, interleaved
  `0.9.3, 0.14.2, 0.9.3, 0.14.2, ...` to reduce systematic drift. Raw output
  is preserved at `/tmp/opencode/spike-chumsky/bench-results.txt` (throwaway).
- Number of executions: 500 warm-up + 5000 measured per binary run; 10 binary
  runs total (50,000 measured parses plus 5,000 warm-up). Standard deviation
  is reported per trial below and aggregated by median.

Raw per-trial results (microseconds):

| Trial | 0.9.3 mean | 0.9.3 std | 0.9.3 p50 | 0.9.3 p95 | 0.14.2 mean | 0.14.2 std | 0.14.2 p50 | 0.14.2 p95 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 212.1 | 292.0 | 178.1 | 273.1 | 213.1 | 75.7 | 207.4 | 251.3 |
| 2 | 245.2 | 46.9 | 248.5 | 288.5 | 188.5 | 303.1 | 170.6 | 214.7 |
| 3 | 191.7 | 54.1 | 212.7 | 247.6 | 125.4 | 40.6 | 98.4 | 186.0 |
| 4 | 208.1 | 53.2 | 209.7 | 253.3 | 137.2 | 42.7 | 139.7 | 194.2 |
| 5 | 204.7 | 47.2 | 202.7 | 218.0 | 176.3 | 83.2 | 172.2 | 193.7 |

Median across trials: 0.9.3 mean 208.1 us (std median 53.2, p50 median 209.7,
p95 median 253.3); 0.14.2 mean 176.3 us (std median 75.7, p50 median 170.6,
p95 median 194.2). Median-of-means ratio 0.9/0.14 = 1.18; median-of-p50 ratio
= 1.23.

**Limitations and honest reading.** The host was overloaded (load 16-17 on 12
logical CPUs) and both versions produced individual trials with a standard
deviation larger than the mean (0.9.3 trial 1: 292 us std at 212 us mean;
0.14.2 trial 2: 303 us std at 188 us mean). The corpus is a tiny synthetic
subset of the DSL, not `examples/ecommerce-api/schema.ag`. The two parsers are
not byte-for-byte equivalent in diagnostics behaviour (section 4), and the
0.14 spike includes a `Display for Token` impl that production lacks. The
ratios above are therefore a hint, not a performance claim: they show no
obvious 0.14 slowdown on this slice, which is useful negative evidence, but
they do NOT establish a speedup for the real parser and do NOT satisfy
ADR-0021 trigger (d), which requires a tracked benchmark showing a
parser-attributable bottleneck in `ag-dsl`. No such tracked benchmark exists.

## 6. What this changes about the deferral

Nothing in the ADR-0021 decision changes. The spike adds three concrete facts:

1. The API cost is real and slightly larger than the already large estimate:
   beyond the documented `parse().into_output_errors()`, `Stream::map`,
   `map_with` and `select!` changes, a migration must decide how expected
   tokens are rendered because `RichPattern` needs `Token: Display` (section 3,
   error 2).
2. The diagnostics behaviour is observably different on the same malformed
   input (section 4). Golden diagnostics tests are a prerequisite, not an
   optional extra; today they do not exist.
3. There is no evidence of a 0.14 performance regression on the measured
   slice, but the measurement is too noisy and too narrow to justify a
   migration. The performance argument for deferral is weakened, not reversed.

The migration remains a multi-PR rewrite of about 1.5k lines that would reset
fuzz and golden-artifact evidence while the 24-hour gate (#156) is still
pending. The deferral is the correct decision with the evidence available.

## 7. Evidence that would justify reopening the decision

Reopening means a successor RFC superseding RFC-0027, as ADR-0021 decision 2
requires. Any one of the following is sufficient grounds to open it:

1. DSL v1.0 grammar freeze (ADR-0021 trigger a): the grammar stops changing,
   so the rewrite cost stops growing.
2. A RUSTSEC advisory against chumsky 0.9.x (trigger b): the deferral is void
   immediately; `cargo audit` currently reports none.
3. A concrete DX defect that 0.9 cannot express (trigger c), demonstrated with
   a failing test against real DSL source. The unclosed-delimiter case and the
   missing expected-token hint seen in this spike are interesting, but neither
   is yet a filed, reproducible DX defect in `ag-dsl`.
4. A tracked `crates/ag-dsl/benches/` benchmark of `ag_dsl::compile()` on a
   quiet reference host, with full rule 17 metadata, that shows a
   parser-attributable bottleneck (trigger d). This spike does not meet that
   bar; it is intentionally small and was run on a contended host.
5. A committee override with a named owner and a plan that adds golden
   diagnostics tests first, records a like-for-like before/after benchmark,
   and schedules the fuzz re-run after #156 is closed.

## 8. Verification performed for this document

- `cargo build --release` for both throwaway crates with `CARGO_INCREMENTAL=0`
  (both succeeded; compiler error logs kept outside the repository).
- Both binaries executed: valid corpus assertions passed in both; malformed
  output captured verbatim in section 4.
- Ten interleaved benchmark trials, raw results preserved outside the repo.
- In the repository worktree: `git diff --check`; content scans for emojis,
  AI-tool attribution, AI session URLs, prohibited descriptor prefixes and
  retired third-party trademark names; descriptor closing-keyword check; and
  `git diff -- Cargo.toml Cargo.lock crates/ag-dsl fuzz .github`, which returned
  no output. Master documents were not touched, so no hash update is needed.

## 9. References

- Issue #180: https://github.com/Anti-Gravital/Anti-Gravital/issues/180
- `docs/rfc/RFC-0027-chumsky-parser-upgrade.md` (sections 2.3, 2.4, 4.2, 5, 9)
- `docs/adr/0021-chumsky-upgrade-deferral.md` (decisions 2 and 3)
- `docs/audits/low-level-review-178-180.md` (section 3)
- `crates/ag-dsl/src/parser.rs` (lines 24-43, 108-143, 151-181, 864-899)
- `fuzz/fuzz_targets/fuzz_parser.rs`, `fuzz/fuzz_targets/fuzz_lexer.rs`
- crates.io registry API (max stable version 0.14.2, fetched 2026-11-22)
