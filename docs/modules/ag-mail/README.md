# ag-mail

> Capitulo de arquitectura: `docs/architecture/08-modulos-batteries-included.md`, seccion 8.8.
> ADR de decision: `docs/adr/0007-ag-mail-ag-domains.md`.
> Fase de la Hoja de Ruta: `docs/roadmap/fase-04-5-ag-mail-y-ag-domains.md`.
> README del crate: `crates/ag-mail/README.md` (se creara en la implementacion de la Fase 4.5).
> Criticidad: Estandar diferido.
> Fase de implementacion: Fase 4.5. Estado: Pendiente (rama `fase-4.5`).

## Dominio

Comunicacion transaccional outbound: envio de correos de verificacion de cuentas,
magic links, recuperacion de contrasena, alertas y notificaciones. Sender SMTP nativo
y adapters de proveedores gestionados (Resend, SES, Postmark). Templates HTML/plaintext
tipados validados en compile-time contra `schema.ag`. Cola asincrona con reintentos
y metricas.

## Alcance v1 (Fase 4.5)

### Si

- Envio outbound de correo transaccional.
- Sender SMTP nativo (`lettre` + `rustls`).
- Adapters Resend / SES / Postmark como features de Cargo.
- Templates HTML y plaintext (`askama`) validados en build-time.
- Cola asincrona con reintentos, backoff exponencial y metricas.
- Integracion con `ag-auth` para verificacion, recuperacion y magic links.

### No (fuera de alcance, no diferible "luego")

- Servidor MTA completo.
- Recepcion de correo (inbound, IMAP/POP).
- Buzones persistentes.
- Antispam, filtrado, reputacion de IP.
- Gestion avanzada de bounces mas alla de registro.

Para inbound o un servidor de correo completo, usar Postfix, Stalwart u otro
proyecto especializado.

## Stack tecnico planificado

| Componente | Libreria | Notas |
|---|---|---|
| SMTP nativo | `lettre` | Async Tokio, TLS via `rustls` (coherente con The Shield). |
| Templates | `askama` | Compilados en build time, ya en uso por `ag-ui`. |
| Adapters HTTP | `reqwest` | Resend, SES, Postmark. Features Cargo. |
| Trait async | `async-trait` | `MailSender` y `MailQueue`. |
| Persistencia cola | `ag-data` (opcional) | Para sobrevivir reinicios; backend por defecto en memoria. |
| Observabilidad | `ag-observe` | `ag_mail_sent_total`, `ag_mail_failed_total`, latencia. |

Las dependencias externas deben pasar `cargo audit` y `cargo deny`. Pinning explicito
en el workspace.

## API publica planificada

```rust
#[async_trait::async_trait]
pub trait MailSender: Send + Sync {
    async fn send(&self, msg: &Email) -> Result<MessageId, AgMailError>;
    fn provider_name(&self) -> &'static str;
    fn dns_requirements(&self, domain: &str) -> Vec<DnsRecordSpec>;
}

pub enum AgMail {
    Native(SmtpSender),                // lettre + rustls
    Adapter(Box<dyn MailSender>),      // Resend, SES, Postmark, ...
}
```

Patron `Native | Adapter` identico al de `ag-storage` (`Native | S3`) y `ag-cache`
(`moka | Redis`).

## Dependencias internas permitidas

- Puede depender de `ag-core`, `ag-data` (cola opcional persistente),
  `ag-realtime` (fan-out opcional), `ag-observe` (metricas) y `ag-domains`
  (cooperacion para SPF/DKIM/DMARC).
- **NO puede depender de `ag-auth`.** Es `ag-auth` quien consume `ag-mail`
  definiendo un trait pequeno que `ag-mail` implementa para enviar correos de
  verificacion, recuperacion y magic links. Esta direccionalidad esta fijada en
  la sexta regla del capitulo 5 (ADR-0007) y se verifica en CI.

## DSL v0.7 — bloque `mail`

```ag
mail WelcomeEmail {
    from "hello@plenty.market"     # debe referenciar un bloque domain
    subject "Welcome to Plenty"
    template "emails/welcome.html" # debe existir
    vars {
        name String
        activation_url String       # debe usarse en el HTML
    }
}
```

Validaciones build-time del compilador:

1. El `from` referencia un dominio declarado en un bloque `domain`.
2. El archivo de template existe.
3. Las variables del HTML coinciden con las `vars` declaradas.
4. Existe fallback plaintext o politica explicita.

Un correo mal formado deja de ser un bug de runtime y se convierte en un error
de compilacion. **Este es el diferenciador real frente a Resend**, no la
entregabilidad: la entregabilidad es trabajo del proveedor.

## Cooperacion con `ag-domains`

`ag-mail` declara sus requisitos de DNS via `MailSender::dns_requirements` y
`ag-domains` los materializa como registros SPF, DKIM y DMARC. La cooperacion es
**opcional**: si el proyecto usa `ag-mail` con un adapter gestionado (Resend) y
no administra DNS propio, `ag-domains` no es necesario. Ver tambien
`docs/modules/ag-domains/README.md`.

## Criterios de salida (puerta de Fase 4.5)

- [ ] `ag-mail` envia correo transaccional HTML y plaintext via sender nativo Y
      via al menos un adapter.
- [ ] `ag-auth` usa `ag-mail` para verificacion de correo y recuperacion en el
      example `auth-mail-demo`.
- [ ] Cobertura de tests unitarios e integracion >= 75%.
- [ ] Cero dependencias circulares (job de CI verde).
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo audit` y
      `cargo deny check` verdes.

## Riesgos vigilados

- **Confusion con un MTA completo.** Mitigacion: alcance v1 fijado en ADR-0007.
- **Madurez de `lettre`/adapters de provider.** Mitigacion: tests de contrato del
  trait `MailSender`, adapters como features de Cargo (no en binario por defecto).
- **Validacion build-time de templates HTML.** Parsear HTML para cruzar `vars` con
  variables del template es mas costoso de lo que parece; se acepta como tarea
  central del compilador del DSL v0.7.
