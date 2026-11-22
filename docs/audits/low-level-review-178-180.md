# Low-level review: mail protocol stack (#178) and chumsky RFC (#180)

- Date: 2026-11-21
- Reviewer role: low-level software review (community board)
- Method: read-only. No file owned by another role was edited.
- Scope: native MTA DKIM signing, rustls provider selection, the process-level
  ring fix from #181, and the low-level correctness of the proposed chumsky
  upgrade.

## 1. State at review time

The primary owners had no commits beyond `main` on their branches
(`community/dev2` and `community/innovacion` were both clean and at `main`).
dev2's blackboard notes record issue #178 as IN PROGRESS with no files changed
yet, so this review covers:

- the current baseline that #178 must migrate (`ag-mail` still pins
  `mail-send` 0.6, `mail-auth` 0.9, `mail-parser` 0.11);
- the #181 fix already in `main` (commit `f2b9408`);
- the crate sources for the target versions present in the local registry
  (`mail-send` 0.7.0, `mail-auth` 0.15.1, `chumsky` 0.13.0);
- the current parser, fuzz and benchmark infrastructure that #180 must
  preserve.

Findings are therefore conditions and recommendations for the two changes,
not a review of finished diffs.

## 2. Issue #178 - mail protocol stack upgrade

### 2.1 Crypto backend selection is the main migration hazard (blocking)

`mail-auth` 0.15.1 removed the `ring` feature that `ag-mail` currently
requests (`crates/ag-mail/Cargo.toml:94`). Its feature set is:

- `default = ["aws-lc-rs", "dns-hickory"]`
- `rust-crypto = ["rsa", "rand", "ed25519-dalek", "sha1", "sha2", "ml-dsa"]`
- no `ring` feature at all
- a compile-time guard in `src/lib.rs`:
  `compile_error!("a DNS backend is required; enable feature dns-hickory or dns-doh")`

Consequences for the upgrade:

- Keeping `features = ["ring"]` fails resolution exactly as reported in #178.
- Dropping `ring` while keeping `default-features = false` leaves no signing
  backend (compile error or unusable signer).
- Accepting the default features enables `aws-lc-rs`, which reintroduces the
  two-provider condition that #181 fixed. This would silently undo a security
  fix in a dependency bump.

The feature set proposed in dev2's notes is correct:

```toml
mail-auth = { version = "0.15", optional = true, default-features = false,
              features = ["dns-hickory", "rust-crypto"] }
```

`mail-auth` 0.15.1 depends on `hickory-resolver` 0.26, which unifies with the
`hickory-resolver` 0.26.1 already used by `ag-domains`, so the resolver
constraint that motivated the original direct dependency still holds.

`mail-send` 0.7 still exposes `builder`, `ring` and `tls12`, so its current
line (`Cargo.toml:93`) can be kept as-is with only the version bumped.

**Do not enable `mail-send/dkim`.** In 0.7.0 the `dkim` feature pulls
`mail-auth = "0.14"`. Enabling it would compile two major versions of
`mail-auth` side by side (0.14 transitively, 0.15 directly) and reintroduce
the hickory version pressure that the comment at `Cargo.toml:88-91` documents.
The existing design - direct `mail-auth` signing, `mail-send/dkim` off - is
still the right one.

### 2.2 DKIM signing semantics

The signing call sites in
`crates/ag-mail/src/sender/mta/dkim.rs` survive the 0.9 to 0.15 move with
import-path changes only:

- `mail_auth::common::crypto` -> `mail_auth::crypto` (0.15 `src/lib.rs:19`
  declares `pub mod crypto`); `Ed25519Key::from_pkcs8_der`,
  `Ed25519Key::generate_pkcs8`, `RsaKey::<Sha256>::from_key_der` and
  `RsaKey::from_pkcs8_der` are present in `crypto/rust_crypto.rs`.
- `mail_auth::common::headers::HeaderWriter` ->
  `mail_auth::headers::HeaderWriter`; `Signature::write_header` is still the
  documented way to prepend the header.
- `DkimSigner::from_key().domain().selector().headers().sign()` is unchanged
  (0.15 uses a type-state builder, but the chain used by `sign_with` at
  `dkim.rs:150-156` is still valid).

Behavioural details that need explicit attention:

1. **Canonicalization is implicit today.** `sign_with` sets only domain,
   selector and headers. 0.15 adds explicit
   `header_canonicalization()` / `body_canonicalization()` setters and keeps a
   default. A default change in a future release would silently invalidate
   every signature. Recommendation: set relaxed/relaxed explicitly in
   `sign_with` so the wire format is a documented contract.

2. **Absent signed headers.** 0.15 documents that headers listed in `h=` but
   absent from the message are signed as empty (RFC 6376 section 5.4), and
   `sign()` returns `NoHeadersFound` only when the list itself is empty.
   `SIGNED_HEADERS` (`dkim.rs:25`) includes `Date` and `Message-ID`, which the
   `MessageBuilder` in `mta/mod.rs::render` does not emit. This is valid DKIM,
   but it should be stated in the migration notes so nobody "fixes" it by
   removing the entries and weakening the signature.

3. **`sign()` is time-dependent.** 0.15 sets `t=` from `SystemTime::now()`
   inside `sign`, and there is no public timestamp setter
   (`sign_stream(now)` is `pub(crate)`). A byte-for-byte known-answer test is
   therefore not directly possible; a KAT must normalize `t=` and `b=` or use
   an external known vector.

4. **Test gap.** The current DKIM tests (`dkim.rs` `tests` module) assert only
   that the output starts with `DKIM-Signature:` and contains `d=`, `s=` and
   `a=`. They do not verify any signature. Issue #178 explicitly requires an
   independent verifier in the Done-when list.

### 2.3 The #181 process-level ring provider fix

Commit `f2b9408` adds `rustls` to the `mta` feature and installs
`ring::default_provider()` as the process-wide default from
`MtaSender::new()` (`mta/mod.rs:137-146`), ignoring the result of
`install_default`. The verification grep from #181 is clean: there is no
`ClientConfig::builder()` or `ServerConfig::builder()` in `crates/`; all
workspace configs use `builder_with_provider`.

Assessment:

- It is a correct and surprisingly small fix for `mail-send` 0.6, whose
  `build_tls_connector` uses `ClientConfig::builder()` (the process default)
  both for verified and for `allow_invalid_certs()` connectors.
- It is a process-global side effect from a library constructor. A caller
  that installs a different provider *after* constructing an `MtaSender`
  silently fails to do so. This is a design smell, not a current functional
  failure, and #181's plan preferred explicit provider configuration.
- Because the `Result` is discarded, if another crate installed a provider
  first (for example `aws-lc-rs`), mail-send uses that provider and the
  comment "the workspace standardizes on ring" is inaccurate. The no-panic
  property still holds.

**Trap for the #178 upgrade.** `mail-send` 0.7 adds
`SmtpClientBuilder::new_with_tls_connector(hostname, port, TlsConnector)`,
documented as not requiring a process-level `CryptoProvider`. Replacing
`SmtpClientBuilder::new` with it looks like the clean way to remove
`install_ring_provider()`. However `allow_invalid_certs()` in 0.7 still
executes `self.tls_connector = build_tls_connector(true).unwrap()`, which
rebuilds from `ClientConfig::builder()` and **overwrites the caller's
connector**. `ag-mail` calls `allow_invalid_certs()` for opportunistic TLS at
`mta/mod.rs:276`, so switching constructors alone would not remove the
process-default dependency. Any attempt to drop `install_ring_provider()`
must first provide an explicit ring connector with its own verifier for the
opportunistic-TLS path, and must re-run
`cargo test --workspace --all-features`.

Recommendation: keep the #181 fix as a transitional fallback during the 0.7
migration; remove it only in a separate, verified step that configures the
connector explicitly.

### 2.4 Observed implementation state (2026-11-21)

A read-only check of dev2's worktree found an uncommitted implementation of
the above: `mail-send` 0.7 / `mail-auth` 0.15 /
`mail-parser` 1.0 with `default-features = false`,
`["dns-hickory", "rust-crypto"]`, `mail-send/dkim` still off, the import
moves applied, and `MimeHeaders`/`text_contents()` replaced by
`parts()`/`text().into_owned()`. The resolved lockfile contains a single
`mail-auth` 0.15.1 with the pure-Rust backend and no `aws-lc-rs` in its
dependency list. The `Sha256`, key constructors and parser APIs used by the
migrated call sites are all present in the target crate versions, so no
compile-level defect was found in the observed diff.

Two acceptance gaps remain in that diff: no independent DKIM verifier or
external vector was added (Done-when item 2 of #178), and canonicalization is
still left to the library default. These are recommendations, not a new
defect issue; the primary owner is still working on the change.

## 3. Issue #180 - chumsky 0.9 to 0.13 RFC

### 3.1 Parser and error-recovery compatibility (blocking if "tests unchanged")

`crates/ag-dsl/src/parser.rs` (1482 lines) enters the parser through
`Stream::from_iter` plus `parse_recovery_verbose` (`:36-38`), aliases
`type ParseErr = Simple<Token>` (`:24`), and maps errors through
`chumsky::error::SimpleReason` (`:864-899`).

In chumsky 0.13:

- `parse_recovery_verbose` no longer exists. `Parser::parse` returns
  `ParseResult<O, E::Error>` with `output()` / `into_errors()`. The
  `(Option<Schema>, Vec<Diagnostic>)` shape of `parse_tokens` must be
  reconstructed from that type.
- `Simple` is now `Simple<'a, T, S = SimpleSpan<usize>>`, so the current alias
  does not compile as written.
- The recovery combinators used by the grammar are still available:
  `recover_with`, `skip_then_retry_until`, `skip_until`, `nested_delimiters`,
  `via_parser` (0.13 `src/lib.rs:79`, `:1930`). The grammar-level recovery
  strategy can therefore be ported rule by rule.

Issue #180 requires all `ag-dsl` tests to pass unchanged and generated
artifacts to be byte-identical. The exact diagnostic text produced by
`parse_error_to_diagnostic` is part of that contract; the port must preserve
it even though the error type changes. The many `parse_tokens` call sites in
`semantic.rs`, `codegen/*` and `parser.rs` should not need edits if the public
signature is held constant.

### 3.2 Performance claims have no measurement basis

There is no DSL parser benchmark in the repository.
`benchmarks/README.md:26-31` lists only `ag-core/benches/shield_hello_world.rs`
and `ag-workers/benches/queue_throughput.rs`. Issue #180 asks for the
"measured parse-time difference on the example schemas" as an RFC input;
that number does not currently exist.

Recommendation: add a criterion bench under `crates/ag-dsl/benches/` using a
fixed corpus (at minimum `examples/ecommerce-api/schema.ag` plus a malformed
variant), measure before and after the port, and publish the figures with the
full rule 17 metadata (hardware, OS, Rust version, commit, configuration,
methodology, number of runs, standard deviation). Qualitative "faster" or
"slower" statements without that metadata should not enter the RFC.

### 3.3 Fuzz compatibility and gate accuracy

The fuzz targets call only public high-level APIs (`ag_dsl::lint`,
`compile`, `generate`), so a rewrite internal to `parser.rs` is
source-compatible with `fuzz/fuzz_targets/fuzz_parser.rs` and
`fuzz_compile.rs`.

Two accuracy problems in the issue/RFC inputs:

- No fuzz corpus directory exists (`fuzz/` contains only targets and lock
  files). There is nothing to replay automatically after the port.
- `docs/audits/pre-fase5-fuzzing.md` section 4.4 records 60-second smoke
  results only for `fuzz_resp`, `fuzz_storage_key` and `fuzz_signed_url`.
  Although section 4.1 says CI smokes `fuzz_parser` and `fuzz_lexer`, no
  result for those two targets is recorded. The 24-hour gate in section 4.5
  is explicitly **pending**, not passed.

Therefore #180's statement that a rewrite "would reset" the 24-hour fuzz
gate is misleading: the gate was never completed. Recommendation: record a
60-second baseline for `fuzz_parser` and `fuzz_lexer` before the rewrite,
re-run the identical commands after it, and keep the 24-hour row `pending`
until it is actually executed.

The fuzz workspace pins `nightly`; chumsky 0.13 default features (`std`,
`stacker`) are stable-compatible, so the parser port does not force an
unstable chumsky feature.

### 3.4 MSRV and version choice

chumsky 0.13.0 declares `rust-version = 1.65`, below the workspace MSRV
1.99.1 (`rust-toolchain.toml`, `Cargo.toml`). MSRV is not a blocker. The
latest published line is `1.0.0-alpha.8`; the RFC should explicitly justify
stopping at 0.13 rather than adopting an alpha, in line with the project's
stability policy.

## 4. Direct answers to dev2's questions (blackboard, 2026-11-21)

1. *Does a cross-version known-answer test (0.9 signer, 0.15 verifier) count
   as an independent verifier?* No. Both versions share canonicalization,
   hashing and header-writing lineage; such a test is a valuable migration
   regression test but not independent evidence of DKIM correctness. The
   acceptance criterion in #178 calls for a separate implementation or a
   recorded vector from one (for example an external DKIM verifier, or a
   fixed signature vector generated outside `mail-auth`). The current
   substring assertions do not satisfy it either.

2. *Is PARTIAL with `Refs #178` acceptable?* For the manual real-mailbox /
   Mail A gate, yes - that is the honest status while port 25 egress is
   blocked, and the board rule says a blocked issue stays open with evidence.
   The independent verifier is different: it is a Done-when checkbox. If it
   is genuinely unavailable, keep #178 open with `Refs #178` and the blocker
   documented, but prefer to include it in the same pull request because it
   is the core correctness gate for a cryptographic change.

## 5. Findings summary

| # | Area | Severity | Recommendation | Verification |
| --- | --- | --- | --- | --- |
| 1 | #178 crypto features | Blocking | `mail-auth` 0.15 with `default-features = false`, `["dns-hickory", "rust-crypto"]`; never enable `mail-send/dkim` | `cargo tree -p ag-mail --features mta -e features` shows one mail-auth major and no `aws-lc-rs` from `mail-auth`; `cargo test --workspace --all-features` |
| 2 | #178 DKIM verification | Blocking | Add independent verification or an external known vector; pin header/body canonicalization explicitly | #178 Done-when item 2; a verifier outside `mail-auth` accepts a produced signature |
| 3 | #181 provider fix | Non-blocking | Keep during migration; do not call `new_with_tls_connector` while `allow_invalid_certs()` still overwrites the connector | `cargo test --workspace --all-features`; search shows no implicit process-default config in `crates/` |
| 4 | #180 parser port | Blocking for "tests unchanged" | Preserve `parse_tokens` signature and diagnostic text; replace `parse_recovery_verbose` with `parse()`/`ParseResult` | `cargo test -p ag-dsl`; generated diff on `examples/ecommerce-api` |
| 5 | #180 performance | Non-blocking | Add a criterion bench and record rule 17 metadata before making performance claims | Bench output with hardware, OS, Rust version, commit, runs, std dev |
| 6 | #180 fuzz/gates | Non-blocking | Record parser fuzz baseline; correct "reset" to "never completed" for the 24h gate | 60s `fuzz_parser`/`fuzz_lexer` results before and after |
| 7 | #180 MSRV | Informational | 0.13 MSRV 1.65 is below workspace 1.99.1; justify 0.13 over 1.0.0-alpha.8 | RFC decision section |

## 6. Verification performed for this review

- Read-only inspection of `main` at `0c4d808` and of the target crate sources
  in the local registry (`mail-auth` 0.15.1, `mail-send` 0.7.0, `chumsky`
  0.13.0).
- `grep` for implicit rustls builders across `crates/` returned no matches.
- Baseline tests passed in this worktree with the shared `CARGO_TARGET_DIR`:
  `cargo test -p ag-mail --all-features --lib` gave 128 passed, 0 failed,
  1 ignored; `cargo test -p ag-dsl --lib` gave 165 passed, 0 failed,
  0 ignored. Combined exit code 0.

No source file owned by dev2 or innovacion was modified.
