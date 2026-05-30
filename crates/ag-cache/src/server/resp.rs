//! RESP2 frame reader and response writer.
//!
//! Reads one command at a time from a buffered async reader.
//! Returns `None` on EOF (client disconnected).

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

/// Alias for the write half of a tokio TCP socket.
pub type Writer = OwnedWriteHalf;
/// Alias for a buffered reader over the read half of a tokio TCP socket.
pub type Reader = BufReader<OwnedReadHalf>;

/// Maximum number of elements in a RESP2 multibulk command (matches Redis
/// `proto-max-multibulk-len`). Bounds the per-command argument count so a tiny
/// header cannot force a large pre-allocation (DoS guard).
const MAX_ARRAY_ELEMENTS: usize = 1024 * 1024;

/// Maximum size of a single RESP2 bulk string (matches Redis
/// `proto-max-bulk-len`, 512 MiB). Bounds a single allocation so a tiny header
/// (`$999999999999`) cannot trigger a huge allocation / OOM (DoS guard).
const MAX_BULK_LEN: usize = 512 * 1024 * 1024;

/// Reads the next RESP2 command from the client.
///
/// Returns `None` if the client closed the connection, on malformed input, or
/// when an array/bulk-string length exceeds the configured maximum (the
/// connection is then dropped by the caller). Returns `Some(args)` where
/// `args[0]` is the command name.
///
/// Generic over the reader so it can be driven from a `Cursor` in tests and
/// fuzzing as well as from a live TCP socket in production.
pub async fn read_command<R>(reader: &mut R) -> Option<Vec<Vec<u8>>>
where
    R: AsyncBufReadExt + AsyncReadExt + Unpin,
{
    let mut line = String::new();
    let n = reader.read_line(&mut line).await.ok()?;
    if n == 0 {
        return None; // EOF
    }
    let line = line.trim_end_matches(['\r', '\n']);

    if let Some(count_str) = line.strip_prefix('*') {
        // Array format: *N\r\n$L\r\nDATA\r\n ...
        let count: usize = count_str.parse().ok()?;
        if count > MAX_ARRAY_ELEMENTS {
            return None; // reject oversized multibulk before allocating
        }
        // Never pre-allocate the attacker-controlled `count`; grow as elements
        // actually arrive.
        let mut args = Vec::with_capacity(count.min(16));
        for _ in 0..count {
            // Read $L line
            let mut len_line = String::new();
            reader.read_line(&mut len_line).await.ok()?;
            let len_line = len_line.trim_end_matches(['\r', '\n']);
            if let Some(len_str) = len_line.strip_prefix('$') {
                let len: usize = len_str.parse().ok()?;
                if len > MAX_BULK_LEN {
                    return None; // reject oversized bulk string before allocating
                }
                // `len <= MAX_BULK_LEN`, so `len + 2` cannot overflow.
                let mut buf = vec![0u8; len + 2];
                reader.read_exact(&mut buf).await.ok()?;
                buf.truncate(len); // drop trailing \r\n
                args.push(buf);
            } else {
                return None;
            }
        }
        Some(args)
    } else {
        // Inline format: COMMAND arg1 arg2
        if line.is_empty() {
            return None;
        }
        Some(
            line.split_whitespace()
                .map(|s| s.as_bytes().to_vec())
                .collect(),
        )
    }
}

/// Writes `+OK\r\n`.
pub async fn write_ok(w: &mut Writer) {
    let _ = w.write_all(b"+OK\r\n").await;
}

/// Writes `+{msg}\r\n`.
pub async fn write_simple(w: &mut Writer, msg: &str) {
    let _ = w.write_all(format!("+{msg}\r\n").as_bytes()).await;
}

/// Writes `-ERR {msg}\r\n`.
pub async fn write_error(w: &mut Writer, msg: &str) {
    let _ = w.write_all(format!("-ERR {msg}\r\n").as_bytes()).await;
}

/// Writes `:{n}\r\n`.
pub async fn write_integer(w: &mut Writer, n: i64) {
    let _ = w.write_all(format!(":{n}\r\n").as_bytes()).await;
}

/// Writes a RESP2 bulk string: `$L\r\nDATA\r\n` or `$-1\r\n` for nil.
pub async fn write_bulk(w: &mut Writer, data: Option<&[u8]>) {
    match data {
        None => {
            let _ = w.write_all(b"$-1\r\n").await;
        }
        Some(b) => {
            let _ = w.write_all(format!("${}\r\n", b.len()).as_bytes()).await;
            let _ = w.write_all(b).await;
            let _ = w.write_all(b"\r\n").await;
        }
    }
}

/// Writes a RESP2 array of bulk strings.
pub async fn write_array(w: &mut Writer, items: &[Vec<u8>]) {
    let _ = w
        .write_all(format!("*{}\r\n", items.len()).as_bytes())
        .await;
    for item in items {
        write_bulk(w, Some(item)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tokio::io::BufReader;

    async fn parse(input: &[u8]) -> Option<Vec<Vec<u8>>> {
        let mut reader = BufReader::new(Cursor::new(input.to_vec()));
        read_command(&mut reader).await
    }

    #[tokio::test]
    async fn parses_array_command() {
        let cmd = parse(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n").await.unwrap();
        assert_eq!(cmd, vec![b"GET".to_vec(), b"foo".to_vec()]);
    }

    #[tokio::test]
    async fn parses_inline_command() {
        let cmd = parse(b"PING\r\n").await.unwrap();
        assert_eq!(cmd, vec![b"PING".to_vec()]);
    }

    #[tokio::test]
    async fn eof_returns_none() {
        assert!(parse(b"").await.is_none());
    }

    #[tokio::test]
    async fn oversized_bulk_len_is_rejected_without_allocating() {
        // Regression: a tiny header claiming a ~1 TB bulk string must be
        // rejected, not allocated (unbounded-allocation DoS guard).
        assert!(parse(b"*1\r\n$999999999999\r\n").await.is_none());
    }

    #[tokio::test]
    async fn oversized_array_count_is_rejected() {
        assert!(parse(b"*999999999999\r\n").await.is_none());
    }

    #[tokio::test]
    async fn bulk_len_at_usize_max_does_not_overflow() {
        // `len + 2` must not overflow; oversized len is rejected first.
        let input = format!("*1\r\n${}\r\n", usize::MAX);
        assert!(parse(input.as_bytes()).await.is_none());
    }
}
