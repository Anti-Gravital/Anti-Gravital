# P4 — ag-realtime escalabilidad (50k) + persistencia de eventos

> **For agentic workers:** Plan hijo de `2026-05-26-corrective-before-fase5-MASTER.md`.
> Ejecutar con superpowers:subagent-driven-development o executing-plans. Pasos con
> checkbox (`- [ ]`). TDD donde aplica. Comentarios en ingles (ADR-0008). Leer cada
> archivo antes de editar.

**Goal:** Demostrar de forma reproducible el criterio de Fase 4 (50.000 conexiones
concurrentes al bus de eventos) con un test de carga documentado, y anadir un buffer
de persistencia opcional para que eventos criticos no se pierdan al reiniciar.

**Architecture:** El bus es `EventBus` sobre `tokio::sync::broadcast`. Un suscriptor =
un `broadcast::Receiver<Event>`. El test de carga crea ~50k receivers, publica N
eventos, y mide cuantos se reciben y la latencia agregada, marcado `#[ignore]` por
costo. La persistencia opcional (`event-persistence`) escribe eventos marcados como
criticos a un buffer (fichero append-only o tabla) antes de publicarlos, y los reproduce
al arrancar. Diseno minimo; si crece, abrir RFC (CLAUDE.md seccion 22).

**Tech Stack:** Rust, tokio (broadcast), criterion (opcional), serde_json.

**Cierra:** DEBT-007 (prueba 50k) y DEBT-008 (persistencia de eventos) de `docs/DEBT.md`.

---

## Interfaces existentes (verificadas)

`crates/ag-realtime/src/bus.rs`:
- `pub struct Event { ... }` (con `subject` y `payload`).
- `pub struct EventBus { sender: broadcast::Sender<Event> }`.
- `EventBus::new(capacity: usize) -> Self`.
- `EventBus::publish(&self, subject: impl Into<String>, payload: Vec<u8>) -> Result<(), BusError>`.
- `EventBus::publish_json<T: Serialize>(&self, subject, &T) -> Result<(), BusError>`.
- `EventBus::subscribe(&self) -> broadcast::Receiver<Event>`.

`crates/ag-realtime/src/lib.rs`: `AgRealtime`, `RealtimeConfig`, `pub use bus::{BusError, Event, EventBus}`.
No existe `crates/ag-realtime/tests/` ni `benches/`.

---

## Mapa de archivos

- Create: `crates/ag-realtime/tests/load_50k.rs`
- Modify: `crates/ag-realtime/Cargo.toml` (feature `event-persistence` + dev-deps del test)
- Create: `crates/ag-realtime/src/persistence.rs`
- Modify: `crates/ag-realtime/src/lib.rs` (exponer persistencia bajo feature)
- Create/Modify: `docs/modules/ag-realtime/README.md` y `docs/benchmarks/ag-realtime-load.md`

---

## Task 1: Test de carga 50k (TDD-lite, marcado `#[ignore]`)

**Files:**
- Create: `crates/ag-realtime/tests/load_50k.rs`

- [ ] **Step 1: Escribir el test de carga**

```rust
//! Load test for the in-process event bus. Verifies the Phase 4 criterion of
//! 50,000 concurrent subscribers. Marked `#[ignore]` because it is resource
//! intensive; run explicitly in the manual gate:
//!
//!   cargo test -p ag-realtime --test load_50k -- --ignored --nocapture
//!
//! Methodology and hardware must be recorded in docs/benchmarks/ag-realtime-load.md
//! per CLAUDE.md section 17.

use ag_realtime::EventBus;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "resource intensive; run in manual scalability gate"]
async fn fifty_thousand_subscribers_receive_event() {
    const SUBSCRIBERS: usize = 50_000;
    // Capacity must hold the burst so slow receivers do not lag out.
    let bus = EventBus::new(1024);

    let mut receivers: Vec<_> = (0..SUBSCRIBERS).map(|_| bus.subscribe()).collect();

    bus.publish("load.test", b"ping".to_vec())
        .expect("publish to 50k subscribers must succeed");

    let mut received = 0usize;
    for rx in receivers.iter_mut() {
        match rx.try_recv() {
            Ok(ev) => {
                assert_eq!(ev.subject, "load.test");
                received += 1;
            }
            Err(_) => {}
        }
    }

    // Allow a small tolerance for lagged receivers under broadcast backpressure.
    assert!(
        received >= SUBSCRIBERS * 99 / 100,
        "expected >=99% delivery, got {received}/{SUBSCRIBERS}"
    );
}
```

(Verificar que `Event` expone `subject` como campo publico: `grep -n "pub subject\|pub struct Event" crates/ag-realtime/src/bus.rs`. Ajustar el acceso si difiere.)

- [ ] **Step 2: Compilar el test (sin ejecutar la carga)**

Run: `cargo test -p ag-realtime --test load_50k --no-run`
Expected: compila.

- [ ] **Step 3: Ejecutar en gate manual (local) y registrar resultado**

Run: `cargo test -p ag-realtime --test load_50k -- --ignored --nocapture`
Expected: PASS. Anotar tiempo y hardware para Task 4.

- [ ] **Step 4: Commit**

```bash
git add crates/ag-realtime/tests/load_50k.rs
git commit -m "test(ag-realtime): 50k subscriber load test (ignored, manual gate)"
```

---

## Task 2: Feature `event-persistence` + buffer append-only (TDD)

**Files:**
- Modify: `crates/ag-realtime/Cargo.toml`
- Create: `crates/ag-realtime/src/persistence.rs`
- Modify: `crates/ag-realtime/src/lib.rs`

- [ ] **Step 1: Declarar la feature**

En `crates/ag-realtime/Cargo.toml` `[features]`:

```toml
event-persistence = []
```

- [ ] **Step 2: Escribir el test del buffer primero**

Crear `crates/ag-realtime/src/persistence.rs`:

```rust
//! Optional append-only event buffer for critical events.
//!
//! Enabled by the `event-persistence` feature. Critical events are appended to a
//! newline-delimited JSON file before publishing, and replayed on startup so a
//! restart does not drop them. This is intentionally minimal; a richer store would
//! require an RFC (CLAUDE.md section 22).

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::bus::Event;

/// Append-only file-backed buffer of critical events.
pub struct EventBuffer {
    path: PathBuf,
}

impl EventBuffer {
    /// Opens (or creates) the buffer at `path`.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(Self { path: path.as_ref().to_path_buf() })
    }

    /// Appends one event as a JSON line. Called before publishing a critical event.
    pub fn append(&self, subject: &str, payload: &[u8]) -> std::io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::json!({
            "subject": subject,
            "payload": payload,
        });
        writeln!(file, "{line}")?;
        Ok(())
    }

    /// Reads all buffered events for replay on startup.
    pub fn replay(&self) -> std::io::Result<Vec<(String, Vec<u8>)>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let file = std::fs::File::open(&self.path)?;
        let mut out = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() { continue; }
            let v: serde_json::Value = serde_json::from_str(&line)?;
            let subject = v["subject"].as_str().unwrap_or_default().to_owned();
            let payload: Vec<u8> = serde_json::from_value(v["payload"].clone()).unwrap_or_default();
            out.push((subject, payload));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_then_replay_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.ndjson");
        let buf = EventBuffer::open(&path).unwrap();

        buf.append("user.created", b"alice").unwrap();
        buf.append("user.deleted", b"bob").unwrap();

        let replayed = buf.replay().unwrap();
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[0].0, "user.created");
        assert_eq!(replayed[0].1, b"alice");
        assert_eq!(replayed[1].0, "user.deleted");
    }

    #[test]
    fn replay_missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let buf = EventBuffer::open(dir.path().join("none.ndjson")).unwrap();
        assert!(buf.replay().unwrap().is_empty());
    }
}
```

Asegurar que el campo `Event` se pueda reconstruir desde `(subject, payload)` al
reproducir (el consumidor llama `bus.publish(subject, payload)`), por lo que el buffer
NO necesita el tipo `Event` completo, solo subject+payload. (Si `Event` lleva metadata
extra como timestamp, documentar que el replay regenera el timestamp.)

- [ ] **Step 3: Exponer el modulo bajo feature en lib.rs**

En `crates/ag-realtime/src/lib.rs`:

```rust
#[cfg(feature = "event-persistence")]
pub mod persistence;
```

Anadir `tempfile` a `[dev-dependencies]` de `ag-realtime/Cargo.toml` si no esta
(`grep -n "tempfile" crates/ag-realtime/Cargo.toml`).

- [ ] **Step 4: Ejecutar tests**

Run: `cargo test -p ag-realtime --features event-persistence persistence`
Expected: PASS (los 2 tests).

- [ ] **Step 5: Commit**

```bash
git add crates/ag-realtime/Cargo.toml crates/ag-realtime/src/persistence.rs crates/ag-realtime/src/lib.rs
git commit -m "feat(ag-realtime): optional append-only event persistence buffer"
```

---

## Task 3: Helper de arranque que reproduce el buffer

**Files:**
- Modify: `crates/ag-realtime/src/persistence.rs` (helper de replay-into-bus)

- [ ] **Step 1: Test del replay hacia el bus**

```rust
#[tokio::test]
async fn replay_into_bus_publishes_all() {
    use crate::EventBus;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("e.ndjson");
    let buf = EventBuffer::open(&path).unwrap();
    buf.append("a", b"1").unwrap();
    buf.append("b", b"2").unwrap();

    let bus = EventBus::new(16);
    let mut rx = bus.subscribe();
    replay_into_bus(&buf, &bus).unwrap();

    let first = rx.try_recv().unwrap();
    assert_eq!(first.subject, "a");
}
```

- [ ] **Step 2: Implementar `replay_into_bus`**

```rust
/// Replays all buffered events into the given bus, in order. Call once on startup.
pub fn replay_into_bus(buffer: &EventBuffer, bus: &crate::EventBus) -> std::io::Result<()> {
    for (subject, payload) in buffer.replay()? {
        // Best-effort: a closed bus during startup is a programming error.
        let _ = bus.publish(subject, payload);
    }
    Ok(())
}
```

- [ ] **Step 3: Ejecutar y commit**

Run: `cargo test -p ag-realtime --features event-persistence replay_into_bus`
Expected: PASS.

```bash
git add crates/ag-realtime/src/persistence.rs
git commit -m "feat(ag-realtime): replay buffered events into bus on startup"
```

---

## Task 4: Documentacion de patrones y resultados de carga

**Files:**
- Create: `docs/benchmarks/ag-realtime-load.md`
- Modify: `docs/modules/ag-realtime/README.md`

- [ ] **Step 1: Documentar el benchmark (CLAUDE.md seccion 17)**

Crear `docs/benchmarks/ag-realtime-load.md` con: hardware (CPU, RAM, SO), version Rust
(1.95.0 segun CI), commit, comando exacto, numero de ejecuciones, resultados
(suscriptores, % entrega, tiempo), y desviacion. Rellenar con los datos reales medidos
en Task 1 Step 3. NO inventar metricas: si no se ejecuto en hardware real, marcar la
seccion de resultados como "pendiente de ejecucion en gate manual".

- [ ] **Step 2: Documentar patrones pub/sub + fallback + persistencia**

En `docs/modules/ag-realtime/README.md`, anadir secciones: patron publish/subscribe con
`EventBus`, fallback NATS->bus interno (cuando la feature `nats-external` no esta o NATS
cae), y uso de `event-persistence` para eventos criticos. Enlazar al benchmark.

- [ ] **Step 3: Commit**

```bash
git add docs/benchmarks/ag-realtime-load.md docs/modules/ag-realtime/README.md
git commit -m "docs(ag-realtime): document pub/sub patterns and 50k load methodology"
```

---

## Task 5: Cerrar deudas y verificacion final

- [ ] **Step 1: Cerrar DEBT-007 y DEBT-008**

En `docs/DEBT.md`: DEBT-007 -> `closed (P4)` (con nota de que el resultado se valida en
gate manual); DEBT-008 -> `closed (P4)`.

- [ ] **Step 2: Verificacion global**

Run:
```bash
cargo fmt -p ag-realtime -- --check
cargo clippy -p ag-realtime --all-features -- -D warnings
cargo test -p ag-realtime --features event-persistence
cargo test -p ag-realtime --test load_50k --no-run
cargo build --workspace
```
Expected: limpio; tests unitarios verdes; test de carga compila.

- [ ] **Step 3: Commit**

```bash
git add docs/DEBT.md
git commit -m "docs(ag-realtime): close DEBT-007 and DEBT-008"
```

---

## Self-review

- Prueba 50k -> Task 1 (test ignore + ejecucion en gate) + Task 4 (documentacion CLAUDE.md s17).
- Persistencia de eventos -> Tasks 2-3 (buffer append-only + replay, con tests).
- Documentacion de patrones/fallback -> Task 4.
- Tipos consistentes: `EventBus::new/publish/subscribe`, `Event.subject`, `EventBuffer::open/append/replay`,
  `replay_into_bus` usados igual en todas las tareas.
- Sin metricas inventadas (CLAUDE.md s17): resultados se rellenan con medicion real.
- Pendiente de verificar al ejecutar: visibilidad del campo `Event.subject`, presencia de `tempfile` en dev-deps.
