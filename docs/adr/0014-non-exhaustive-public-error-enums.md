# ADR-0014 - `#[non_exhaustive]` policy for public error enums

- Status: accepted
- Date: 2026-06-13
- Source RFC: not applicable (API-stability decision, CLAUDE.md rule 22/31)

## Context

CLAUDE.md rule 31 requires public API stability and backward
compatibility. A public `enum` that is not `#[non_exhaustive]` makes
every newly added variant a breaking change for any downstream crate
that matches the enum exhaustively, because the compiler forces those
`match` expressions to cover every variant.

Error enums are the most likely public enums to grow over time: new
failure modes appear as features land (new backends, new validation
paths, new adapters). Before 1.0 the workspace exposed 27 public error
enums (`AgError`, `AgMailError`, `WebAuthnError`, `StorageError`,
`DataError`, ...) with no uniform policy: `AgMailError` was exhaustive
while a few unrelated non-error enums already carried the attribute.

Adding `#[non_exhaustive]` after 1.0 is itself a breaking change, so the
decision has to be made now, while the public surface can still change
freely.

`#[non_exhaustive]` only constrains *downstream* crates: within the
defining crate, exhaustive `match` and struct/enum construction keep
working unchanged. The cost is therefore paid only by cross-crate
consumers (other workspace crates, examples, templates, and eventually
external users), which must add a wildcard arm.

## Decision

Every public error enum in the workspace is annotated
`#[non_exhaustive]`. "Public error enum" means an `enum` that is part of
a crate's public API and represents an error (returned in a public
`Result`, named `*Error`/`*ErrorKind`, or otherwise surfaced to
callers).

- The attribute is placed directly above the `pub enum` declaration,
  below the `#[derive(...)]` line.
- Internal-only enums (private, or `pub(crate)`) stay exhaustive; the
  attribute adds no value when there is no downstream consumer.
- Non-error public enums are out of scope for this ADR and are decided
  case by case; this ADR mandates the attribute for error enums only.

Downstream `match` expressions on these enums must include a wildcard
arm (`_ => ...`) to remain exhaustive.

## Consequences

Positive:

- New error variants can be added in a minor release without breaking
  downstream `match` code (rule 31).
- One uniform, discoverable rule across the workspace instead of a
  per-crate accident.

Negative / cost:

- Cross-crate consumers (examples, templates, and external users) must
  carry a wildcard arm even when they believe they handle every case;
  the wildcard can mask an unhandled new variant at compile time.
- Constructing these enums by literal is no longer possible outside the
  defining crate; this is acceptable because errors are produced inside
  the crate that defines them, not by callers.

## Alternatives considered

- **Leave enums exhaustive and rely on a major version bump for each new
  variant.** Rejected: it makes routine error-surface growth a breaking
  change and discourages reporting new, more precise failure modes.
- **Apply `#[non_exhaustive]` only to a hand-picked subset of "likely to
  grow" enums.** Rejected: the judgement is unreliable and produces the
  exact per-crate inconsistency this ADR removes; uniform is simpler to
  teach and to audit.

## Notes

- Verification: `grep -rnP "pub enum \w*Error" crates/*/src --include="*.rs" -B1`
  shows `#[non_exhaustive]` above each public error enum;
  `cargo build --workspace --all-targets` and `cargo test --workspace`
  pass with the wildcard arms the attribute requires.
- Related: CLAUDE.md rules 22 (governance), 31 (public APIs), 24
  (error handling).
