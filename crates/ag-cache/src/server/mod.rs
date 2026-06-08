//! Native RESP2 cache server.
//!
//! When the `native-server` feature is enabled and `CacheConfig::native_server_enabled`
//! is `true`, `AgCache::new` spawns a `NativeCacheServer` in a background task.
//! The server speaks RESP2 and accepts connections on `127.0.0.1:{port}`.
//!
//! External clients (redis-cli, redis-rs, ioredis) connect without knowing
//! they are talking to an in-process cache instead of Redis.

pub mod cmd;
pub mod resp;

use std::io;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use crate::l1::L1Cache;
use cmd::ExpiryMap;
use resp::{read_command, Reader, Writer};

/// Default maximum number of concurrent native RESP2 connections.
pub const DEFAULT_MAX_CONNECTIONS: usize = 1_024;

/// An in-process TCP server that speaks RESP2.
pub struct NativeCacheServer {
    listener: TcpListener,
    cache: Arc<L1Cache>,
    expiry: Arc<ExpiryMap>,
    connection_limit: Arc<Semaphore>,
    max_connections: usize,
}

impl NativeCacheServer {
    /// Binds the server to `127.0.0.1:{port}`.
    ///
    /// Use port `0` in tests to let the OS choose a free port.
    pub async fn bind(port: u16, cache: Arc<L1Cache>) -> io::Result<Self> {
        Self::bind_with_limit(port, cache, DEFAULT_MAX_CONNECTIONS).await
    }

    /// Binds the server with an explicit concurrent connection limit.
    pub async fn bind_with_limit(
        port: u16,
        cache: Arc<L1Cache>,
        max_connections: usize,
    ) -> io::Result<Self> {
        if max_connections == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native RESP2 connection limit must be greater than zero",
            ));
        }

        let addr = format!("127.0.0.1:{port}");
        let listener = TcpListener::bind(&addr).await?;
        let local = listener.local_addr()?;
        info!(addr = %local, max_connections, "native RESP2 server listening");
        Ok(Self {
            listener,
            cache,
            expiry: Arc::new(DashMap::new()),
            connection_limit: Arc::new(Semaphore::new(max_connections)),
            max_connections,
        })
    }

    /// Returns the local address the server is listening on.
    pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Returns the configured concurrent connection limit.
    pub fn max_connections(&self) -> usize {
        self.max_connections
    }

    /// Runs the accept loop. Spawns one task per admitted connection.
    /// Blocks until the listener is dropped or an accept error occurs.
    pub async fn serve(self) {
        let cache = self.cache;
        let expiry = self.expiry;
        let connection_limit = self.connection_limit;
        loop {
            match self.listener.accept().await {
                Ok((stream, peer)) => {
                    let permit = match Arc::clone(&connection_limit).try_acquire_owned() {
                        Ok(permit) => permit,
                        Err(_) => {
                            warn!(%peer, "native RESP2 connection limit reached");
                            drop(stream);
                            continue;
                        }
                    };
                    debug!(%peer, "new RESP2 connection");
                    let c = Arc::clone(&cache);
                    let e = Arc::clone(&expiry);
                    tokio::spawn(async move {
                        let _permit = permit;
                        if let Err(err) = handle_connection(stream, c, e).await {
                            warn!(%err, "RESP2 connection error");
                        }
                    });
                }
                Err(err) => {
                    error!(%err, "accept error — stopping native RESP2 server");
                    break;
                }
            }
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    cache: Arc<L1Cache>,
    expiry: Arc<ExpiryMap>,
) -> io::Result<()> {
    let (read_half, write_half) = stream.into_split();
    let mut reader: Reader = tokio::io::BufReader::new(read_half);
    let mut writer: Writer = write_half;

    while let Some(args) = read_command(&mut reader).await {
        if args.is_empty() {
            continue;
        }
        let command = String::from_utf8_lossy(&args[0]).to_uppercase();
        match command.as_str() {
            "PING" => cmd::cmd_ping(&args, &mut writer).await,
            "GET" => cmd::cmd_get(&args, &cache, &expiry, &mut writer).await,
            "SET" => cmd::cmd_set(&args, &cache, &expiry, &mut writer).await,
            "DEL" => cmd::cmd_del(&args, &cache, &expiry, &mut writer).await,
            "EXISTS" => cmd::cmd_exists(&args, &cache, &expiry, &mut writer).await,
            "MGET" => cmd::cmd_mget(&args, &cache, &expiry, &mut writer).await,
            "MSET" => cmd::cmd_mset(&args, &cache, &expiry, &mut writer).await,
            "EXPIRE" => cmd::cmd_expire(&args, &cache, &expiry, &mut writer).await,
            "TTL" => cmd::cmd_ttl(&args, &cache, &expiry, &mut writer).await,
            "KEYS" => cmd::cmd_keys(&args, &expiry, &mut writer).await,
            "FLUSHDB" => cmd::cmd_flushdb(&cache, &expiry, &mut writer).await,
            "DBSIZE" => cmd::cmd_dbsize(&expiry, &mut writer).await,
            "COMMAND" => cmd::cmd_command(&mut writer).await,
            other => {
                resp::write_error(
                    &mut writer,
                    &format!("unsupported command '{other}' in ag-cache native server"),
                )
                .await;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn cache() -> Arc<L1Cache> {
        Arc::new(L1Cache::new(16, Duration::from_secs(60)))
    }

    #[tokio::test]
    async fn bind_with_limit_rejects_zero() {
        let error = match NativeCacheServer::bind_with_limit(0, cache(), 0).await {
            Ok(_) => panic!("zero connection limit must fail"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn bind_with_limit_reports_configured_limit() {
        let server = NativeCacheServer::bind_with_limit(0, cache(), 7)
            .await
            .unwrap();
        assert_eq!(server.max_connections(), 7);
    }

    #[tokio::test]
    async fn connection_over_limit_is_closed() {
        use tokio::io::AsyncReadExt;

        let server = NativeCacheServer::bind_with_limit(0, cache(), 1)
            .await
            .unwrap();
        let address = server.local_addr().unwrap();
        tokio::spawn(server.serve());

        let _first = TcpStream::connect(address).await.unwrap();
        tokio::time::sleep(Duration::from_millis(25)).await;

        let mut second = TcpStream::connect(address).await.unwrap();
        let mut byte = [0_u8; 1];
        let read = tokio::time::timeout(Duration::from_secs(1), second.read(&mut byte))
            .await
            .expect("over-limit connection was not closed")
            .unwrap();
        assert_eq!(read, 0);
    }
}
