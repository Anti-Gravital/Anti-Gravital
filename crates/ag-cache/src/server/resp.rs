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

/// Reads the next RESP2 command from the client.
///
/// Returns `None` if the client closed the connection.
/// Returns `Some(args)` where `args[0]` is the command name (uppercase bytes).
pub async fn read_command(reader: &mut Reader) -> Option<Vec<Vec<u8>>> {
    let mut line = String::new();
    let n = reader.read_line(&mut line).await.ok()?;
    if n == 0 {
        return None; // EOF
    }
    let line = line.trim_end_matches(['\r', '\n']);

    if let Some(count_str) = line.strip_prefix('*') {
        // Array format: *N\r\n$L\r\nDATA\r\n ...
        let count: usize = count_str.parse().ok()?;
        let mut args = Vec::with_capacity(count);
        for _ in 0..count {
            // Read $L line
            let mut len_line = String::new();
            reader.read_line(&mut len_line).await.ok()?;
            let len_line = len_line.trim_end_matches(['\r', '\n']);
            if let Some(len_str) = len_line.strip_prefix('$') {
                let len: usize = len_str.parse().ok()?;
                // Read DATA + \r\n
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
    let _ = w
        .write_all(format!("+{msg}\r\n").as_bytes())
        .await;
}

/// Writes `-ERR {msg}\r\n`.
pub async fn write_error(w: &mut Writer, msg: &str) {
    let _ = w
        .write_all(format!("-ERR {msg}\r\n").as_bytes())
        .await;
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
            let _ = w
                .write_all(format!("${}\r\n", b.len()).as_bytes())
                .await;
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
    // Parser is tested via the integration test (Task 7) through a real
    // TCP connection. The response writers are pure I/O and tested there too.
}
