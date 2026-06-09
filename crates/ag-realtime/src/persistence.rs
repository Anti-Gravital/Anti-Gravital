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

use tokio::sync::Semaphore;

use serde::{Deserialize, Serialize};

/// Default maximum number of async appends that may be pending at once.
pub const DEFAULT_MAX_PENDING_APPENDS: usize = 64;

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
    pending_appends: Arc<Semaphore>,
    max_pending_appends: usize,
}

impl EventBuffer {
    /// Opens (or creates) the buffer at path.
    pub fn open(path: impl AsRef<Path>) -> std::io::Result<Self> {
        Self::open_with_max_pending_appends(path, DEFAULT_MAX_PENDING_APPENDS)
    }

    /// Opens the buffer with an explicit bound for pending async appends.
    ///
    /// When the bound is reached, append_async waits for capacity instead of
    /// submitting more blocking filesystem work.
    pub fn open_with_max_pending_appends(
        path: impl AsRef<Path>,
        max_pending_appends: usize,
    ) -> std::io::Result<Self> {
        if max_pending_appends == 0 || u32::try_from(max_pending_appends).is_err() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "max pending event appends must be between 1 and u32::MAX",
            ));
        }

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
            pending_appends: Arc::new(Semaphore::new(max_pending_appends)),
            max_pending_appends,
        })
    }

    /// Returns the maximum number of async appends that may be pending.
    pub fn max_pending_appends(&self) -> usize {
        self.max_pending_appends
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
    ///
    /// Pending work is bounded by max_pending_appends. When the limit is full,
    /// this future waits and provides backpressure without blocking a Tokio
    /// worker thread.
    pub async fn append_async(&self, subject: &str, payload: &[u8]) -> std::io::Result<()> {
        let permit = Arc::clone(&self.pending_appends)
            .acquire_owned()
            .await
            .map_err(|_| std::io::Error::other("event append limiter closed"))?;
        let writer = Arc::clone(&self.writer);
        let event = PersistedEvent {
            subject: subject.to_owned(),
            payload: payload.to_vec(),
        };
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            append_event(&writer, &event)
        })
        .await
        .map_err(|error| std::io::Error::other(format!("event writer task failed: {error}")))?
    }

    /// Flushes every append submitted before this call without blocking Tokio.
    ///
    /// This is the shutdown barrier for async callers: stop submitting work,
    /// await flush_async, then drop all EventBuffer clones. No background writer
    /// task remains after append futures complete.
    pub async fn flush_async(&self) -> std::io::Result<()> {
        let permit_count = u32::try_from(self.max_pending_appends)
            .map_err(|_| std::io::Error::other("invalid event append limit"))?;
        let permits = Arc::clone(&self.pending_appends)
            .acquire_many_owned(permit_count)
            .await
            .map_err(|_| std::io::Error::other("event append limiter closed"))?;
        let writer = Arc::clone(&self.writer);
        tokio::task::spawn_blocking(move || {
            let _permits = permits;
            lock_writer(&writer)?.flush()
        })
        .await
        .map_err(|error| std::io::Error::other(format!("event flush task failed: {error}")))?
    }

    /// Reads all buffered events for replay on startup.
    pub fn replay(&self) -> std::io::Result<Vec<(String, Vec<u8>)>> {
        lock_writer(&self.writer)?.flush()?;
        let file = File::open(&self.path)?;
        let mut out = Vec::new();
        for (index, line) in BufReader::new(file).lines().enumerate() {
            let line_number = index + 1;
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let event: PersistedEvent = serde_json::from_str(&line).map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid persisted event at line {line_number}: {error}"),
                )
            })?;
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

trait ReplayPublisher {
    fn publish(&self, subject: String, payload: Vec<u8>) -> Result<(), crate::BusError>;
}

impl ReplayPublisher for crate::EventBus {
    fn publish(&self, subject: String, payload: Vec<u8>) -> Result<(), crate::BusError> {
        crate::EventBus::publish(self, subject, payload)
    }
}

/// Replays all buffered events into the given bus, in order. Call once on startup.
///
/// Invalid persisted records return InvalidData. A bus publication failure
/// stops replay and returns BrokenPipe.
pub fn replay_into_bus(buffer: &EventBuffer, bus: &crate::EventBus) -> std::io::Result<()> {
    replay_into_publisher(buffer, bus)
}

fn replay_into_publisher(
    buffer: &EventBuffer,
    publisher: &impl ReplayPublisher,
) -> std::io::Result<()> {
    for (subject, payload) in buffer.replay()? {
        publisher
            .publish(subject.clone(), payload)
            .map_err(|error| {
                std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    format!("failed to replay event '{subject}': {error}"),
                )
            })?;
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
    fn open_uses_default_pending_append_limit() {
        let dir = tempfile::tempdir().unwrap();
        let buf = EventBuffer::open(dir.path().join("events.ndjson")).unwrap();
        assert_eq!(buf.max_pending_appends(), DEFAULT_MAX_PENDING_APPENDS);
    }

    #[test]
    fn explicit_pending_append_limit_is_validated() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.ndjson");
        let error = EventBuffer::open_with_max_pending_appends(&path, 0).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);

        let buf = EventBuffer::open_with_max_pending_appends(path, 3).unwrap();
        assert_eq!(buf.max_pending_appends(), 3);
    }

    #[test]
    fn replay_new_file_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let buf = EventBuffer::open(dir.path().join("none.ndjson")).unwrap();
        assert!(buf.replay().unwrap().is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn async_append_limit_applies_backpressure_and_reuses_capacity() {
        let dir = tempfile::tempdir().unwrap();
        let buf = EventBuffer::open_with_max_pending_appends(dir.path().join("events.ndjson"), 1)
            .unwrap();
        let writer = Arc::clone(&buf.writer);
        let (locked_tx, locked_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let blocker = std::thread::spawn(move || {
            let writer_guard = lock_writer(&writer).unwrap();
            locked_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            drop(writer_guard);
        });
        locked_rx.recv().unwrap();

        let first_buf = buf.clone();
        let first = tokio::spawn(async move {
            first_buf.append_async("first", b"1").await.unwrap();
        });
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;

        let second_buf = buf.clone();
        let mut second = tokio::spawn(async move {
            second_buf.append_async("second", b"2").await.unwrap();
        });
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), &mut second)
                .await
                .is_err(),
            "second append must wait while the only permit is occupied"
        );

        release_tx.send(()).unwrap();
        first.await.unwrap();
        second.await.unwrap();
        blocker.join().unwrap();

        buf.append_async("third", b"3").await.unwrap();
        buf.flush_async().await.unwrap();
        assert_eq!(buf.replay().unwrap().len(), 3);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn flush_async_waits_for_prior_appends() {
        let dir = tempfile::tempdir().unwrap();
        let buf = EventBuffer::open_with_max_pending_appends(dir.path().join("events.ndjson"), 1)
            .unwrap();

        let writer = Arc::clone(&buf.writer);
        let (locked_tx, locked_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let blocker = std::thread::spawn(move || {
            let writer_guard = lock_writer(&writer).unwrap();
            locked_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            drop(writer_guard);
        });
        locked_rx.recv().unwrap();

        let append_buf = buf.clone();
        let append = tokio::spawn(async move {
            append_buf.append_async("flush", b"pending").await.unwrap();
        });
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;

        let flush_buf = buf.clone();
        let mut flush = tokio::spawn(async move {
            flush_buf.flush_async().await.unwrap();
        });
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), &mut flush)
                .await
                .is_err(),
            "flush must wait for the prior append permit"
        );

        release_tx.send(()).unwrap();
        append.await.unwrap();
        flush.await.unwrap();
        blocker.join().unwrap();
        assert_eq!(buf.replay().unwrap().len(), 1);
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
    fn malformed_record_is_rejected_with_line_context() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.ndjson");
        std::fs::write(
            &path,
            b"{\"subject\":\"valid\",\"payload\":[1]}\n{not-json}\n",
        )
        .unwrap();

        let buf = EventBuffer::open(&path).unwrap();
        let error = buf.replay().unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("line 2"));
    }

    #[test]
    fn truncated_final_record_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.ndjson");
        std::fs::write(
            &path,
            b"{\"subject\":\"valid\",\"payload\":[1]}\n{\"subject\":",
        )
        .unwrap();

        let buf = EventBuffer::open(&path).unwrap();
        let error = buf.replay().unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("line 2"));
    }

    #[test]
    fn record_with_missing_or_wrong_fields_is_rejected() {
        for record in [
            r#"{"payload":[1,2,3]}"#,
            r#"{"subject":"user.created","payload":"not-bytes"}"#,
        ] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("events.ndjson");
            std::fs::write(&path, format!("{record}\n")).unwrap();

            let buf = EventBuffer::open(&path).unwrap();
            let error = buf.replay().unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
            assert!(error.to_string().contains("line 1"));
        }
    }

    struct ClosedPublisher;

    impl ReplayPublisher for ClosedPublisher {
        fn publish(&self, _subject: String, _payload: Vec<u8>) -> Result<(), crate::BusError> {
            Err(crate::BusError::Closed)
        }
    }

    #[test]
    fn replay_propagates_publish_failure() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("events.ndjson");
        let buf = EventBuffer::open(&path).unwrap();
        buf.append("user.created", b"alice").unwrap();

        let error = replay_into_publisher(&buf, &ClosedPublisher).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe);
        assert!(error.to_string().contains("user.created"));
        assert!(error.to_string().contains("bus closed"));
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
