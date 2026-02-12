//! Five DSL Language Server Protocol (LSP) implementation.
//!
//! Integrates the Five DSL compiler to provide diagnostics and language features
//! for desktop (stdio) and browser (WASM) environments.

pub mod bridge;
pub mod document;
pub mod error;
pub mod features;
pub mod semantic;
pub mod workspace;

#[cfg(feature = "native")]
pub mod server;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use bridge::CompilerBridge;
pub use error::LspError;

#[cfg(feature = "native")]
pub use server::FiveLanguageServer;

#[cfg(target_arch = "wasm32")]
pub use wasm::FiveLspWasm;
