# P3 — ag-domains notAfter + renovacion programada por fecha

> **For agentic workers:** Plan hijo de `2026-05-26-corrective-before-fase5-MASTER.md`.
> Ejecutar con superpowers:subagent-driven-development o executing-plans. Pasos con
> checkbox (`- [ ]`). TDD estricto. Comentarios en ingles (ADR-0008). Leer cada
> archivo antes de editar.

**Goal:** Parsear la fecha `notAfter` del certificado emitido y programar la
renovacion ACME exactamente cuando faltan `renew_before_days` para vencer (hoy
renueva en cada ciclo), y exponer una metrica de dias-hasta-expiracion para alertas.

**Architecture:** Se anade una funcion `parse_not_after(cert_chain_pem) -> DateTime<Utc>`
que extrae la validez del leaf cert del PEM. `spawn_renewal_task` deja de dormir un
periodo fijo: calcula `not_after - renew_before_days`, duerme hasta ese instante (o un
intervalo de chequeo de 24 h, lo que ocurra antes), y solo renueva cuando procede.
Se emite la metrica `ag_domains_cert_days_until_expiry`.

**Tech Stack:** Rust, instant-acme, rcgen, x509-parser (nuevo, justificado abajo),
chrono, tokio, ag-observe.

**Cierra:** DEBT-005 (notAfter) de `docs/DEBT.md`. Elimina el `TECH-DEBT` de
`crates/ag-domains/src/acme/renewal.rs:117-122`.

---

## Estado actual (verificado)

`crates/ag-domains/src/acme/renewal.rs`:
- `pub struct IssuedCert { pub cert_chain_pem: String, pub private_key_pem: String }`.
- `pub fn spawn_renewal_task<P>(config, credentials, provider, renew_before_days, on_renewed) -> JoinHandle<()>`
  con el `TECH-DEBT` en lineas 117-122: `let sleep_secs = renew_before_days.max(1) * 86_400;`
  y renovacion incondicional en cada iteracion del loop (lineas 124-142).
- `Cargo.toml`: feature `acme = ["dep:instant-acme", "dep:rcgen"]`, `instant-acme = "0.7"`.

No hay parser X509 disponible en el crate hoy.

---

## Justificacion de dependencia (CLAUDE.md seccion 15)

Para parsear `notAfter` de un PEM se necesita un parser X509. Opciones:
- `x509-parser` (MIT/Apache-2.0, maduro, sin OpenSSL, puro Rust) — **elegido**.
- `rustls-pki-types` + `der`: mas bajo nivel, mas codigo para extraer validez.
- OpenSSL: dependencia de sistema, contradice binarios estaticos (CLAUDE.md seccion 13).

`x509-parser` se anade bajo la feature `acme` existente (solo se compila con ACME).
Registrar la justificacion en el commit y en `docs/DEBT.md` no aplica (es resolucion,
no deuda); mencionar la dep en RFC-0007 si esa RFC lista dependencias.

---

## Mapa de archivos

- Modify: `crates/ag-domains/Cargo.toml` (anadir `x509-parser` y `chrono` a feature acme)
- Modify: `crates/ag-domains/src/acme/renewal.rs` (parse_not_after + reescritura de spawn_renewal_task)
- Modify: `crates/ag-domains/src/metrics.rs` (gauge dias hasta expiracion)
- Test: `#[cfg(test)]` en `renewal.rs`

---

## Task 1: Anadir deps de parseo bajo feature `acme`

**Files:**
- Modify: `crates/ag-domains/Cargo.toml`

- [ ] **Step 1: Anadir x509-parser y chrono**

En `[dependencies]`:

```toml
x509-parser = { version = "0.16", optional = true }
chrono = { workspace = true, optional = true }
```

Y extender la feature acme:

```toml
acme = ["dep:instant-acme", "dep:rcgen", "dep:x509-parser", "dep:chrono"]
```

(Si `chrono` no esta en el workspace root, anadirlo alli. Verificar:
`grep -n "chrono" Cargo.toml`.)

- [ ] **Step 2: Verificar resolucion de deps**

Run: `cargo build -p ag-domains --features acme 2>&1 | head -20`
Expected: compila (aun sin usar las nuevas deps).

- [ ] **Step 3: Commit**

```bash
git add crates/ag-domains/Cargo.toml Cargo.toml
git commit -m "build(ag-domains): add x509-parser and chrono under acme feature"
```

---

## Task 2: `parse_not_after` (TDD)

**Files:**
- Modify: `crates/ag-domains/src/acme/renewal.rs`

- [ ] **Step 1: Escribir el test primero**

Anadir al modulo `#[cfg(test)]` de `renewal.rs`. Necesitamos un cert PEM auto-firmado
de prueba con un `notAfter` conocido. Generarlo con `rcgen` (ya es dep) dentro del test:

```rust
#[test]
fn parse_not_after_reads_certificate_validity() {
    use rcgen::{CertificateParams, KeyPair};
    use chrono::{Datelike, Utc};

    // Self-signed cert valid until a known year.
    let mut params = CertificateParams::new(vec!["example.com".to_string()]).unwrap();
    let target_year = Utc::now().year() + 1;
    params.not_after = rcgen::date_time_ymd(target_year, 1, 1);
    let key = KeyPair::generate().unwrap();
    let cert = params.self_signed(&key).unwrap();
    let pem = cert.pem();

    let not_after = parse_not_after(&pem).expect("must parse notAfter");
    assert_eq!(not_after.year(), target_year, "parsed year must match cert");
}

#[test]
fn parse_not_after_rejects_garbage() {
    assert!(parse_not_after("not a pem").is_err());
}
```

(Verificar la API de rcgen del workspace: `grep -n "rcgen" Cargo.toml` y ajustar
`self_signed`/`not_after`/`date_time_ymd` a la version. La version usada en
`renewal.rs` ya emplea `CertificateParams::new` + `KeyPair::generate`.)

- [ ] **Step 2: Ejecutar el test para verlo fallar**

Run: `cargo test -p ag-domains --features acme parse_not_after`
Expected: FAIL ("cannot find function `parse_not_after`").

- [ ] **Step 3: Implementar `parse_not_after`**

Anadir en `renewal.rs` (tras los `use`, anadir `use chrono::{DateTime, Utc};` y
`use x509_parser::prelude::*;`):

```rust
/// Parses the `notAfter` validity bound of the leaf certificate in a PEM chain.
///
/// Returns the expiry as a UTC timestamp. Errors if the PEM is malformed or
/// contains no certificate.
pub fn parse_not_after(cert_chain_pem: &str) -> Result<DateTime<Utc>, AgDomainsError> {
    let (_, pem) = x509_parser::pem::parse_x509_pem(cert_chain_pem.as_bytes())
        .map_err(|e| AgDomainsError::Acme(format!("invalid PEM: {e}")))?;
    let cert = pem
        .parse_x509()
        .map_err(|e| AgDomainsError::Acme(format!("invalid X509: {e}")))?;
    let ts = cert.validity().not_after.timestamp();
    DateTime::from_timestamp(ts, 0)
        .ok_or_else(|| AgDomainsError::Acme("notAfter out of range".to_owned()))
}
```

- [ ] **Step 4: Ejecutar el test para verlo pasar**

Run: `cargo test -p ag-domains --features acme parse_not_after`
Expected: PASS (ambos tests).

- [ ] **Step 5: Commit**

```bash
git add crates/ag-domains/src/acme/renewal.rs
git commit -m "feat(ag-domains): parse certificate notAfter from PEM chain"
```

---

## Task 3: Renovacion programada por fecha real

**Files:**
- Modify: `crates/ag-domains/src/acme/renewal.rs` (cuerpo de `spawn_renewal_task`)

- [ ] **Step 1: Escribir un test del calculo de "tiempo hasta renovar"**

Extraer la decision en una funcion pura testeable y testearla primero:

```rust
#[test]
fn seconds_until_renewal_respects_threshold() {
    use chrono::{Duration, Utc};
    let now = Utc::now();
    let not_after = now + Duration::days(30);
    // Renew 10 days before expiry => sleep ~20 days, capped at check_interval.
    let secs = seconds_until_renewal(not_after, 10, now, 86_400);
    assert!(secs <= 86_400, "must not sleep past the 24h check interval");

    let near = now + Duration::days(5);
    let secs_near = seconds_until_renewal(near, 10, now, 86_400);
    assert_eq!(secs_near, 0, "past threshold => renew now (0 sleep)");
}
```

- [ ] **Step 2: Ejecutar para verlo fallar**

Run: `cargo test -p ag-domains --features acme seconds_until_renewal`
Expected: FAIL (funcion no existe).

- [ ] **Step 3: Implementar la funcion pura**

```rust
/// Seconds to sleep before the next renewal check.
///
/// Returns 0 when the certificate is already within `renew_before_days` of expiry
/// (renew immediately). Otherwise returns the time until the renewal threshold,
/// capped at `check_interval_secs` so the task re-evaluates at least that often.
pub(crate) fn seconds_until_renewal(
    not_after: chrono::DateTime<chrono::Utc>,
    renew_before_days: u64,
    now: chrono::DateTime<chrono::Utc>,
    check_interval_secs: u64,
) -> u64 {
    let threshold = not_after - chrono::Duration::days(renew_before_days as i64);
    let until = (threshold - now).num_seconds();
    if until <= 0 {
        0
    } else {
        (until as u64).min(check_interval_secs)
    }
}
```

- [ ] **Step 4: Reescribir el loop de `spawn_renewal_task`**

Reemplazar el bloque del `TECH-DEBT` (lineas ~116-142) por una version que: tras emitir
el cert, parsea `not_after`, emite la metrica, y duerme `seconds_until_renewal` en vez
de un periodo fijo. Renueva solo cuando el sleep calculado fue 0 (o tras despertar al
umbral). Estructura:

```rust
const CHECK_INTERVAL_SECS: u64 = 86_400;

tokio::spawn(async move {
    // Track the latest known expiry; None forces an immediate first issuance.
    let mut not_after: Option<chrono::DateTime<chrono::Utc>> = None;

    loop {
        let sleep_secs = match not_after {
            Some(exp) => seconds_until_renewal(
                exp,
                renew_before_days,
                chrono::Utc::now(),
                CHECK_INTERVAL_SECS,
            ),
            None => 0,
        };

        if sleep_secs > 0 {
            // Emit days-until-expiry before sleeping.
            if let Some(exp) = not_after {
                let days = (exp - chrono::Utc::now()).num_days().max(0);
                crate::metrics::set_cert_days_until_expiry(&config.domain, days);
            }
            tokio::time::sleep(Duration::from_secs(sleep_secs)).await;
            continue;
        }

        info!(domain = config.domain, "renewing TLS certificate via ACME");
        let creds: AccountCredentials = serde_json::from_slice(&credentials_json)
            .expect("previously serialized AccountCredentials is always deserializable");

        match issue_with_credentials(&config, creds, &provider).await {
            Ok(cert) => {
                match parse_not_after(&cert.cert_chain_pem) {
                    Ok(exp) => {
                        not_after = Some(exp);
                        let days = (exp - chrono::Utc::now()).num_days().max(0);
                        crate::metrics::set_cert_days_until_expiry(&config.domain, days);
                    }
                    Err(e) => {
                        warn!(domain = config.domain, error = %e, "could not parse notAfter; will recheck in 24h");
                        not_after = Some(chrono::Utc::now() + chrono::Duration::seconds(CHECK_INTERVAL_SECS as i64));
                    }
                }
                info!(domain = config.domain, "certificate renewed");
                on_renewed(cert);
            }
            Err(e) => {
                error!(domain = config.domain, error = %e, "failed to renew certificate");
                // Back off one check interval before retrying.
                not_after = Some(chrono::Utc::now() + chrono::Duration::seconds(CHECK_INTERVAL_SECS as i64));
            }
        }
    }
});
```

Eliminar el comentario `TECH-DEBT` y la variable `sleep_secs` fija.

- [ ] **Step 5: Ejecutar tests del crate**

Run: `cargo test -p ag-domains --features acme`
Expected: PASS (incluye los nuevos + los 6 existentes de `renewal.rs`).

- [ ] **Step 6: Commit**

```bash
git add crates/ag-domains/src/acme/renewal.rs
git commit -m "feat(ag-domains): schedule renewal by parsed notAfter instead of fixed period"
```

---

## Task 4: Metrica de dias hasta expiracion

**Files:**
- Modify: `crates/ag-domains/src/metrics.rs`

- [ ] **Step 1: Anadir el gauge**

Leer `crates/ag-domains/src/metrics.rs` y seguir el patron existente. Anadir:

```rust
/// Reports days remaining until the certificate for `domain` expires.
/// Drives near-expiry alerting in dashboards.
pub fn set_cert_days_until_expiry(domain: &str, days: i64) {
    #[cfg(feature = "metrics")]
    metrics::gauge!("ag_domains_cert_days_until_expiry", "domain" => domain.to_owned())
        .set(days as f64);
    #[cfg(not(feature = "metrics"))]
    let _ = (domain, days);
}
```

(Ajustar `#[cfg]` y el nombre del crate de metricas al patron real del archivo; si
`ag-domains` no tiene feature `metrics` aun, anadirla en Cargo.toml o emitir via
`ag-observe` segun como lo hagan los demas crates — verificar
`grep -rn "metrics\|gauge\|counter" crates/ag-domains/src/metrics.rs`.)

- [ ] **Step 2: Verificar y commit**

Run: `cargo build -p ag-domains --features acme` y `cargo clippy -p ag-domains --features acme -- -D warnings`
Expected: limpio.

```bash
git add crates/ag-domains/src/metrics.rs crates/ag-domains/Cargo.toml
git commit -m "feat(ag-domains): expose cert days-until-expiry gauge"
```

---

## Task 5: Cerrar deuda y verificacion final

- [ ] **Step 1: Cerrar DEBT-005 en docs/DEBT.md**

Cambiar `Status: open` a `Status: closed (P3, 2026-...)` en DEBT-005. Actualizar la
seccion Tech Debt de `crates/ag-domains/README.md` (quitar notAfter de pendientes).

- [ ] **Step 2: Confirmar que no queda el TECH-DEBT en el codigo**

Run: `grep -n "TECH-DEBT" crates/ag-domains/src/acme/renewal.rs`
Expected: sin coincidencias (exit 1).

- [ ] **Step 3: Verificacion global**

Run:
```bash
cargo fmt -p ag-domains -- --check
cargo clippy -p ag-domains --features "acme" -- -D warnings
cargo test -p ag-domains --features acme
cargo build --workspace
```
Expected: todo limpio y verde.

- [ ] **Step 4: Commit**

```bash
git add docs/DEBT.md crates/ag-domains/README.md
git commit -m "docs(ag-domains): close DEBT-005 notAfter renewal"
```

---

## Self-review

- notAfter parseado del PEM -> Task 2 (`parse_not_after`, 2 tests).
- Renovacion por fecha real -> Task 3 (`seconds_until_renewal` pura y testeada + loop reescrito).
- Metrica de expiracion para alertas -> Task 4.
- TECH-DEBT eliminado del codigo -> Task 5 step 2.
- Dependencia x509-parser justificada (CLAUDE.md seccion 15) -> seccion de justificacion.
- Tipos consistentes: `IssuedCert.cert_chain_pem`, `parse_not_after`, `seconds_until_renewal`,
  `set_cert_days_until_expiry`, `AgDomainsError::Acme` usados igual en todas las tareas.
- Pendiente de verificar al ejecutar: API exacta de rcgen (`self_signed`, `date_time_ymd`),
  patron de metricas de ag-domains.
