//! Binario del servidor LSP Anti-Gravital.
//!
//! Escucha en stdin/stdout con el protocolo LSP. El cliente (VS Code, etc.)
//! lanza este proceso como hijo y se comunica por stdio.

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
