//! Five DSL Language Server Protocol (LSP) implementation
//!
//! This LSP server integrates the Five DSL compiler to provide real-time diagnostics
//! and language features for both desktop (VSCode via stdio) and browser (Monaco via WASM).
//!
//! ## Architecture
//!
//! The LSP is organized into:
//! - `CompilerBridge`: Bridges between LSP and the five-dsl-compiler
//! - `Server`: Main LSP server implementing tower-lsp traits
//! - Document state management and workspace support
//! - Feature modules: diagnostics, completion, hover, go-to-definition, etc.

pub mod bridge;
pub mod document;
pub mod error;
pub mod features;
pub mod workspace;

#[cfg(feature = "native")]
pub mod server;

pub use bridge::CompilerBridge;
pub use error::LspError;

#[cfg(feature = "native")]
pub use server::FiveLanguageServer;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(target_os = "emscripten"),
    not(target_os = "wasi")
))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
