//! Anti-Gravital LSP server binary.
//!
//! Listens on stdin/stdout with the LSP protocol. The client (VS Code, etc.)
//! launches this process as a child and communicates over stdio.

mod backend;

use backend::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
