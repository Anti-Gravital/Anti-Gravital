//! Minimal dependency-free HTTP mock for the domain-link example.
//!
//! It stands in for a generated Rust service. It binds to `BIND` (default
//! `127.0.0.1:8080`), answers `GET /health` and any other GET with a small
//! body that echoes the request Host header and `APP_ENV`, so an offline
//! `curl --resolve` check proves which environment answered.

use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let bind = env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "local".to_string());
    let listener = TcpListener::bind(&bind).expect("cannot bind BIND address");
    println!("domain-link-demo listening on http://{} (env={})", bind, app_env);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let env_name = app_env.clone();
                std::thread::spawn(move || handle(&mut stream, &env_name));
            }
            Err(err) => eprintln!("accept error: {}", err),
        }
    }
}

fn handle(stream: &mut TcpStream, app_env: &str) {
    let mut buf = [0u8; 8192];
    let read = match stream.read(&mut buf) {
        Ok(read) => read,
        Err(_) => return,
    };
    if read == 0 {
        return;
    }

    let request = String::from_utf8_lossy(&buf[..read]);
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    let raw_host = lines
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("host") {
                Some(value.trim())
            } else {
                None
            }
        })
        .unwrap_or("unknown");
    // Keep the reflected Host header from injecting response headers.
    let host: String = raw_host.chars().filter(|c| *c != '\r' && *c != '\n').collect();

    let (status, reason) = if method == "GET" {
        ("200 OK", "ok")
    } else {
        ("405 Method Not Allowed", "method-not-allowed")
    };
    let body = format!("{} host={} path={} env={}\n", reason, host, path, app_env);
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nX-Served-Host: {}\r\nX-App-Env: {}\r\nConnection: close\r\n\r\n{}",
        status,
        body.len(),
        host,
        app_env,
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}
