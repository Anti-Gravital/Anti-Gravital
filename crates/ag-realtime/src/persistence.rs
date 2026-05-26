//! Optional append-only event buffer for critical events.
//!
//! Enabled by the `event-persistence` feature. Critical events are appended to a
//! newline-delimited JSON file before publishing, and replayed on startup so a
//! restart does not drop them. This is intentionally minimal; a richer store would
//! require an RFC (CLAUDE.md section 22).

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Append-only file-backed buffer of critical events.
pub struct EventBuffer {
    path: PathBuf,
}

impl EventBuffer {
    /// Opens (or creates) the buffer at `path`.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }

    /// Appends one event as a JSON line. Call before publishing a critical event.
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
            if line.trim().is_empty() {
                continue;
            }
            let v: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            let subject = v["subject"].as_str().unwrap_or_default().to_owned();
            let payload: Vec<u8> = serde_json::from_value(v["payload"].clone()).unwrap_or_default();
            out.push((subject, payload));
        }
        Ok(out)
    }
}

/// Replays all buffered events into the given bus, in order. Call once on startup.
pub fn replay_into_bus(buffer: &EventBuffer, bus: &crate::EventBus) -> std::io::Result<()> {
    for (subject, payload) in buffer.replay()? {
        // Best-effort: a closed bus during startup is a programming error.
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
        assert_eq!(replayed[0].0, "user.created");
        assert_eq!(replayed[0].1, b"alice");
        assert_eq!(replayed[1].0, "user.deleted");
        assert_eq!(replayed[1].1, b"bob");
    }

    #[test]
    fn replay_missing_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let buf = EventBuffer::open(dir.path().join("none.ndjson")).unwrap();
        assert!(buf.replay().unwrap().is_empty());
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
