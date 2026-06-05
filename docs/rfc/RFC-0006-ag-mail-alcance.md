# RFC-0006: ag-mail — alcance, stack tecnico y plan de implementacion

- Estado: aceptada; superseded para el alcance de `ag-mail` por `RFC-0009`
  (`ADR-0010`, 2026-06-03). El relay outbound que esta RFC fijo se conserva
  como subconjunto; el motor MTA nativo se especifica en `RFC-0009`.
- Autor: Angel Nereira (BDFL), Gravital Labs
- Fecha de borrador: 2026-05-24
- Fase objetivo: Fase 4.5
- Modulos o crates afectados: `ag-mail` (nuevo), `ag-auth` (consumidor), `ag-dsl` (v0.7), `ag-observe` (metricas)
- RFC predecesora: ninguna
- ADR gobernante: `docs/adr/0007-ag-mail-ag-domains.md` (Aprobado 2026-05-23)
- Periodo de comentarios: omitido por decision del BDFL (este RFC formaliza el detalle tecnico de un ADR ya aprobado por la regla 28 del CLAUDE.md, que exige RFC antes de introducir nuevos crates).

## 1. Motivacion

`ADR-0007` decidio introducir `ag-mail` como modulo estandar diferido para
cerrar la promesa schema-first sobre correo transaccional. Este RFC concreta
la decision en un plan de implementacion verificable: que librerias se usan,
que modulos componen el crate, como se organizan los tests y como se integra
con `ag-auth` sin crear ciclos.

Sin este RFC, la implementacion arrancaria con decisiones tecnicas ad-hoc
que contradicen la regla 28 del CLAUDE.md ("se crea RFC antes de introducir
nuevos crates").

## 2. Problema

El ADR define el **que** (alcance, restricciones, direccionalidad de
dependencias) pero NO el **como** (librerias concretas, layout de modulos,
patron Native/Adapter, estrategia de tests con red real vs mocks, integracion
con la cola opcional persistente, validacion build-time de templates desde
el DSL). Sin acuerdo previo sobre estos detalles, el implementador improvisa
y la revision del PR se vuelve cara.

## 3. Alternativas consideradas

### A. Implementar todo en un solo modulo `lib.rs` con builder patron

Ventaja: arranque rapido, menos boilerplate.

Inconveniente: el ADR distingue claramente entre sender (interaccion con
proveedor), template (validacion build-time), queue (reintentos) y metrics.
Aglomerar en `lib.rs` borra esa separacion y dificulta tests por capa.

### B. Crate monolitico sin features de Cargo para adapters

Ventaja: compila siempre todo, menos configuracion.

Inconveniente: cualquier proyecto que use el sender SMTP nativo paga el
binario de `reqwest` aunque no use los adaptadores HTTP. La regla 13 del
CLAUDE.md (simplicidad operacional) y el ADR (adapters como features de
Cargo) imponen aislar adapters opcionales.

### C (elegida). Estructura modular con `MailSender` trait + Native | Adapter

Modulos separados por responsabilidad. Adapters como features de Cargo.
Patron Native | Adapter espejo de `ag-storage` (Native | S3) y `ag-cache`
(moka | Redis). Tests por capa con strategy de mocks para HTTP y servidor
SMTP local (Mailpit/MailHog) para integracion.

### D. Diferir el sender SMTP nativo a una iteracion posterior, arrancar solo con un adaptador HTTP de proveedor

Ventaja: time-to-market mas rapido.

Inconveniente: contradice el ADR que exige sender nativo en v1 para
proyectos que no quieren depender de un proveedor externo. La Auditoria
Critica (~/Downloads/Anti-Gravital-Auditoria-Critica.md) recomendo
exactamente esto, pero el BDFL decidio mantener ambos para preservar la
narrativa "Anti-Gravital como capa autocontenida".

## 4. Diseno propuesto

### 4.1 Layout del crate

```
crates/ag-mail/
|- Cargo.toml
|- README.md
|- src/
|  |- lib.rs                 # AgMail (enum Native | Adapter), builder publico, re-exports
|  |- error.rs               # AgMailError + From para reqwest/lettre
|  |- message.rs             # Email, EmailBuilder, Address, Attachment, MessageId
|  |- sender/
|  |  |- mod.rs              # trait MailSender + DnsRecordSpec
|  |  |- smtp.rs             # SmtpSender nativo (lettre+rustls)         [feature "smtp"]
|  |  |- (adaptadores de proveedor retirados por ADR-0011)
|  |- template/
|  |  |- mod.rs              # render askama, contexto tipado
|  |  |- validate.rs         # validacion build-time vars↔template (helper para ag-dsl)
|  |- queue/
|  |  |- mod.rs              # cola async, reintentos, backoff exponencial
|  |  |- store.rs            # persistencia opcional via ag-data         [feature "queue-persistent"]
|  |- metrics.rs             # contadores/histogramas hacia ag-observe
|- tests/
   |- integration.rs         # tests con Mailpit local (CI)               [feature "smtp"]
```

### 4.2 API publica minima (v0)

```rust
#[async_trait::async_trait]
pub trait MailSender: Send + Sync {
    async fn send(&self, msg: &Email) -> Result<MessageId, AgMailError>;
    fn provider_name(&self) -> &'static str;
    fn dns_requirements(&self, domain: &str) -> Vec<DnsRecordSpec>;
}

pub enum AgMail {
    Native(SmtpSender),
    Adapter(Box<dyn MailSender>),
}

impl AgMail {
    pub async fn send(&self, msg: &Email) -> Result<MessageId, AgMailError>;
}
```

`DnsRecordSpec` es un struct pequeno con `record_type`, `name`, `value`,
`ttl`. Vive en `ag-mail::sender` porque conceptualmente pertenece a la
declaracion del proveedor, no a la infraestructura. `ag-domains` lo
consumira como tipo opaco sin depender de `ag-mail` (la cooperacion es via
trait, no via dato compartido — `ag-domains` define su propia conversion).

### 4.3 Dependencias nuevas (con feature gates)

| Crate | Version | Feature | Justificacion |
|---|---|---|---|
| `lettre` | 0.11 | `smtp` (default) | SMTP transport async Tokio, TLS via rustls. Maduro, mantenido. |
| `askama` | 0.12 | `templates` (default) | Reusado de `ag-ui`, mismo build-time templating. |
| (sin deps HTTP de adaptadores; los adaptadores de marca fueron retirados por ADR-0011) |
| `tokio` | (workspace) | always | Runtime. |
| `tracing` | (workspace) | always | Spans para observabilidad. |
| `metrics` | (workspace) | `metrics` (default) | Contadores e histogramas. |

Tests con Mailpit como dev-dependency via `testcontainers-modules` cuando
exista soporte; mientras tanto, container manual en CI.

### 4.4 Cambios en CI o tooling

- Nuevo job `mail-smoke` en `ci.yml` que arranca Mailpit (`axllent/mailpit`)
  via service container y corre `cargo test -p ag-mail --features smtp`
  contra `127.0.0.1:1025`.
- Tests con red real contra proveedores externos quedan `#[ignore]` con
  doc-comment explicando env vars necesarias. CI con secrets corre
  `cargo test -p ag-mail -- --ignored --include-ignored` en push a `main`.

### 4.5 Cambios en DSL (v0.7)

Bloque `mail` en `ag-dsl` v0.7 (detalle en RFC-0007 § ag-domains y en
`docs/dsl/versionado.md`):

```ag
mail WelcomeEmail {
    from "hello@plenty.market"     # debe referenciar un bloque domain
    subject "Welcome to Plenty"
    template "emails/welcome.html"
    vars { name String activation_url String }
}
```

El validador semantico de `ag-dsl` invoca el helper
`ag_mail::template::validate::check(template_path, declared_vars)` para
confirmar que las variables del HTML coinciden con las declaradas.

### 4.6 Cambios en documentacion maestra

Ninguno adicional: `ADR-0007` ya actualizo Arquitectura-Tecnica §8.8 y
Hoja-de-Ruta Fase 4.5. Este RFC complementa el ADR con el detalle de
implementacion.

## 5. Plan de implementacion

PRs en orden:

1. **PR Etapa 2-1**: workspace + skeleton de `ag-mail` y `ag-domains`.
   Mismo PR para ambos crates porque comparten el cambio en `Cargo.toml`.
2. **PR Etapa 2-5**: `ag-mail` message + sender SMTP + templates.
3. **PR Etapa 2-6**: cola + adaptador HTTP de proveedor + metricas.
4. **PR Etapa 2-7**: cooperacion mail↔domains (SPF/DKIM/DMARC).
5. **PR Etapa 2-8**: DSL v0.7 (bloques mail, domain).
6. **PR Etapa 2-9**: CLI (`ag mail test`).
7. **PR Etapa 2-10**: integracion `ag-auth → ag-mail`.
8. **PR Etapa 2-11**: example `auth-mail-demo`.

Cada PR exige `cargo fmt`, `cargo clippy -D warnings`, `cargo test`,
`cargo audit`, `cargo deny check` verdes antes de mergear.

## 6. Riesgos

| Riesgo | Probabilidad | Impacto | Mitigacion |
|---|---|---|---|
| `lettre` upstream poco mantenido | Baja | Alto (cert no llega) | Vigilancia activa de issues; trait `MailSender` permite swap. |
| Validacion build-time de templates falsamente estricta o laxa | Media | Medio (DX o produccion) | Tests del helper `validate::check` con casos canonicos; opt-in via flag CLI. |
| Confusion con un MTA completo | Media | Bajo (reputacion) | README, errores y CLI rechazan explicitamente operaciones inbound. |
| Entregabilidad pobre desde sender nativo en IPs nuevas | Alta | Alto (correo no llega) | Documentar en README; recomendar adapter para produccion. |

## 7. Impacto

- **Alcance del producto:** cumple el ADR. No expande el alcance original.
- **Cronograma:** dentro de la duracion 1-2 meses de la Fase 4.5.
- **Complejidad operacional:** sin Mailpit/MTA obligatorio en produccion;
  Mailpit solo en CI.
- **APIs publicas:** crate nuevo, sin cambios a los crates existentes.
- **Documentacion:** este RFC + actualizaciones futuras al README de
  `crates/ag-mail`.

## 8. Rollback

Si la implementacion descubre que `lettre` no es viable, el trait
`MailSender` permite reemplazar el sender nativo por otro (p. ej. Stalwart
client) sin cambiar la API publica del crate. Si todo el crate resulta
inviable, eliminarlo es seguro porque ningun otro crate del workspace lo
referencia obligatoriamente (la integracion con `ag-auth` se hace via trait
que `ag-auth` define).

## 9. Decision

- Decisor: Angel Nereira (BDFL)
- Fecha de decision: 2026-05-24
- Resultado: aceptada
- Justificacion: formaliza tecnicamente el alcance fijado por `ADR-0007`
  sin contradecir ninguna restriccion del ADR. Cumple la regla 28 del
  CLAUDE.md.

## 10. Referencias

- `docs/adr/0007-ag-mail-ag-domains.md` — decision gobernante.
- `docs/master/ANTI-GRAVITAL-Arquitectura-Tecnica.md` § 8.8 — especificacion.
- `docs/roadmap/fase-04-5-ag-mail-y-ag-domains.md` — criterios de entrada/salida.
- `docs/modules/ag-mail/README.md` — ficha del modulo.
- `CLAUDE.md` reglas 14 (crates), 15 (dependencias), 22 (RFC), 28 (RFC para nuevos crates).
- RFC-0007 — alcance complementario de `ag-domains`.
