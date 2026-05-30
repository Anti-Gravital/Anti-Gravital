# auth-mail-demo

Demuestra como conectar `ag-auth` con `ag-mail` para los tres flujos
transaccionales principales:

1. Registro con verificacion de email.
2. Recuperacion de contrasena.
3. Autenticacion sin contrasena (magic link).

Es un programa de un solo disparo: imprime el resultado de cada flujo y
termina. No abre puertos ni necesita servicios externos.

## Requisitos externos

Ninguno. El ejemplo usa `NullSender`, que captura los correos en memoria
en lugar de enviarlos por SMTP.

## Ejecucion

```bash
cargo run -p auth-mail-demo
```

Salida esperada (los tokens son aleatorios en cada ejecucion):

```
ag-auth + ag-mail conectados.

--- Flujo 1: verificacion de email ---
...
Demo completado: 3 correos enviados sin necesitar SMTP real.
```

## Usar SMTP real

Sustituye `NullSender` por `SmtpSender` en `src/main.rs`:

```rust
use ag_mail::sender::smtp::{SmtpConfig, SmtpSender};
let config = SmtpConfig::new("smtp.example.com", 587, "user", "pass");
let sender = SmtpSender::new(config)?;
```

## Crates demostrados

- `ag-auth`: generacion de tokens y orquestacion de flujos de auth.
- `ag-mail`: envio transaccional (feature `smtp`) y `NullSender`
  (feature `test-utils`) para pruebas sin SMTP.
