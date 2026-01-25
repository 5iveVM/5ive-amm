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
//! NOTE: This binary is currently disabled pending tower-lsp transport implementation.
//! The LSP library is fully functional and can be used via WASM or by implementing
//! a custom transport layer.

#[cfg(feature = "native")]
use five_lsp::FiveLanguageServer;

#[cfg(feature = "native")]
use tower_lsp::Server;

#[cfg(feature = "native")]
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "native")]
use tracing_subscriber::util::SubscriberInitExt;

fn main() {
    // Disabled for now - native binary transport pending
    eprintln!("Five LSP native binary is not yet enabled.");
    eprintln!("The LSP library is available for use via WASM or custom transport.");
    std::process::exit(1);
}

#[cfg(feature = "native")]
#[tokio::main]
async fn native_main() {
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

    // TODO: Implement proper tower-lsp stdio transport
    // Current issue: tower-lsp 0.20 Server::new() requires understanding correct socket pattern
}
