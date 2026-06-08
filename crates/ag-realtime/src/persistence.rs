//! Optional append-only event buffer for critical events.
//!
//! Enabled by the event-persistence feature. Critical events are appended to a
//! newline-delimited JSON file before publishing, and replayed on startup so a
//! restart does not drop them. This is intentionally minimal; a richer store would
//! require an RFC (CLAUDE.md section 22).

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct PersistedEvent {
    subject: String,
    payload: Vec<u8>,
}

/// Append-only file-backed buffer of critical events.
#[derive(Clone, Debug)]
pub struct EventBuffer {
    path: PathBuf,
    writer: Arc<Mutex<File>>,
}

impl EventBuffer {
    /// Opens (or creates) the buffer at path.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        let writer = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            writer: Arc::new(Mutex::new(writer)),
        })
    }

    /// Appends one event as a JSON line. Call before publishing a critical event.
    ///
    /// This synchronous method reuses a persistent file handle. Async callers
    /// should prefer append_async to keep filesystem I/O off the runtime.
    pub fn append(&self, subject: &str, payload: &[u8]) -> std::io::Result<()> {
        append_event(
            &self.writer,
            &PersistedEvent {
                subject: subject.to_owned(),
                payload: payload.to_vec(),
            },
        )
    }

    /// Appends one event on Tokio's blocking pool.
    pub async fn append_async(&self, subject: &str, payload: &[u8]) -> std::io::Result<()> {
        let writer = Arc::clone(&self.writer);
        let event = PersistedEvent {
            subject: subject.to_owned(),
            payload: payload.to_vec(),
        };
        tokio::task::spawn_blocking(move || append_event(&writer, &event))
            .await
            .map_err(|error| std::io::Error::other(format!("event writer task failed: {error}")))?
    }

    /// Reads all buffered events for replay on startup.
    pub fn replay(&self) -> std::io::Result<Vec<(String, Vec<u8>)>> {
        lock_writer(&self.writer)?.flush()?;
        let file = File::open(&self.path)?;
        let mut out = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let event: PersistedEvent = serde_json::from_str(&line)
                .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
            out.push((event.subject, event.payload));
        }
        Ok(out)
    }
}

fn append_event(writer: &Arc<Mutex<File>>, event: &PersistedEvent) -> std::io::Result<()> {
    let mut file = lock_writer(writer)?;
    serde_json::to_writer(&mut *file, event).map_err(std::io::Error::other)?;
    file.write_all(b"\n")?;
    file.flush()
}

fn lock_writer(writer: &Arc<Mutex<File>>) -> std::io::Result<MutexGuard<'_, File>> {
    writer
        .lock()
        .map_err(|_| std::io::Error::other("event buffer writer lock poisoned"))
}

/// Replays all buffered events into the given bus, in order. Call once on startup.
pub fn replay_into_bus(buffer: &EventBuffer, bus: &crate::EventBus) -> std::io::Result<()> {
    for (subject, payload) in buffer.replay()? {
        let _ = bus.publish(subject, payload);
    }
    Ok(())
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
        assert_eq!(replayed[0], ("user.created".to_owned(), b"alice".to_vec()));
        assert_eq!(replayed[1], ("user.deleted".to_owned(), b"bob".to_vec()));
    }

    #[test]
    fn replay_new_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let buf = EventBuffer::open(dir.path().join("none.ndjson")).unwrap();
        assert!(buf.replay().unwrap().is_empty());
    }

    #[tokio::test]
    async fn concurrent_async_appends_are_complete() {
        let dir = tempfile::tempdir().unwrap();
        let buf = EventBuffer::open(dir.path().join("events.ndjson")).unwrap();
        let mut tasks = Vec::new();

        for index in 0..32 {
            let buf = buf.clone();
            tasks.push(tokio::spawn(async move {
                buf.append_async("concurrent", index.to_string().as_bytes())
                    .await
                    .unwrap();
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }

        let replayed = buf.replay().unwrap();
        assert_eq!(replayed.len(), 32);
        assert!(replayed.iter().all(|(subject, _)| subject == "concurrent"));
    }

    #[test]
    fn malformed_record_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.ndjson");
        std::fs::write(&path, b"{not-json}\n").unwrap();

        let buf = EventBuffer::open(&path).unwrap();
        let error = buf.replay().unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

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
        let second = rx.try_recv().unwrap();
        assert_eq!(second.subject, "b");
    }
}
