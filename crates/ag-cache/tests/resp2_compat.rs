//! Integration tests for the native RESP2 server.
//!
//! Each test binds the server on port 0, connects via TcpStream, sends raw
//! RESP2 commands, and asserts the response bytes. This validates protocol
//! compliance without mocking any layer.

use ag_cache::server::NativeCacheServer;
use ag_cache::l1::L1Cache;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

/// Spawns the server on port 0 and returns the bound port.
async fn spawn_server() -> u16 {
    let l1 = Arc::new(L1Cache::new(1000, Duration::from_secs(60)));
    let srv = NativeCacheServer::bind(0, l1).await.expect("bind must succeed");
    let port = srv.local_addr().expect("local_addr").port();
    tokio::spawn(srv.serve());
    port
}

/// Sends a raw RESP2 command and reads back the full response.
async fn send_recv(stream: &mut TcpStream, cmd: &str) -> String {
    stream.write_all(cmd.as_bytes()).await.unwrap();
    let mut buf = vec![0u8; 4096];
    tokio::time::sleep(Duration::from_millis(50)).await;
    let n = stream.try_read(&mut buf).unwrap_or(0);
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

#[tokio::test]
async fn ping_returns_pong() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();
    let resp = send_recv(&mut s, "*1\r\n$4\r\nPING\r\n").await;
    assert_eq!(resp, "+PONG\r\n");
}

#[tokio::test]
async fn ping_with_message_returns_bulk() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();
    let resp = send_recv(&mut s, "*2\r\n$4\r\nPING\r\n$5\r\nhello\r\n").await;
    assert_eq!(resp, "$5\r\nhello\r\n");
}

#[tokio::test]
async fn set_and_get_roundtrip() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();

    let set = send_recv(&mut s, "*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n").await;
    assert_eq!(set, "+OK\r\n");

    let get = send_recv(&mut s, "*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n").await;
    assert_eq!(get, "$3\r\nbar\r\n");
}

#[tokio::test]
async fn get_missing_key_returns_nil() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();
    let resp = send_recv(&mut s, "*2\r\n$3\r\nGET\r\n$7\r\nmissing\r\n").await;
    assert_eq!(resp, "$-1\r\n");
}

#[tokio::test]
async fn del_removes_key() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();

    send_recv(&mut s, "*3\r\n$3\r\nSET\r\n$1\r\nx\r\n$1\r\n1\r\n").await;
    let del = send_recv(&mut s, "*2\r\n$3\r\nDEL\r\n$1\r\nx\r\n").await;
    assert_eq!(del, ":1\r\n");

    let get = send_recv(&mut s, "*2\r\n$3\r\nGET\r\n$1\r\nx\r\n").await;
    assert_eq!(get, "$-1\r\n");
}

#[tokio::test]
async fn set_nx_does_not_overwrite() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();

    send_recv(&mut s, "*3\r\n$3\r\nSET\r\n$1\r\nk\r\n$5\r\nfirst\r\n").await;
    let nx = send_recv(
        &mut s,
        "*4\r\n$3\r\nSET\r\n$1\r\nk\r\n$6\r\nsecond\r\n$2\r\nNX\r\n",
    )
    .await;
    assert_eq!(nx, "$-1\r\n");

    let get = send_recv(&mut s, "*2\r\n$3\r\nGET\r\n$1\r\nk\r\n").await;
    assert_eq!(get, "$5\r\nfirst\r\n");
}

#[tokio::test]
async fn exists_counts_keys() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();

    send_recv(&mut s, "*3\r\n$3\r\nSET\r\n$2\r\na1\r\n$1\r\n1\r\n").await;
    send_recv(&mut s, "*3\r\n$3\r\nSET\r\n$2\r\na2\r\n$1\r\n2\r\n").await;

    let resp = send_recv(&mut s, "*3\r\n$6\r\nEXISTS\r\n$2\r\na1\r\n$2\r\na2\r\n").await;
    assert_eq!(resp, ":2\r\n");
}

#[tokio::test]
async fn dbsize_returns_live_count() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();

    send_recv(&mut s, "*3\r\n$3\r\nSET\r\n$2\r\nb1\r\n$1\r\n1\r\n").await;
    send_recv(&mut s, "*3\r\n$3\r\nSET\r\n$2\r\nb2\r\n$1\r\n2\r\n").await;

    let resp = send_recv(&mut s, "*1\r\n$6\r\nDBSIZE\r\n").await;
    assert_eq!(resp, ":2\r\n");
}

#[tokio::test]
async fn flushdb_clears_all() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();

    send_recv(&mut s, "*3\r\n$3\r\nSET\r\n$1\r\nc\r\n$1\r\n1\r\n").await;
    let flush = send_recv(&mut s, "*1\r\n$7\r\nFLUSHDB\r\n").await;
    assert_eq!(flush, "+OK\r\n");

    let get = send_recv(&mut s, "*2\r\n$3\r\nGET\r\n$1\r\nc\r\n").await;
    assert_eq!(get, "$-1\r\n");
}

#[tokio::test]
async fn mset_and_mget() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();

    send_recv(
        &mut s,
        "*5\r\n$4\r\nMSET\r\n$2\r\nm1\r\n$1\r\nA\r\n$2\r\nm2\r\n$1\r\nB\r\n",
    )
    .await;

    let resp = send_recv(
        &mut s,
        "*3\r\n$4\r\nMGET\r\n$2\r\nm1\r\n$2\r\nm2\r\n",
    )
    .await;
    assert!(resp.contains("$1\r\nA\r\n"));
    assert!(resp.contains("$1\r\nB\r\n"));
}

#[tokio::test]
async fn unsupported_command_returns_error() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();
    let resp = send_recv(&mut s, "*1\r\n$4\r\nEVAL\r\n").await;
    assert!(resp.starts_with("-ERR"));
    assert!(resp.contains("unsupported"));
}

#[tokio::test]
async fn set_with_ex_and_ttl() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();

    // SET mykey myval EX 100
    send_recv(
        &mut s,
        "*5\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$5\r\nmyval\r\n$2\r\nEX\r\n$3\r\n100\r\n",
    )
    .await;

    let ttl = send_recv(&mut s, "*2\r\n$3\r\nTTL\r\n$5\r\nmykey\r\n").await;
    assert!(ttl.starts_with(':'));
    let secs: i64 = ttl.trim_matches(['\r', '\n', ':']).parse().unwrap();
    assert!((98..=100).contains(&secs), "TTL was {secs}");
}

#[tokio::test]
async fn inline_ping() {
    let port = spawn_server().await;
    let mut s = TcpStream::connect(format!("127.0.0.1:{port}")).await.unwrap();
    let resp = send_recv(&mut s, "PING\r\n").await;
    assert_eq!(resp, "+PONG\r\n");
}
