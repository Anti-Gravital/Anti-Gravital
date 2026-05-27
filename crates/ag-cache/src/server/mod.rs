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
use tracing::{debug, error, info, warn};

use crate::l1::L1Cache;
use cmd::ExpiryMap;
use resp::{read_command, Reader, Writer};

/// An in-process TCP server that speaks RESP2.
pub struct NativeCacheServer {
    listener: TcpListener,
    cache: Arc<L1Cache>,
    expiry: Arc<ExpiryMap>,
}

impl NativeCacheServer {
    /// Binds the server to `127.0.0.1:{port}`.
    ///
    /// Use port `0` in tests to let the OS choose a free port.
    pub async fn bind(port: u16, cache: Arc<L1Cache>) -> io::Result<Self> {
        let addr = format!("127.0.0.1:{port}");
        let listener = TcpListener::bind(&addr).await?;
        let local = listener.local_addr()?;
        info!(addr = %local, "native RESP2 server listening");
        Ok(Self {
            listener,
            cache,
            expiry: Arc::new(DashMap::new()),
        })
    }

    /// Returns the local address the server is listening on.
    pub fn local_addr(&self) -> io::Result<std::net::SocketAddr> {
        self.listener.local_addr()
    }

    /// Runs the accept loop. Spawns one task per connection.
    /// Blocks until the listener is dropped or an accept error occurs.
    pub async fn serve(self) {
        let cache = self.cache;
        let expiry = self.expiry;
        loop {
            match self.listener.accept().await {
                Ok((stream, peer)) => {
                    debug!(%peer, "new RESP2 connection");
                    let c = Arc::clone(&cache);
                    let e = Arc::clone(&expiry);
                    tokio::spawn(async move {
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
