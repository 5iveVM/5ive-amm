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
//! NOTE: Native stdio transport is intentionally deferred.
//! The browser/WASM path is the supported integration, including workspace symbol
//! aggregation via per-file queries in the frontend.

#[cfg(feature = "native")]
use five_lsp::FiveLanguageServer;

#[cfg(feature = "native")]
use tower_lsp::Server;

#[cfg(feature = "native")]
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "native")]
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    // Native transport remains intentionally disabled in this pass.
    eprintln!("Five LSP native binary is not yet enabled.");
    eprintln!("Use the WASM/browser integration path for active language features.");
    std::process::exit(1);
}

#[cfg(feature = "native")]
#[tokio::main]
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

    // TODO: Implement proper tower-lsp stdio transport
    // Current issue: tower-lsp 0.20 Server::new() requires understanding correct socket pattern
}
