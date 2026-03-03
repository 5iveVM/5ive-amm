/**
 * Five DSL LSP Client Wrapper
 *
 * Provides a TypeScript wrapper around the WASM-compiled Five LSP bindings.
 * This client allows real-time diagnostics and language features in Monaco Editor.
 *
 * Usage:
 * ```typescript
 * const client = new FiveLspClient();
 * await client.initialize();
 *
 * const diagnostics = client.getDiagnostics('file:///test.v', sourceCode);
 * console.log(diagnostics);  // Array of Diagnostic objects
 * ```
 */

// WASM module is loaded dynamically from /wasm/five_lsp.js
// This import statement is not needed as we load it dynamically at runtime

import type {
  LspDiagnostic,
  LspHover,
  LspCompletionList,
  LspLocation,
  LspWorkspaceEdit,
  LspCodeAction,
  LspSemanticToken,
  LspDocumentSymbol,
  LspSymbolInformation,
  LspSignatureHelp,
  LspInlayHint,
} from '../types/lsp';

// Re-export for backward compatibility
export type { LspDiagnostic as Diagnostic };

/**
 * Five LSP Client for browser environments
 *
 * Wraps the WASM-compiled LSP and provides async initialization and error handling.
 */
export class FiveLspClient {
  private wasmModule: any = null;
  private lsp: any = null;
  private initialized = false;
  private readonly trackedDocuments = new Map<string, string>();

  /**
   * Initialize the LSP client
   *
   * This must be called before using any other methods.
   * It loads the WASM module and creates the LSP instance.
   *
   * @throws Error if WASM module cannot be loaded
   *
   * @example
   * ```typescript
   * const client = new FiveLspClient();
   * await client.initialize();
   * ```
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    try {
      // Dynamically import the WASM module using absolute URL to avoid webpack interference
      const wasmUrl = new URL('/wasm/five_lsp.js', window.location.href).href;
      this.wasmModule = await import(/* webpackIgnore: true */ wasmUrl);

      if (!this.wasmModule) {
        throw new Error('Failed to load WASM module');
      }

      // Initialize the WASM module first (the default export is the init function)
      if (this.wasmModule.default) {
        await this.wasmModule.default();
        console.log('[FiveLspClient] WASM module initialized');
      }

      if (!this.wasmModule.FiveLspWasm) {
        console.error('[FiveLspClient] WASM module exports:', Object.keys(this.wasmModule));
        throw new Error('Failed to load FiveLspWasm from WASM module');
      }

      // Create LSP instance (FiveLspWasm is a constructor class)
      this.lsp = new this.wasmModule.FiveLspWasm();
      this.initialized = true;

      console.log('[FiveLspClient] Initialized successfully');
    } catch (error) {
      console.error('[FiveLspClient] Failed to initialize:', error);
      throw new Error(`Failed to initialize Five LSP: ${error}`);
    }
  }

  /**
   * Ensure the client is initialized
   *
   * @throws Error if not initialized
   */
  private ensureInitialized(): void {
    if (!this.initialized || !this.lsp) {
      throw new Error(
        'Five LSP client not initialized. Call initialize() first.'
      );
    }
  }

  /**
   * Get hover information for a symbol at the given position
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @param line - 0-indexed line number
   * @param character - 0-indexed character position
   * @returns JSON string containing hover information, or null if no symbol found
   * @throws Error if there's a compilation error
   *
   * @example
   * ```typescript
   * const hoverJson = await client.getHover('file:///main.v', sourceCode, 0, 4);
   * if (hoverJson) {
   *   const hover = JSON.parse(hoverJson);
   *   console.log(hover.contents); // Type information
   * }
   * ```
   */
  async getHover(
    uri: string,
    source: string,
    line: number,
    character: number
  ): Promise<string | null> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.get_hover(uri, source, line, character);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error getting hover:', error);
      throw new Error(`Failed to get hover information: ${error}`);
    }
  }

  /**
   * Get diagnostics for a Five DSL file
   *
   * Returns a list of compilation errors and warnings for the given source code.
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code to analyze
   * @returns Array of Diagnostic objects
   * @throws Error if there's a compilation error
   *
   * @example
   * ```typescript
   * const diagnostics = client.getDiagnostics('file:///main.v', sourceCode);
   * diagnostics.forEach(d => {
   *   console.log(`Line ${d.range.start.line}: ${d.message}`);
   * });
   * ```
   */
  getDiagnostics(uri: string, source: string): LspDiagnostic[] {
    this.ensureInitialized();

    try {
      const result = this.lsp.get_diagnostics(uri, source);

      // Result is a JSON string from Rust
      const diagnostics = JSON.parse(result);

      if (!Array.isArray(diagnostics)) {
        console.warn('[FiveLspClient] Unexpected response format:', result);
        return [];
      }

      return diagnostics as LspDiagnostic[];
    } catch (error) {
      console.error('[FiveLspClient] Error getting diagnostics:', error);
      throw new Error(`Failed to get diagnostics: ${error}`);
    }
  }

  /**
   * Get completion suggestions at the given position
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @param line - 0-indexed line number
   * @param character - 0-indexed character position
   * @returns JSON string containing CompletionList with suggestions
   * @throws Error if there's a compilation error
   *
   * @example
   * ```typescript
   * const completionsJson = await client.getCompletions('file:///main.v', sourceCode, 0, 4);
   * const completionList = JSON.parse(completionsJson);
   * console.log(completionList.items); // Array of CompletionItem objects
   * ```
   */
  async getCompletions(
    uri: string,
    source: string,
    line: number,
    character: number
  ): Promise<string> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.get_completions(uri, source, line, character);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error getting completions:', error);
      throw new Error(`Failed to get completions: ${error}`);
    }
  }

  /**
   * Get go-to-definition location for a symbol at the given position
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @param line - 0-indexed line number
   * @param character - 0-indexed character position
   * @returns JSON string containing Location if definition found, or null if not
   * @throws Error if there's a compilation error
   *
   * @example
   * ```typescript
   * const locationJson = await client.getDefinition('file:///main.v', sourceCode, 0, 9);
   * if (locationJson) {
   *   const location = JSON.parse(locationJson);
   *   console.log(location.range); // Range of the definition
   * }
   * ```
   */
  async getDefinition(
    uri: string,
    source: string,
    line: number,
    character: number
  ): Promise<string | null> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.get_definition(uri, source, line, character);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error getting definition:', error);
      throw new Error(`Failed to get definition: ${error}`);
    }
  }

  /**
   * Find all references to a symbol at the given position
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @param line - 0-indexed line number
   * @param character - 0-indexed character position
   * @returns JSON string containing an array of Locations where symbol is referenced
   * @throws Error if there's a compilation error
   *
   * @example
   * ```typescript
   * const referencesJson = await client.findReferences('file:///main.v', sourceCode, 0, 4);
   * const references = JSON.parse(referencesJson);  // Array of Location objects
   * references.forEach(ref => {
   *   console.log(`Found at line ${ref.range.start.line}`);
   * });
   * ```
   */
  async findReferences(
    uri: string,
    source: string,
    line: number,
    character: number
  ): Promise<string> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.find_references(uri, source, line, character);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error finding references:', error);
      throw new Error(`Failed to find references: ${error}`);
    }
  }

  /**
   * Get semantic tokens for syntax highlighting
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @returns JSON string containing array of SemanticToken objects
   * @throws Error if there's a compilation error
   */
  async getSemanticTokens(uri: string, source: string): Promise<string> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.get_semantic_tokens(uri, source);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error getting semantic tokens:', error);
      throw new Error(`Failed to get semantic tokens: ${error}`);
    }
  }

  /**
   * Get code actions for a diagnostic at the given position
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @param diagnosticJson - JSON string of Diagnostic object
   * @returns JSON string containing array of CodeAction objects
   * @throws Error if there's a compilation error
   */
  async getCodeActions(
    uri: string,
    source: string,
    diagnosticJson: string
  ): Promise<string> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.get_code_actions(uri, source, diagnosticJson);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error getting code actions:', error);
      throw new Error(`Failed to get code actions: ${error}`);
    }
  }

  /**
   * Get document symbols for outline/navigator view
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @returns JSON string containing array of DocumentSymbol objects
   * @throws Error if there's a compilation error
   */
  async getDocumentSymbols(uri: string, source: string): Promise<string> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.get_document_symbols(uri, source);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error getting document symbols:', error);
      throw new Error(`Failed to get document symbols: ${error}`);
    }
  }

  /**
   * Prepare a rename operation (check if identifier can be renamed)
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @param line - 0-indexed line number
   * @param character - 0-indexed character position
   * @returns The identifier name if can be renamed, or null if cannot
   * @throws Error if there's a compilation error
   */
  async prepareRename(
    uri: string,
    source: string,
    line: number,
    character: number
  ): Promise<string | null> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.prepare_rename(uri, source, line, character);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error preparing rename:', error);
      throw new Error(`Failed to prepare rename: ${error}`);
    }
  }

  /**
   * Rename a symbol across all its occurrences
   *
   * @param uri - File URI (e.g., "file:///test.v")
   * @param source - The source code
   * @param line - 0-indexed line number
   * @param character - 0-indexed character position
   * @param newName - The new name for the symbol
   * @returns JSON string containing WorkspaceEdit with all replacements, or null if cannot rename
   * @throws Error if there's a compilation error
   */
  async rename(
    uri: string,
    source: string,
    line: number,
    character: number,
    newName: string
  ): Promise<string | null> {
    this.ensureInitialized();

    try {
      const result = await this.lsp.rename(uri, source, line, character, newName);
      return result;
    } catch (error) {
      console.error('[FiveLspClient] Error renaming symbol:', error);
      throw new Error(`Failed to rename symbol: ${error}`);
    }
  }

  /**
   * Set a document in the LSP workspace
   *
   * This notifies the LSP of a file's content, enabling multi-file analysis.
   * Call this when a file is opened, created, or modified.
   *
   * @param uri - File URI (e.g., "file:///workspace/src/main.v")
   * @param source - The source code content
   * @throws Error if there's a compilation error
   *
   * @example
   * ```typescript
   * await client.setDocument('file:///workspace/src/main.v', sourceCode);
   * ```
   */
  async setDocument(uri: string, source: string): Promise<void> {
    this.ensureInitialized();

    try {
      this.trackedDocuments.set(uri, source);
      console.log(`[FiveLspClient] setDocument: ${uri} (${source.length} chars)`);
    } catch (error) {
      console.error('[FiveLspClient] Error setting document:', error);
      throw new Error(`Failed to set document: ${error}`);
    }
  }

  /**
   * Remove a document from the LSP workspace
   *
   * Call this when a file is deleted or closed.
   *
   * @param uri - File URI (e.g., "file:///workspace/src/main.v")
   * @throws Error if there's an error
   *
   * @example
   * ```typescript
   * await client.removeDocument('file:///workspace/src/main.v');
   * ```
   */
  async removeDocument(uri: string): Promise<void> {
    this.ensureInitialized();

    try {
      this.trackedDocuments.delete(uri);
      console.log(`[FiveLspClient] removeDocument: ${uri}`);
    } catch (error) {
      console.error('[FiveLspClient] Error removing document:', error);
      throw new Error(`Failed to remove document: ${error}`);
    }
  }

  /**
   * Get workspace symbols matching a query
   *
   * Searches for symbols across all files in the workspace.
   *
   * @param query - Search query (case-insensitive substring match)
   * @returns Array of SymbolInformation objects
   * @throws Error if there's a compilation error
   *
   * @example
   * ```typescript
   * const symbols = await client.getWorkspaceSymbols('transfer');
   * symbols.forEach(sym => {
   *   console.log(`${sym.name} at ${sym.location.uri}`);
   * });
   * ```
   */
  async getWorkspaceSymbols(query: string): Promise<LspSymbolInformation[]> {
    this.ensureInitialized();

    try {
      console.log(`[FiveLspClient] getWorkspaceSymbols: "${query}"`);
      if (!query) {
        return [];
      }

      const deduped = new Map<string, LspSymbolInformation>();
      for (const [uri, source] of this.trackedDocuments.entries()) {
        try {
          const result = await this.lsp.get_workspace_symbols(uri, source, query);
          const symbols = JSON.parse(result) as LspSymbolInformation[];
          if (!Array.isArray(symbols)) {
            continue;
          }
          for (const symbol of symbols) {
            const range = symbol.location?.range?.start;
            const key = [
              symbol.name,
              symbol.kind ?? '',
              symbol.location?.uri ?? '',
              range?.line ?? '',
              range?.character ?? '',
            ].join(':');
            if (!deduped.has(key)) {
              deduped.set(key, symbol);
            }
          }
        } catch (error) {
          console.warn(`[FiveLspClient] Skipping workspace symbol scan for ${uri}:`, error);
        }
      }

      return Array.from(deduped.values()).sort((left, right) => {
        const nameCompare = left.name.localeCompare(right.name);
        if (nameCompare !== 0) {
          return nameCompare;
        }

        const leftUri = left.location?.uri ?? '';
        const rightUri = right.location?.uri ?? '';
        const uriCompare = leftUri.localeCompare(rightUri);
        if (uriCompare !== 0) {
          return uriCompare;
        }

        const leftLine = left.location?.range?.start?.line ?? 0;
        const rightLine = right.location?.range?.start?.line ?? 0;
        if (leftLine !== rightLine) {
          return leftLine - rightLine;
        }

        const leftCharacter = left.location?.range?.start?.character ?? 0;
        const rightCharacter = right.location?.range?.start?.character ?? 0;
        return leftCharacter - rightCharacter;
      });
    } catch (error) {
      console.error('[FiveLspClient] Error getting workspace symbols:', error);
      throw new Error(`Failed to get workspace symbols: ${error}`);
    }
  }

  /**
   * Get diagnostics for all files in the workspace
   *
   * Returns a map of file URIs to their diagnostics.
   *
   * @returns Map of URI to diagnostics array
   * @throws Error if there's a compilation error
   *
   * @example
   * ```typescript
   * const allDiagnostics = await client.getWorkspaceDiagnostics();
   * allDiagnostics.forEach((diags, uri) => {
   *   console.log(`${uri}: ${diags.length} issues`);
   * });
   * ```
   */
  async getWorkspaceDiagnostics(): Promise<Map<string, LspDiagnostic[]>> {
    this.ensureInitialized();

    try {
      console.log('[FiveLspClient] getWorkspaceDiagnostics');
      const diagnosticsByUri = new Map<string, LspDiagnostic[]>();
      for (const [uri, source] of this.trackedDocuments.entries()) {
        try {
          diagnosticsByUri.set(uri, this.getDiagnostics(uri, source));
        } catch (error) {
          console.warn(`[FiveLspClient] Skipping workspace diagnostics for ${uri}:`, error);
        }
      }
      return diagnosticsByUri;
    } catch (error) {
      console.error('[FiveLspClient] Error getting workspace diagnostics:', error);
      throw new Error(`Failed to get workspace diagnostics: ${error}`);
    }
  }

  /**
   * Clear all internal caches
   *
   * This forces recompilation on the next analysis call.
   * Useful when files have been significantly modified or memory cleanup is needed.
   *
   * @example
   * ```typescript
   * client.clearCaches();
   * ```
   */
  clearCaches(): void {
    this.ensureInitialized();
    this.trackedDocuments.clear();
    this.lsp.clear_caches();
  }

  /**
   * Check if the client is initialized
   *
   * @returns True if the client has been initialized
   */
  isInitialized(): boolean {
    return this.initialized;
  }
}

// Export a singleton instance for easy use
export const lspClient = new FiveLspClient();
