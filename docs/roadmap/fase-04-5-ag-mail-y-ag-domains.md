# Fase 4.5 - ag-mail y ag-domains: comunicacion y dominios

> Fuente verbatim: docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md
> Indice: [docs/roadmap/README.md](./README.md)
> Anterior: [fase-04-modulos-estandar.md](./fase-04-modulos-estandar.md)
> Siguiente: [fase-05-ag-cloud.md](./fase-05-ag-cloud.md)

## Phase 4.5 — `ag-mail` + `ag-domains`: communication and domains

**Status: Technical implementation complete (2026-05-24).**

**Objective.** Add operational capabilities for transactional communication, DNS,
TLS and domains without overloading Phase 4 nor delaying the standard modules.
It prepares the ground so that `ag-cloud` (Phase 5) deploys applications with
domain, certificate and transactional email using an integrated
experience. The introduction of this phase is made official in `ADR-0007`.

**Duration:** 1–2 months.

### 4.5.1 Entry criteria

- [x] Phase 4 completed with all of its exit criteria checked.
- [x] `ag-auth` exposes hooks/events for email verification, password
  recovery and magic links.
- [x] `ag-observe` records metrics and traces of asynchronous jobs.
- [x] RFC approved for the initial scope of `ag-mail`. See RFC-0006.
- [x] RFC approved for the initial scope of `ag-domains`. See RFC-0007.

### 4.5.2 Deliverables

- [x] `ag-mail` crate (deferred standard): `MailSender` trait + `SmtpSender`
  (`lettre` + `rustls`). 38 tests.
- [x] HTML/plaintext templates: `MailTemplate` trait + `StringTemplate` with
  `{{var}}` substitution. External engines (askama, minijinja) integrable via
  trait. Compile-time var validation via `template::validate`.
- [x] Email declaration in `schema.ag` (`mail` block). DSL v0.7.
- [x] `ag-auth` → `ag-mail` integration for verification, recovery and
  magic links. `AuthMailer` with `"mail"` feature.
- [x] Asynchronous queue with retries and exponential backoff. `InMemoryQueue`
  backend. Persistent backend via `ag-data` deferred (TECH-DEBT documented).
- [x] Metrics towards `ag-observe`: `ag_mail_sent_total`, `ag_mail_retry_total`,
  `ag_mail_send_latency_seconds` (feature `"metrics"`).
- [x] `ag-domains` crate (optional infra): `DnsProvider` trait with Cloudflare
  adapter; declarative A/AAAA/CNAME/TXT/MX model. 28 tests.
- [x] ACME support (Let's Encrypt) via `instant-acme`: `issue()` +
  `issue_with_credentials()` + `spawn_renewal_task()`. DNS-01 challenge.
  TECH-DEBT: `notAfter` parsing for exact renewal.
- [x] Generation of SPF/DKIM/DMARC required by `ag-mail`. `apply_mail_records`
  idempotent (`ag-mail` ↔ `ag-domains` cooperation without dependency cycle).
- [x] Propagation verification against multiple public resolvers
  (`hickory-resolver`). `PropagationChecker` + `DEFAULT_RESOLVERS`.
- [x] DSL v0.7: `mail`, `domain`, `template` blocks; compiler validates that
  `from` references a declared `domain`, provider is valid, vars exist in
  templates, and DMARC policy is valid.
- [x] Update of the `ag-lsp` LSP for the new blocks: hover and completions for
  `mail`/`domain`/`template` and their 7 properties.
- [x] CLI commands: `ag domains check`, `ag domains sync`, `ag mail test`.
- [x] `auth-mail-demo` example in `examples/`: three flows with `NullSender`.
- [x] Documentation: "Configure domain, TLS and transactional email with
  Anti-Gravital". See `docs/manual/03-dominio-tls-correo.md`.

### 4.5.3 Exit criteria (gate before Phase 5)

- [x] `ag-mail` sends transactional HTML and plaintext email from an
  Anti-Gravital project via the native sender **and** via at least one adapter.
- [x] `ag-auth` uses `ag-mail` for email verification and password
  recovery in the `auth-mail-demo` example.
- [x] `ag-domains` implements functional `CloudflareProvider` with contract tests.
- [x] `ag-domains` issues and renews TLS certificates via ACME (Let's Encrypt
  staging/production).
- [x] `ag-domains` generates SPF/DKIM/DMARC required by `ag-mail`.
- [x] `ag domains check`, `ag domains sync` and `ag mail test` compile and pass CI.
- [x] 14 cross-module E2E tests in `tests/integration` (7 Phase 4 + 7 Phase 4.5).
- [x] Zero circular dependencies (green CI job).
- [x] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit` and
  `cargo deny check` green.

### 4.5.4 Phase risks

The main risk is **confusing `ag-mail` with a complete MTA**. The
mitigation is the explicit restriction of the v1 scope to outbound + adapters;
inbound, IMAP/POP, persistent mailboxes and antispam remain documented as
out of scope, not as "deferred to v2".

The second risk is the **dependency on young upstreams** (`instant-acme`,
`hickory-resolver`) in domains where bugs are paid for dearly: a certificate
that does not renew brings down the site. The mitigation is a small
and versioned `DnsProvider` trait with contract tests, explicit pinning in the
workspace, and active monitoring of the evolution of the crates.

The third risk is **turning Anti-Gravital into a hosting panel** by
accumulation of capabilities. The mitigation is the project's interoperability
rule: both crates are abstractions with adapters, not replacements for
providers. The boundary is fixed in `ADR-0007` and does not move without a new
ADR.

### 4.5.5 Forward note — Phase 4.6 native MTA (`ADR-0010`)

That `ADR-0007` boundary has now moved, via the new ADR it required.
`ADR-0010` (2026-06-03) supersedes the v1 "NOT an MTA / inbound never"
restriction and expands `ag-mail` into a native outbound MTA, phased and
opt-in behind Cargo features, preserving the Native | Adapter pattern and the
implemented Phase 4.5 baseline. The work is phased Phase 4.6 (`RFC-0009`
section 5: stages A-D) plus continuous deliverability hardening in Phase 5+.
Phase 4.6-A (the native MTA core: MX resolution, ESMTP+STARTTLS delivery,
Ed25519 DKIM signing and bounce classification) is implemented behind the
opt-in `mta` Cargo feature; stages 4.6-B..D remain forward work. Phase 4.5
stays complete for its original outbound-relay scope. The provider adapters
remain a supported production path until native deliverability is proven.

---

