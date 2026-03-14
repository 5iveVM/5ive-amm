//! Five DSL Language Server Protocol (LSP) server
//!
//! This is the native binary entrypoint for VSCode and other LSP clients.
//! It uses stdio for communication with the client.
//!
//! Usage:
//! ```bash
//! five-lsp  # Waits for LSP messages on stdin
//! ```
//!
//! NOTE: Native transport is available behind the `native` feature.
//! The browser/WASM path remains supported for web integrations.

#[cfg(feature = "native")]
use five_lsp::FiveLanguageServer;

#[cfg(feature = "native")]
use tower_lsp::{LspService, Server};

#[cfg(feature = "native")]
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "native")]
use tracing_subscriber::util::SubscriberInitExt;

#[cfg(feature = "native")]
fn main() {
    native_main();
}

#[cfg(feature = "native")]
#[tokio::main(flavor = "current_thread")]
async fn native_main() {
    // Initialize logging (writes to stderr so LSP protocol on stdout isn't polluted)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Five DSL LSP Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(FiveLanguageServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(not(feature = "native"))]
fn main() {
    eprintln!("five-lsp native server is disabled. Rebuild with: --features native");
    std::process::exit(1);
}
