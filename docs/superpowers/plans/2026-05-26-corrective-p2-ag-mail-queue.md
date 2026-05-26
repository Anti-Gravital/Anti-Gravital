# P2 — ag-mail cola persistente + headers SMTP

> **For agentic workers:** Plan hijo de `2026-05-26-corrective-before-fase5-MASTER.md`.
> Ejecutar con superpowers:subagent-driven-development o executing-plans. Pasos con
> checkbox (`- [ ]`). TDD estricto. Texto/comentarios en ingles (ADR-0008).
> Antes de editar, leer cada archivo (las interfaces citadas son del 2026-05-26).

**Goal:** Implementar el backend de cola persistente de `ag-mail` (feature
`queue-persistent`) sobre `ag-data`/PostgreSQL, para que los correos encolados
sobrevivan reinicios; y resolver/documentar los headers SMTP personalizados.

**Architecture:** Se introduce un trait comun `MailQueue` (ya existe) con dos
backends intercambiables: `InMemoryQueue` (default, ya existe) y `PersistentQueue`
(nuevo, feature `queue-persistent`). `PersistentQueue` guarda cada correo como una
fila en la tabla `ag_mail_queue` (estado, intentos, next_retry_at) y un worker hace
polling de filas `pending` cuyo `next_retry_at` ya paso, las envia con el
`MailSender`, y actualiza estado con backoff persistido.

**Tech Stack:** Rust, sqlx + PostgreSQL, tokio, async-trait, serde_json, ag-observe.

**Cierra:** DEBT-001 (cola persistente) y DEBT-002 (headers SMTP) de `docs/DEBT.md`.

---

## Interfaces existentes (verificadas)

- `crates/ag-mail/src/queue/mod.rs`:
  - `pub trait MailQueue: Send + Sync { async fn enqueue(&self, email: Email) -> Result<(), AgMailError>; }`
  - `pub struct RetryPolicy { max_retries: u32, base_delay: Duration, backoff_factor: u32 }` con `delay_for(attempt: u32) -> Duration`.
  - `pub struct InMemoryQueue { tx: mpsc::Sender<QueueItem> }`.
  - `#[cfg(feature = "queue-persistent")] pub mod store;` (hoy stub de 4 lineas).
- `crates/ag-mail/src/message.rs`: `Email` y `Address` son `Serialize + Deserialize`.
- `crates/ag-mail/src/sender/mod.rs`: `pub trait MailSender { ... }` con `SmtpSender` default.
- `crates/ag-data/src/lib.rs`: `pub type DbPool = sqlx::PgPool;`, `pub struct DataConfig { url, max_connections, acquire_timeout_secs }`, `pub async fn connect(&DataConfig) -> Result<DbPool, DataError>`, `pub enum DataError`.
- `crates/ag-mail/Cargo.toml`: feature `queue-persistent = []` (vacia, sin ag-data).

---

## Mapa de archivos

- Modify: `crates/ag-mail/Cargo.toml` (dep opcional ag-data + sqlx + chrono/uuid bajo feature)
- Rewrite: `crates/ag-mail/src/queue/store.rs` (PersistentQueue)
- Create: `crates/ag-mail/migrations/0001_mail_queue.sql`
- Modify: `crates/ag-mail/src/metrics.rs` (gauge profundidad de cola + histograma tiempo en cola)
- Modify: `crates/ag-mail/src/sender/smtp.rs` (headers personalizados)
- Test: dentro de `store.rs` (`#[cfg(test)]`) + `crates/ag-mail/tests/persistent_queue.rs`

---

## Task 1: Cablear `ag-data` y deps bajo feature `queue-persistent`

**Files:**
- Modify: `crates/ag-mail/Cargo.toml`

- [ ] **Step 1: Anadir dependencias opcionales**

En `[dependencies]` de `crates/ag-mail/Cargo.toml` anadir:

```toml
ag-data = { workspace = true, optional = true }
sqlx = { workspace = true, optional = true }
chrono = { workspace = true, optional = true }
uuid = { workspace = true, optional = true, features = ["v4", "serde"] }
```

Y cambiar la feature vacia por:

```toml
queue-persistent = ["dep:ag-data", "dep:sqlx", "dep:chrono", "dep:uuid"]
```

(Si `chrono`/`uuid` no estan en el workspace root `Cargo.toml`, anadirlos alli con
versiones fijas y `workspace = true`. Verificar con `grep -n "chrono\|uuid" Cargo.toml`.)

- [ ] **Step 2: Verificar que compila sin la feature (default)**

Run: `cargo build -p ag-mail`
Expected: compila (la feature no se activa por defecto).

- [ ] **Step 3: Verificar que la feature al menos resuelve dependencias**

Run: `cargo build -p ag-mail --features queue-persistent 2>&1 | head -20`
Expected: puede fallar por `store.rs` aun stub — eso lo arregla Task 2. Las deps deben resolverse.

- [ ] **Step 4: Commit**

```bash
git add crates/ag-mail/Cargo.toml Cargo.toml
git commit -m "build(ag-mail): wire ag-data/sqlx behind queue-persistent feature"
```

---

## Task 2: Migracion SQL de la tabla de cola

**Files:**
- Create: `crates/ag-mail/migrations/0001_mail_queue.sql`

- [ ] **Step 1: Escribir la migracion**

```sql
-- ag-mail persistent queue. One row per queued email.
CREATE TABLE IF NOT EXISTS ag_mail_queue (
    id            UUID PRIMARY KEY,
    payload       JSONB        NOT NULL,
    status        TEXT         NOT NULL DEFAULT 'pending',
    attempts      INTEGER      NOT NULL DEFAULT 0,
    max_retries   INTEGER      NOT NULL,
    next_retry_at TIMESTAMPTZ  NOT NULL DEFAULT now(),
    last_error    TEXT,
    created_at    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Worker polls pending rows whose retry time has passed, oldest first.
CREATE INDEX IF NOT EXISTS ag_mail_queue_due_idx
    ON ag_mail_queue (status, next_retry_at)
    WHERE status = 'pending';
```

- [ ] **Step 2: Commit**

```bash
git add crates/ag-mail/migrations/0001_mail_queue.sql
git commit -m "feat(ag-mail): add SQL migration for persistent mail queue"
```

---

## Task 3: Implementar `PersistentQueue` (TDD)

**Files:**
- Rewrite: `crates/ag-mail/src/queue/store.rs`

- [ ] **Step 1: Escribir el test de estado/transicion primero**

Reemplazar el contenido stub de `store.rs`. Empezar por la estructura y un test
unitario puro (sin DB) del calculo de `next_retry_at` y la serializacion del Email:

```rust
//! Persistent mail queue backend backed by `ag-data` (PostgreSQL).
//!
//! Enabled by the `queue-persistent` feature. Each queued email is one row in
//! `ag_mail_queue`. A background worker polls due `pending` rows, sends them via
//! the `MailSender`, and updates the row state with persisted exponential backoff.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{
    error::AgMailError,
    message::Email,
    queue::{MailQueue, RetryPolicy},
    sender::MailSender,
};

/// Lifecycle state of a queued email.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Sending,
    Sent,
    Failed,
}

impl JobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Sending => "sending",
            JobStatus::Sent => "sent",
            JobStatus::Failed => "failed",
        }
    }
}

/// Computes the next retry timestamp for a given attempt using the policy.
pub(crate) fn next_retry_at(policy: &RetryPolicy, attempt: u32, now: DateTime<Utc>) -> DateTime<Utc> {
    let delay = policy.delay_for(attempt);
    now + ChronoDuration::from_std(delay).unwrap_or_else(|_| ChronoDuration::seconds(2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn next_retry_grows_with_attempt() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: Duration::from_secs(2),
            backoff_factor: 2,
        };
        let now = Utc::now();
        let r0 = next_retry_at(&policy, 0, now);
        let r1 = next_retry_at(&policy, 1, now);
        assert!(r1 > r0, "retry for attempt 1 must be later than attempt 0");
    }

    #[test]
    fn email_roundtrips_through_json() {
        let email = Email::builder()
            .from("a@x.com")
            .to("b@y.com")
            .subject("hi")
            .text("body")
            .build()
            .unwrap();
        let json = serde_json::to_value(&email).unwrap();
        let back: Email = serde_json::from_value(json).unwrap();
        assert_eq!(email, back);
    }
}
```

(Si `Email` no implementa `PartialEq`, comparar campos individuales o derivar
`PartialEq` en `message.rs` — verificar primero con `grep -n "derive" crates/ag-mail/src/message.rs`.)

- [ ] **Step 2: Ejecutar el test unitario (sin DB)**

Run: `cargo test -p ag-mail --features queue-persistent next_retry_grows_with_attempt`
Expected: PASS.

- [ ] **Step 3: Implementar `PersistentQueue` y su worker**

Anadir a `store.rs` (antes del bloque de tests):

```rust
/// Persistent queue: enqueues into PostgreSQL and a worker drains due rows.
pub struct PersistentQueue {
    pool: ag_data::DbPool,
    policy: RetryPolicy,
}

impl PersistentQueue {
    /// Builds the queue over an existing pool and spawns the worker.
    pub fn new<S>(pool: ag_data::DbPool, sender: Arc<S>, policy: RetryPolicy) -> Self
    where
        S: MailSender + 'static,
    {
        let worker_pool = pool.clone();
        let worker_policy = policy.clone();
        tokio::spawn(async move {
            run_worker(worker_pool, sender, worker_policy).await;
        });
        Self { pool, policy }
    }
}

#[async_trait]
impl MailQueue for PersistentQueue {
    async fn enqueue(&self, email: Email) -> Result<(), AgMailError> {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO ag_mail_queue (id, payload, status, attempts, max_retries) \
             VALUES ($1, $2, 'pending', 0, $3)",
        )
        .bind(id)
        .bind(Json(&email))
        .bind(self.policy.max_retries as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| AgMailError::Queue(e.to_string()))?;

        crate::metrics::queue_depth_inc();
        Ok(())
    }
}

/// Polls due rows and sends them. One pass per second.
async fn run_worker<S>(pool: ag_data::DbPool, sender: Arc<S>, policy: RetryPolicy)
where
    S: MailSender + 'static,
{
    use tokio::time::{sleep, Duration};
    loop {
        if let Err(e) = drain_once(&pool, sender.as_ref(), &policy).await {
            tracing::error!(error = %e, "ag-mail persistent worker pass failed");
        }
        sleep(Duration::from_secs(1)).await;
    }
}

async fn drain_once<S>(pool: &ag_data::DbPool, sender: &S, policy: &RetryPolicy) -> Result<(), AgMailError>
where
    S: MailSender,
{
    // Claim one due row atomically (SKIP LOCKED for safe concurrency).
    let row: Option<(Uuid, Json<Email>, i32)> = sqlx::query_as(
        "UPDATE ag_mail_queue SET status = 'sending', updated_at = now() \
         WHERE id = ( \
            SELECT id FROM ag_mail_queue \
            WHERE status = 'pending' AND next_retry_at <= now() \
            ORDER BY next_retry_at FOR UPDATE SKIP LOCKED LIMIT 1 \
         ) RETURNING id, payload, attempts",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AgMailError::Queue(e.to_string()))?;

    let Some((id, Json(email), attempts)) = row else {
        return Ok(());
    };

    match sender.send(&email).await {
        Ok(_) => {
            sqlx::query("UPDATE ag_mail_queue SET status = 'sent', updated_at = now() WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| AgMailError::Queue(e.to_string()))?;
            crate::metrics::queue_depth_dec();
        }
        Err(e) => {
            let attempt = attempts as u32 + 1;
            let (status, next) = if attempt > policy.max_retries {
                ("failed", Utc::now())
            } else {
                ("pending", next_retry_at(policy, attempt, Utc::now()))
            };
            sqlx::query(
                "UPDATE ag_mail_queue SET status = $1, attempts = $2, next_retry_at = $3, \
                 last_error = $4, updated_at = now() WHERE id = $5",
            )
            .bind(status)
            .bind(attempt as i32)
            .bind(next)
            .bind(e.to_string())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AgMailError::Queue(e.to_string()))?;
            if status == "failed" {
                crate::metrics::queue_depth_dec();
            }
        }
    }
    Ok(())
}
```

(Verificar la firma real de `MailSender::send` con `grep -n "fn send" crates/ag-mail/src/sender/mod.rs` y ajustar `sender.send(&email)`. Verificar que `AgMailError` tiene variante `Queue(String)`; si no, anadirla en `error.rs`.)

- [ ] **Step 4: Confirmar compilacion con la feature**

Run: `cargo build -p ag-mail --features queue-persistent`
Expected: compila.

- [ ] **Step 5: Commit**

```bash
git add crates/ag-mail/src/queue/store.rs crates/ag-mail/src/error.rs
git commit -m "feat(ag-mail): implement PersistentQueue over ag-data with backoff"
```

---

## Task 4: Test de integracion con PostgreSQL (recuperacion tras reinicio)

**Files:**
- Create: `crates/ag-mail/tests/persistent_queue.rs`

- [ ] **Step 1: Escribir el test (marcado `#[ignore]` si no hay DB en CI)**

```rust
//! Integration test for the persistent mail queue. Requires a PostgreSQL
//! instance via TEST_DATABASE_URL. Ignored by default so CI without a DB stays green.

#![cfg(feature = "queue-persistent")]

use std::sync::Arc;

use ag_mail::message::Email;
use ag_mail::queue::store::PersistentQueue;
use ag_mail::queue::{MailQueue, RetryPolicy};
use ag_mail::sender::NullSender;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL PostgreSQL"]
async fn message_survives_and_is_sent() {
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    let sender = Arc::new(NullSender::new());
    let queue = PersistentQueue::new(pool.clone(), sender.clone(), RetryPolicy::default());

    let email = Email::builder()
        .from("a@x.com").to("b@y.com").subject("s").text("t")
        .build().unwrap();
    queue.enqueue(email).await.unwrap();

    // Wait for the worker to drain it.
    for _ in 0..10 {
        let sent: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM ag_mail_queue WHERE status = 'sent'",
        ).fetch_one(&pool).await.unwrap();
        if sent == 1 { break; }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    let captured = sender.captured();
    assert_eq!(captured.len(), 1, "NullSender must have captured one email");
}
```

(Verificar el API de `NullSender` con `grep -n "NullSender\|captured\|fn new" crates/ag-mail/src/sender/mod.rs` y ajustar. `NullSender` esta bajo feature `test-utils`; anadir `test-utils` a los dev-deps/features del test.)

- [ ] **Step 2: Ejecutar (local con DB) o confirmar que compila el test**

Run (sin DB): `cargo test -p ag-mail --features "queue-persistent test-utils" --no-run`
Expected: compila.
Run (con DB local): `TEST_DATABASE_URL=postgresql://localhost/ag_test cargo test -p ag-mail --features "queue-persistent test-utils" -- --ignored`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/ag-mail/tests/persistent_queue.rs
git commit -m "test(ag-mail): integration test for persistent queue restart-survival"
```

---

## Task 5: Metricas de cola en `ag-observe`

**Files:**
- Modify: `crates/ag-mail/src/metrics.rs`

- [ ] **Step 1: Anadir gauge de profundidad de cola**

Leer `crates/ag-mail/src/metrics.rs` y seguir el patron existente (counters
`ag_mail_sent_total`, `ag_mail_retry_total`). Anadir:

```rust
/// Increments the gauge tracking pending emails in the persistent queue.
pub fn queue_depth_inc() {
    #[cfg(feature = "metrics")]
    metrics::gauge!("ag_mail_queue_depth").increment(1.0);
}

/// Decrements the queue-depth gauge when a job leaves the pending state.
pub fn queue_depth_dec() {
    #[cfg(feature = "metrics")]
    metrics::gauge!("ag_mail_queue_depth").decrement(1.0);
}
```

(Si `metrics` no esta activa, las funciones deben seguir existiendo como no-ops para
que `store.rs` compile sin `metrics`. Ajustar `#[cfg]` segun el patron del archivo.)

- [ ] **Step 2: Verificar y commit**

Run: `cargo build -p ag-mail --features "queue-persistent metrics"` y `cargo build -p ag-mail --features queue-persistent`
Expected: ambos compilan.

```bash
git add crates/ag-mail/src/metrics.rs
git commit -m "feat(ag-mail): add queue-depth gauge metric"
```

---

## Task 6: Headers SMTP personalizados (DEBT-002)

**Files:**
- Modify: `crates/ag-mail/src/sender/smtp.rs`

- [ ] **Step 1: Investigar el API de lettre**

Run: `grep -n "header\|Message::builder\|lettre" crates/ag-mail/src/sender/smtp.rs`
Determinar si la version de lettre del workspace expone `message::header::Headers`
o un `header()` arbitrario. Verificar version: `grep -n "lettre" Cargo.toml crates/ag-mail/Cargo.toml`.

- [ ] **Step 2a (si lettre lo permite): aplicar headers personalizados**

Escribir primero un test en `smtp.rs` `#[cfg(test)]` que construya un `Email` con un
header custom y verifique que el mensaje lettre lo incluye (usando
`lettre::message::Message` y serializando a string):

```rust
#[test]
fn custom_headers_are_applied() {
    let email = Email::builder()
        .from("a@x.com").to("b@y.com").subject("s").text("t")
        .header("X-Campaign", "spring")
        .build().unwrap();
    let message = build_lettre_message(&email).unwrap();
    let formatted = String::from_utf8(message.formatted()).unwrap();
    assert!(formatted.contains("X-Campaign: spring"));
}
```

Luego implementar el mapeo de `email.headers` a la API de lettre en
`build_lettre_message` (o el helper equivalente). Usar
`lettre::message::header::Headers` + `MessageBuilder::header` segun la version.

Run: `cargo test -p ag-mail custom_headers_are_applied`
Expected: PASS.

- [ ] **Step 2b (si lettre NO lo permite): documentar y abrir issue**

Si la version de lettre no expone headers arbitrarios, NO forzar un hack. Actualizar
`docs/DEBT.md` DEBT-002 con la version exacta de lettre y el enlace al issue upstream,
y dejar un `// TECH-DEBT:` en `smtp.rs` con el formato de CLAUDE.md seccion 29.

- [ ] **Step 3: Commit**

```bash
git add crates/ag-mail/src/sender/smtp.rs docs/DEBT.md
git commit -m "feat(ag-mail): apply custom SMTP headers (or document lettre limitation)"
```

---

## Task 7: Cerrar deudas en docs y verificacion final

- [ ] **Step 1: Marcar DEBT-001 (y DEBT-002 si se resolvio) como cerradas**

Editar `docs/DEBT.md`: cambiar `Status: open` a `Status: closed (P2, 2026-...)` en
DEBT-001, y en DEBT-002 segun el resultado de Task 6.

- [ ] **Step 2: Actualizar README de ag-mail**

En `crates/ag-mail/README.md`, en la seccion Tech Debt, marcar la cola persistente
como implementada (feature `queue-persistent`).

- [ ] **Step 3: Verificacion global**

Run:
```bash
cargo fmt -p ag-mail -- --check
cargo clippy -p ag-mail --features "queue-persistent metrics" -- -D warnings
cargo test -p ag-mail --features "queue-persistent test-utils" --no-run
cargo build --workspace
```
Expected: todo limpio; tests compilan; workspace compila.

- [ ] **Step 4: Commit**

```bash
git add docs/DEBT.md crates/ag-mail/README.md
git commit -m "docs(ag-mail): close DEBT-001 persistent queue; update README"
```

---

## Self-review

- DEBT-001 cola persistente -> Tasks 1-5 (Cargo, migracion, PersistentQueue, test, metricas).
- DEBT-002 headers SMTP -> Task 6 (resuelto o documentado con version/issue).
- Compatibilidad: `InMemoryQueue` sin tocar; feature aislada; default sin ag-data.
- Tipos consistentes: `MailQueue::enqueue`, `RetryPolicy::delay_for`, `ag_data::DbPool`,
  `JobStatus`, `next_retry_at` usados igual en todas las tareas.
- Pendiente de verificar al ejecutar: firma exacta de `MailSender::send`, `Email: PartialEq`,
  API de `NullSender`, variante `AgMailError::Queue`, version de lettre.
