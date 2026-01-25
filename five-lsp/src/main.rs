#![cfg(feature = "native")]

//! Five DSL Language Server Protocol (LSP) server
//!
//! This is the native binary entrypoint for VSCode and other LSP clients.
//! It uses stdio for communication with the client.
//!
//! Usage:
//! ```bash
//! five-lsp  # Waits for LSP messages on stdin
//! ```

use five_lsp::FiveLanguageServer;
use tower_lsp::Server;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[tokio::main]
async fn main() {
    // Initialize logging (writes to stderr so LSP protocol on stdout isn't polluted)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
            ),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Five DSL LSP Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // Create the LSP transport using socket pair approach
    let (client_transport, server_transport) = tokio::io::duplex(1024);

    // Build the service with the input/output streams
    let (service, socket) = tower_lsp::Server::new(
        stdin,
        stdout,
        tower_lsp::jsonrpc::MethodRouter::new(),
    ).build_socket(server_transport);

    let client = service.client_clone();
    let server = FiveLanguageServer::new(client);

    service.run(socket, server).await
}
