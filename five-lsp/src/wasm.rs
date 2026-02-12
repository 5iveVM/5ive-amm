//! WASM bindings for browser-based LSP (Monaco Editor integration)
//!
//! This module provides JavaScript/WebAssembly bindings that allow the LSP
//! to run in the browser, communicating with Monaco Editor directly.
//!
//! The module exposes `FiveLspWasm` which wraps the compiler bridge and
//! allows TypeScript to call diagnostics in real-time as the user edits.
//!
//! Usage (from TypeScript):
//! ```typescript
//! import * as wasmModule from 'five-lsp-wasm';
//!
//! const lsp = wasmModule.FiveLspWasm.new();
//! const diagnostics = lsp.get_diagnostics('file:///test.v', sourceCode);
//! console.log(diagnostics);  // Array of Diagnostic objects
//! ```

use crate::bridge::CompilerBridge;
use crate::features::{hover, completion, goto_definition, find_references, semantic, code_actions, document_symbols, workspace_symbols, rename};
use lsp_types::Url;
use wasm_bindgen::prelude::*;

/// WASM wrapper for the Five LSP compiler bridge
///
/// This is the main entry point for WASM clients. It wraps the Rust
/// CompilerBridge and exposes it to JavaScript via wasm-bindgen.
#[wasm_bindgen]
pub struct FiveLspWasm {
    bridge: CompilerBridge,
}

#[wasm_bindgen]
impl FiveLspWasm {
    /// Create a new LSP instance
    ///
    /// This initializes the compiler bridge and prepares it for use.
    #[wasm_bindgen(constructor)]
    pub fn new() -> FiveLspWasm {
        // Set up panic hooks for better error reporting in browser
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        FiveLspWasm {
            bridge: CompilerBridge::new(),
        }
    }

    /// Get diagnostics for a Five DSL file
    ///
    /// # Arguments
    /// * `uri` - File URI (e.g., "file:///test.v")
    /// * `source` - The source code to analyze
    ///
    /// # Returns
    /// A JSON string containing an array of diagnostics, or an error message
    ///
    /// # Example
    /// ```typescript
    /// const lsp = FiveLspWasm.new();
    /// const result = lsp.get_diagnostics('file:///test.v', 'init { let x = 5; }');
    /// const diagnostics = JSON.parse(result);
    /// ```
    pub fn get_diagnostics(&mut self, uri: &str, source: &str) -> Result<String, JsValue> {
        // Parse URI
        let url = lsp_types::Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Get diagnostics from bridge
        let diagnostics = self
            .bridge
            .get_diagnostics(&url, source)
            .map_err(|e| JsValue::from_str(&format!("Compilation error: {}", e)))?;

        // Convert to JSON for passing to JavaScript
        serde_json::to_string(&diagnostics)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get hover information for a symbol at the given position
    ///
    /// # Arguments
    /// * `uri` - File URI (e.g., "file:///test.v")
    /// * `source` - The source code
    /// * `line` - 0-indexed line number
    /// * `character` - 0-indexed character position
    ///
    /// # Returns
    /// A JSON string containing hover information, or error message
    ///
    /// # Example
    /// ```typescript
    /// const lsp = FiveLspWasm.new();
    /// const result = lsp.get_hover('file:///test.v', 'let x = 5;', 0, 4);
    /// const hover = result ? JSON.parse(result) : null;
    /// ```
    pub fn get_hover(
        &mut self,
        uri: &str,
        source: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<String>, JsValue> {
        // Parse URI
        let url = lsp_types::Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Create position
        let position = lsp_types::Position { line, character };

        // Get hover from bridge
        let hover_info = hover::get_hover(&self.bridge, source, position, &url);

        // Convert to JSON for passing to JavaScript
        if let Some(hover) = hover_info {
            let json = serde_json::to_string(&hover)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    /// Get completion suggestions at the given position
    ///
    /// # Arguments
    /// * `uri` - File URI (e.g., "file:///test.v")
    /// * `source` - The source code
    /// * `line` - 0-indexed line number
    /// * `character` - 0-indexed character position
    ///
    /// # Returns
    /// A JSON string containing CompletionList with suggestions
    ///
    /// # Example
    /// ```typescript
    /// const lsp = FiveLspWasm.new();
    /// const result = lsp.get_completions('file:///test.v', 'let x = ', 0, 8);
    /// const completions = JSON.parse(result);
    /// ```
    pub fn get_completions(
        &self,
        uri: &str,
        source: &str,
        line: u32,
        character: u32,
    ) -> Result<String, JsValue> {
        // Parse URI
        let url = lsp_types::Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Get completions from feature module
        let completion_list = completion::get_completions(
            &self.bridge,
            source,
            line as usize,
            character as usize,
            &url,
        );

        // Convert to JSON for passing to JavaScript
        serde_json::to_string(&completion_list)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get go-to-definition location for a symbol at the given position
    ///
    /// # Arguments
    /// * `uri` - File URI (e.g., "file:///test.v")
    /// * `source` - The source code
    /// * `line` - 0-indexed line number
    /// * `character` - 0-indexed character position
    ///
    /// # Returns
    /// A JSON string containing Location if definition found, null otherwise
    ///
    /// # Example
    /// ```typescript
    /// const lsp = FiveLspWasm.new();
    /// const result = lsp.get_definition('file:///test.v', 'function foo() {}', 0, 9);
    /// const location = result ? JSON.parse(result) : null;
    /// ```
    pub fn get_definition(
        &mut self,
        uri: &str,
        source: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<String>, JsValue> {
        // Parse URI
        let url = Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Get definition from feature module
        let location = goto_definition::get_definition(
            &mut self.bridge,
            &url,
            source,
            line,
            character,
        );

        // Convert to JSON for passing to JavaScript
        if let Some(loc) = location {
            let json = serde_json::to_string(&loc)
                .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
            Ok(Some(json))
        } else {
            Ok(None)
        }
    }

    /// Find all references to a symbol at the given position
    ///
    /// # Arguments
    /// * `uri` - File URI (e.g., "file:///test.v")
    /// * `source` - The source code
    /// * `line` - 0-indexed line number
    /// * `character` - 0-indexed character position
    ///
    /// # Returns
    /// A JSON string containing an array of Locations where the symbol is referenced
    ///
    /// # Example
    /// ```typescript
    /// const lsp = FiveLspWasm.new();
    /// const result = lsp.find_references('file:///test.v', 'let x = 5; let y = x;', 0, 4);
    /// const references = JSON.parse(result);  // Array of Location objects
    /// ```
    pub fn find_references(
        &mut self,
        uri: &str,
        source: &str,
        line: u32,
        character: u32,
    ) -> Result<String, JsValue> {
        // Parse URI
        let url = Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Get references from feature module
        let references = find_references::find_references(
            &mut self.bridge,
            &url,
            source,
            line as usize,
            character as usize,
        );

        // Convert to JSON for passing to JavaScript
        serde_json::to_string(&references)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get semantic tokens for syntax highlighting
    ///
    /// Returns an array of semantic tokens for AST-based syntax highlighting.
    /// Provides more accurate highlighting than regex-based approaches.
    pub fn get_semantic_tokens(
        &mut self,
        uri: &str,
        source: &str,
    ) -> Result<String, JsValue> {
        // Parse URI
        let url = Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Get semantic tokens
        let tokens = semantic::get_semantic_tokens(&mut self.bridge, source, &url);

        // Convert to JSON
        serde_json::to_string(&tokens)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get document symbols for outline view
    ///
    /// Returns all top-level definitions (functions, variables, accounts) for
    /// display in the editor's outline/navigator panel.
    pub fn get_document_symbols(
        &mut self,
        uri: &str,
        source: &str,
    ) -> Result<String, JsValue> {
        // Parse URI
        let url = Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Get document symbols
        let symbols = document_symbols::get_document_symbols(&mut self.bridge, source, &url);

        // Convert to JSON
        serde_json::to_string(&symbols)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get workspace symbols matching a query
    ///
    /// Searches for symbols across the workspace (or current file) that match the query string.
    /// Supports case-insensitive substring matching.
    ///
    /// # Arguments
    /// * `uri` - File URI
    /// * `source` - The source code to search
    /// * `query` - Search query (case-insensitive substring match)
    ///
    /// # Returns
    /// JSON string containing array of SymbolInformation objects
    pub fn get_workspace_symbols(
        &mut self,
        uri: &str,
        source: &str,
        query: &str,
    ) -> Result<String, JsValue> {
        // Parse URI
        let url = Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Get workspace symbols matching query
        let symbols = workspace_symbols::workspace_symbols(&mut self.bridge, source, query, &url);

        // Convert to JSON
        serde_json::to_string(&symbols)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Get code actions for a diagnostic
    ///
    /// Provides quick fix suggestions for a diagnostic at the given position.
    pub fn get_code_actions(
        &self,
        uri: &str,
        source: &str,
        diagnostic_json: &str,
    ) -> Result<String, JsValue> {
        // Parse URI
        let url = Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Parse diagnostic from JSON
        let diagnostic: lsp_types::Diagnostic = serde_json::from_str(diagnostic_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse diagnostic: {}", e)))?;

        // Get code actions
        let actions = code_actions::get_code_actions(source, &diagnostic, &url);

        // Convert to JSON
        serde_json::to_string(&actions)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Prepare a rename operation
    ///
    /// Validates that a symbol at the given position can be renamed and returns its name.
    ///
    /// # Arguments
    /// * `uri` - File URI (for multi-file context)
    /// * `source` - The source code
    /// * `line` - 0-indexed line number
    /// * `character` - 0-indexed character position
    pub fn prepare_rename(
        &self,
        uri: &str,
        source: &str,
        line: u32,
        character: u32,
    ) -> Result<Option<String>, JsValue> {
        // Parse URI (validate but currently not used for single-file analysis)
        let _url = Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        match rename::prepare_rename(source, line as usize, character as usize) {
            Some(name) => Ok(Some(name)),
            None => Ok(None),
        }
    }

    /// Rename a symbol across all occurrences
    ///
    /// Performs a safe rename of a symbol, updating all references to it.
    pub fn rename(
        &mut self,
        uri: &str,
        source: &str,
        line: u32,
        character: u32,
        new_name: &str,
    ) -> Result<Option<String>, JsValue> {
        // Parse URI
        let url = Url::parse(uri)
            .map_err(|e| JsValue::from_str(&format!("Invalid URI: {}", e)))?;

        // Perform rename
        match rename::rename(&mut self.bridge, source, line as usize, character as usize, new_name, &url) {
            Some(edit) => {
                let json = serde_json::to_string(&edit)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
                Ok(Some(json))
            }
            None => Ok(None),
        }
    }

    /// Clear all caches
    ///
    /// Useful after large changes or when memory needs to be freed.
    /// This forces recompilation on the next analysis.
    pub fn clear_caches(&mut self) {
        self.bridge.clear_caches();
    }
}
