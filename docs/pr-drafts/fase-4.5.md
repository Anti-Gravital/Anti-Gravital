# feat(fase-4.5): ag-mail + ag-domains — correo transaccional y gestion DNS

## Fase afectada

Fase 4.5 — Crates de correo transaccional y gestion DNS (ADR-0007)

## Tipo de cambio

Nuevos crates funcionales (`feat`): ag-mail, ag-domains. Extension de ag-auth,
ag-dsl, ag-cli y tests de integracion.

## Documentos relacionados

- `docs/adr/0007-ag-mail-ag-domains.md` — decision arquitectonica
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` secciones 8.7, 8.8
- `docs/master/ANTI-GRAVITAL-Hoja-de-Ruta.md` seccion Fase 4.5
- `docs/modules/ag-mail.md` — especificacion del modulo
- `docs/modules/ag-domains.md` — especificacion del modulo
- `docs/dsl/v0.7.md` — bloques mail/domain en el DSL

## Resumen

Implementacion completa de Fase 4.5 (Etapas 2-1 a 2-12). Agrega correo
transaccional outbound y gestion DNS via traits con adapters intercambiables.

**ag-mail** (`crates/ag-mail`):
- `MailSender` trait + `SmtpSender` (lettre + rustls) (relay nativo)
- `InMemoryQueue` con reintentos exponenciales y worker tokio
- `StringTemplate` con sustitucion `{{var}}`
- `NullSender` (feature test-utils) para tests cross-crate
- 38 tests

**ag-domains** (`crates/ag-domains`):
- `DnsProvider` trait + `CloudflareProvider` (reqwest)
- `apply_mail_records`: upsert idempotente de SPF/DKIM/DMARC
- `PropagationChecker`: verificacion multi-resolver con hickory-dns
- ACME skeleton
- 24 tests

**ag-dsl v0.7** (`crates/ag-dsl`):
- Tokens `mail`, `domain`, `template` en el lexer
- Nodos `MailBlock`, `DomainBlock`, `MailTemplateDef` en el AST
- Parsers y analisis semantico para los nuevos bloques
- 151 tests totales

**ag-auth** (feature `mail`):
- `AuthMailer` con inyeccion de `Arc<dyn MailSender>`
- `AgAuth::with_mail()` builder
- Tres operaciones: send_verification, send_password_reset, send_magic_link
- 37 tests (32 sin feature, 37 con feature mail)

**ag-cli**:
- `ag domains check` — propagacion TXT via resolvers publicos
- `ag domains sync` — aplica registros SPF/DKIM/DMARC
- `ag mail test` — envia correo de prueba via SMTP

**examples/auth-mail-demo**:
- Ejemplo ejecutable de los tres flujos de correo con NullSender

**tests/integration**:
- 7 tests E2E en `fase45_e2e.rs` (mail, domains, auth+mail)
- Total: 14 tests de integracion (7 Fase 4 + 7 Fase 4.5)

## Plan de prueba

- `cargo test --workspace` — todos los tests pasan
- `cargo test -p ag-auth --features mail` — 37 tests, 0 fallos
- `cargo test -p ag-integration-tests` — 14 tests E2E, 0 fallos
- `cargo run -p auth-mail-demo` — produce salida correcta sin SMTP real
- `cargo clippy --workspace` — 0 errores
- `cargo fmt --all -- --check` — sin cambios pendientes

## Criterios de salida avanzados

- ag-mail operativo con relay SMTP nativo
- ag-domains con upsert idempotente SPF/DKIM/DMARC
- DSL v0.7 con bloques mail/domain parseados y validados
- ag-auth puede enviar los tres tipos de correo de autenticacion
- CLI expone ag domains y ag mail como subcomandos
- Ejemplo ejecutable sin dependencias externas
- 14 tests E2E cross-module pasan

## Checklist final

- [x] Pertenece a la fase correcta (4.5 segun ADR-0007)
- [x] Respeta la documentacion (Arquitectura Tecnica, Hoja de Ruta)
- [x] No rompe arquitectura (no circular deps, no ag-mail->ag-auth)
- [x] No anade complejidad innecesaria
- [x] No crea dependencias circulares
- [x] Compila en dev y release
- [x] Pasa tests (workspace: 0 fallos)
- [x] Pasa fmt
- [x] Pasa clippy
- [x] Tiene documentacion (docs/modules, docs/dsl, docs/adr)
- [x] Tiene manejo de errores correcto (AgMailError, AgDomainsError)
- [x] Mantiene coherencia con Anti-Gravital v4.0
