//! Diagnostics feature (Phase 1: MVP)
//!
//! Provides real-time error and warning reporting as the user types.
//! Reuses the five-dsl-compiler's error formatting infrastructure.

use crate::bridge::CompilerBridge;
use crate::error::LspError;
use lsp_types::Url;

/// Get diagnostics for a Five DSL file
pub fn get_diagnostics(
    bridge: &mut CompilerBridge,
    uri: &Url,
    source: &str,
) -> Result<Vec<lsp_types::Diagnostic>, LspError> {
    bridge.get_diagnostics(uri, source)
}
