# Configurar dominio, TLS y correo transaccional con Anti-Gravital

Esta guia muestra como integrar `ag-domains` y `ag-mail` en un proyecto
Anti-Gravital para gestionar registros DNS, emitir certificados TLS via
ACME y enviar correo transaccional desde el DSL.

---

## Requisitos previos

- Rust 1.78 o superior.
- Un dominio registrado con los nameservers apuntando al proveedor DNS.
- Una cuenta en Cloudflare (o cualquier proveedor con adapter implementado).
- Token de API de Cloudflare con permisos `Zone:DNS:Edit`.

---

## 1. Declarar dominio y correo en el DSL

El DSL v0.7 introduce los bloques `domain`, `mail` y `template`.
Crear o editar el archivo `schema.ag` del proyecto:

```ag
domain mi_dominio {
    name "ejemplo.com"
    provider cloudflare
    dkim_selector "s1"
    dmarc_policy quarantine
    dmarc_rua "reportes@ejemplo.com"
}

mail transaccional {
    provider smtp
    from "noreply@ejemplo.com"

    template bienvenida {
        subject "Bienvenido {{nombre}}"
        vars [nombre, token]
    }

    template recuperacion {
        subject "Recupera tu contrasena"
        vars [enlace]
    }
}
```

El compilador DSL (`ag_dsl::compile`) valida que:
- El `from` del bloque `mail` referencia un dominio declarado.
- Cada `vars` de un `template` coincide con los marcadores `{{var}}` del HTML.

---

## 2. Aplicar registros DNS (SPF, DKIM, DMARC)

`ag-domains` genera y aplica los tres registros necesarios para la entregabilidad
del correo en un upsert idempotente: si el registro ya existe con el mismo
contenido, no se modifica.

```rust
use ag_domains::{
    mail_records::{DkimConfig, MailRecordsConfig, apply_mail_records},
    provider::cloudflare::CloudflareProvider,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = CloudflareProvider::new("CF_TOKEN_AQUI");
    let zone_id = provider.zone_id("ejemplo.com").await?;

    let config = MailRecordsConfig {
        spf_includes: vec!["include:_spf.proveedor.example".to_owned()],
        dkim: Some(DkimConfig {
            selector: "s1".to_owned(),
            public_key_base64: "MIIBIjANBgkq...".to_owned(),
        }),
        dmarc_policy: ag_domains::mail_records::DmarcPolicy::Quarantine,
        dmarc_rua: Some("reportes@ejemplo.com".to_owned()),
    };

    apply_mail_records(&provider, &zone_id, "ejemplo.com", &config).await?;
    println!("Registros SPF/DKIM/DMARC aplicados.");
    Ok(())
}
```

Equivalente via CLI:

```sh
ag domains sync --schema schema.ag --zone-id <ZONE_ID> --token <CF_TOKEN>
```

---

## 3. Verificar propagacion DNS

La propagacion puede tardar entre segundos y 48 horas dependiendo del TTL
anterior y del proveedor. Anti-Gravital consulta multiples resolvers publicos
para dar una vision real del estado:

```rust
use ag_domains::propagation::{PropagationChecker, DEFAULT_RESOLVERS};

let checker = PropagationChecker::new(DEFAULT_RESOLVERS);
let result = checker.check_txt("_dmarc.ejemplo.com", "v=DMARC1").await;

println!(
    "Propagacion: {}/{} resolvers confirmados",
    result.confirmed, result.total
);
```

Equivalente via CLI:

```sh
ag domains check --domain ejemplo.com --expected "v=DMARC1" --min-confirmed 3
```

---

## 4. Emitir un certificado TLS via ACME

`ag-domains` integra `instant-acme` para emitir certificados de Let's Encrypt
usando el challenge DNS-01 (el proveedor DNS crea y elimina el registro TXT
automaticamente):

```rust
use ag_domains::{
    acme::renewal::{CertConfig, issue},
    provider::cloudflare::CloudflareProvider,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = CloudflareProvider::new("CF_TOKEN_AQUI");
    let zone_id = provider.zone_id("ejemplo.com").await?;

    let config = CertConfig {
        domain: "ejemplo.com".to_owned(),
        zone_id,
        contact_email: "admin@ejemplo.com".to_owned(),
        staging: true, // usar false en produccion
    };

    let (cert, credentials) = issue(&config, &provider).await?;

    // Guardar el PEM del certificado y la clave privada.
    std::fs::write("cert.pem", &cert.cert_chain_pem)?;
    std::fs::write("key.pem", &cert.private_key_pem)?;

    // Guardar las credenciales para renovaciones futuras.
    let creds_json = serde_json::to_string(&credentials)?;
    std::fs::write("acme-credentials.json", creds_json)?;

    println!("Certificado emitido y guardado en cert.pem / key.pem");
    Ok(())
}
```

Para renovacion automatica con una tarea en background:

```rust
use ag_domains::acme::renewal::spawn_renewal_task;

let handle = spawn_renewal_task(
    config,
    credentials,
    provider,
    30, // renovar 30 dias antes del vencimiento
    |new_cert| {
        // Callback cuando el certificado se renueva.
        std::fs::write("cert.pem", &new_cert.cert_chain_pem).ok();
        std::fs::write("key.pem", &new_cert.private_key_pem).ok();
    },
);

// handle.abort() para detener la tarea.
```

---

## 5. Enviar correo transaccional

### Via SMTP (lettre + rustls)

```rust
use ag_mail::{
    message::{Address, EmailBuilder},
    sender::{smtp::{SmtpConfig, SmtpSender}, MailSender},
};

let config = SmtpConfig::new("smtp.ejemplo.com", 587, "user", "pass");
let sender = SmtpSender::new(config).await?;

let email = EmailBuilder::new()
    .from(Address::with_name("Ejemplo", "noreply@ejemplo.com"))
    .to(Address::new("usuario@ejemplo.com"))
    .subject("Bienvenido")
    .html_body("<h1>Hola</h1>")
    .text_body("Hola")
    .build()?;

sender.send(&email).await?;
```

### Via un proveedor externo (SMTP)

Para enviar a traves de un proveedor externo, apunta el `SmtpSender` nativo a
su endpoint SMTP. No hay adaptadores con nombre de marca (ver `ADR-0011`); la
via sin terceros es el `MtaSender` nativo (feature `mta`).

```rust
use ag_mail::sender::smtp::{SmtpConfig, SmtpSender};

let config = SmtpConfig::new("smtp.proveedor.example", 587, "usuario", "clave");
let sender = SmtpSender::new(config)?;
sender.send(&email).await?;
```

### Via cola con reintentos

```rust
use std::sync::Arc;
use ag_mail::queue::{InMemoryQueue, RetryPolicy};

let policy = RetryPolicy::default(); // 3 reintentos, backoff exponencial
let queue = InMemoryQueue::new(Arc::new(sender), policy, 64);
let worker = queue.spawn_worker();

queue.enqueue(email).await?;
// worker.abort() para detener el worker.
```

---

## 6. Integracion ag-auth -> ag-mail

`ag-auth` puede enviar correos de verificacion, recuperacion de contrasena
y magic links mediante `AuthMailer`:

```rust
use std::sync::Arc;
use ag_auth::{AgAuth, AuthConfig, AuthMailer};
use ag_mail::sender::smtp::{SmtpConfig, SmtpSender};

let sender: Arc<dyn ag_mail::sender::MailSender> =
    Arc::new(SmtpSender::new(SmtpConfig::new("smtp.proveedor.example", 587, "usuario", "clave"))?);

let mailer = Arc::new(AuthMailer::new(
    sender,
    "noreply@ejemplo.com",
    "Mi Proyecto",
));

let auth = AgAuth::new(config, reqwest::Client::new())?.with_mail(mailer);

// Enviar correo de verificacion:
auth.mailer.as_ref().unwrap()
    .send_verification("usuario@ejemplo.com", "TOKEN", "https://ejemplo.com")
    .await?;
```

Ver tambien: `examples/auth-mail-demo` para un ejemplo ejecutable completo
que demuestra los tres flujos sin necesitar SMTP real.

---

## Referencia rapida de comandos CLI

| Comando | Descripcion |
|---|---|
| `ag domains check --domain ejemplo.com --expected "v=TXT"` | Verifica propagacion DNS |
| `ag domains sync --schema schema.ag --zone-id Z --token T` | Aplica SPF/DKIM/DMARC desde el DSL |
| `ag mail test --to user@e.com --from no@e.com --smtp-host smtp.e.com` | Envia correo de prueba |

---

## Documentos relacionados

- `docs/adr/0007-ag-mail-ag-domains.md` — decision arquitectonica
- `docs/modules/ag-mail/` — especificacion del modulo
- `docs/modules/ag-domains/` — especificacion del modulo
- `docs/rfc/RFC-0006-ag-mail-alcance.md` — alcance v1 de ag-mail
- `docs/rfc/RFC-0007-ag-domains-alcance.md` — alcance v1 de ag-domains
