/**
 * Monaco Editor Integration for Five DSL LSP
 *
 * Sets up Monaco Editor providers (diagnostics, hover, completion, etc.)
 * to use the Five LSP for real-time language features.
 *
 * Usage:
 * ```typescript
 * import * as monaco from 'monaco-editor';
 * import { setupFiveLsp } from './lib/monaco-lsp';
 *
 * // After Monaco is initialized
 * await setupFiveLsp(monaco);
 * ```
 */

import type * as monaco from 'monaco-editor';
import { FiveLspClient, Diagnostic } from './lsp-client';
import { registerHoverProvider } from './monaco-hover';
import { registerCompletionProvider } from './monaco-completion';
import { registerDefinitionProvider } from './monaco-goto-definition';
import { registerReferencesProvider } from './monaco-find-references';
import { registerCodeActionsProvider } from './monaco-code-actions';
import { registerDocumentSymbolsProvider } from './monaco-document-symbols';
import { registerRenameProvider } from './monaco-rename';

/**
 * Initialize and register all Five LSP providers with Monaco Editor
 *
 * This sets up:
 * - Diagnostic provider (error squiggles)
 * - Future: Hover provider, completion, etc.
 *
 * @param monacoInstance - The monaco module instance
 * @returns Promise that resolves when setup is complete
 *
 * @example
 * ```typescript
 * import * as monaco from 'monaco-editor';
 * import { setupFiveLsp } from './lib/monaco-lsp';
 *
 * async function initializeEditor() {
 *   const editor = monaco.editor.create(document.getElementById('editor'), {
 *     language: 'five',
 *     theme: 'vs-dark'
 *   });
 *
 *   await setupFiveLsp(monaco);
 * }
 * ```
 */
export async function setupFiveLsp(
  monacoInstance: typeof monaco
): Promise<void> {
  console.log('[Monaco LSP Setup] Initializing...');

  // Create LSP client
  const lspClient = new FiveLspClient();

  try {
    // Initialize WASM module
    await lspClient.initialize();
    console.log('[Monaco LSP Setup] LSP client initialized');
  } catch (error) {
    console.error('[Monaco LSP Setup] Failed to initialize LSP:', error);
    return;
  }

  // Register Phase 2 providers
  registerDiagnosticsProvider(monacoInstance, lspClient);
  registerHoverProvider(monacoInstance, lspClient);
  registerCompletionProvider(monacoInstance, lspClient);
  registerDefinitionProvider(monacoInstance, lspClient);
  registerReferencesProvider(monacoInstance, lspClient);

  // Register Phase 3 providers
  // Note: Semantic tokens disabled - Monaco version may not support this API
  // registerSemanticTokensProvider(monacoInstance, lspClient);
  registerCodeActionsProvider(monacoInstance, lspClient);
  registerDocumentSymbolsProvider(monacoInstance, lspClient);
  registerRenameProvider(monacoInstance, lspClient);

  console.log('[Monaco LSP Setup] Complete - All providers registered (Phase 2: 5, Phase 3: 3)');
}

/**
 * Register the diagnostics provider
 *
 * Provides real-time error squiggles as the user edits Five DSL files.
 */
function registerDiagnosticsProvider(
  monaco: typeof import('monaco-editor'),
  lspClient: FiveLspClient
): void {
  // Set up a debounced diagnostics update
  const updateDiagnostics = debounce(
    (model: monaco.editor.ITextModel) => {
      try {
        const uri = model.uri.toString();
        const source = model.getValue();

        // Get diagnostics from LSP
        const diagnostics = lspClient.getDiagnostics(uri, source);

        // Convert to Monaco Diagnostic format
        const monacoDiagnostics = diagnostics.map((diag) =>
          convertToMonacoDiagnostic(diag)
        );

        // Set diagnostics for this model
        monaco.editor.setModelMarkers(model, 'five-lsp', monacoDiagnostics);

        console.log(
          `[Monaco LSP] Updated ${monacoDiagnostics.length} diagnostics for ${uri}`
        );
      } catch (error) {
        console.error('[Monaco LSP] Error updating diagnostics:', error);
        // Don't clear diagnostics on error - keep the last good state
      }
    },
    500 // Debounce after 500ms of inactivity
  );

  // Listen for content changes on all Five files
  monaco.editor.onDidCreateModel((model) => {
    if (model.getLanguageId() !== 'five') {
      return;
    }

    console.log(`[Monaco LSP] Monitoring ${model.uri.toString()}`);

    // Initial diagnostic pass
    updateDiagnostics(model);

    // Re-analyze on every change
    const listener = model.onDidChangeContent(() => {
      updateDiagnostics(model);
    });

    // Clean up listener when model is disposed
    model.onWillDispose(() => {
      listener.dispose();
      monaco.editor.setModelMarkers(model, 'five-lsp', []);
    });
  });

  // Analyze any already-open models
  for (const model of monaco.editor.getModels()) {
    if (model.getLanguageId() === 'five') {
      updateDiagnostics(model);
    }
  }
}

/**
 * Convert LSP Diagnostic to Monaco Diagnostic format
 */
function convertToMonacoDiagnostic(
  lspDiag: Diagnostic
): monaco.editor.IMarker {
  return {
    startLineNumber: lspDiag.range.start.line + 1,
    startColumn: lspDiag.range.start.character + 1,
    endLineNumber: lspDiag.range.end.line + 1,
    endColumn: lspDiag.range.end.character + 1,
    message: lspDiag.message,
    severity: convertSeverity(lspDiag.severity),
    code: lspDiag.code,
    source: lspDiag.source || 'five-lsp',
  };
}

/**
 * Convert LSP severity level to Monaco severity
 *
 * LSP: 1 = error, 2 = warning, 3 = information, 4 = hint
 * Monaco: 8 = error, 4 = warning, 2 = info, 1 = hint
 */
function convertSeverity(
  lspSeverity: number | undefined
): import('monaco-editor').MarkerSeverity {
  const monaco = require('monaco-editor');
  switch (lspSeverity) {
    case 1:
      return monaco.MarkerSeverity.Error;
    case 2:
      return monaco.MarkerSeverity.Warning;
    case 3:
      return monaco.MarkerSeverity.Info;
    case 4:
      return monaco.MarkerSeverity.Hint;
    default:
      return monaco.MarkerSeverity.Error;
  }
}

/**
 * Debounce utility for performance
 *
 * Prevents excessive diagnostics updates while the user is typing quickly.
 */
function debounce<T extends (...args: any[]) => void>(
  fn: T,
  delay: number
): T {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return ((...args: any[]) => {
    if (timeoutId !== null) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => {
      fn(...args);
      timeoutId = null;
    }, delay);
  }) as T;
}

/**
 * Export the LSP client for use in other modules
 *
 * This allows other parts of the application to access the LSP client
 * for additional features beyond Monaco integration.
 *
 * Usage:
 * ```typescript
 * import { lspClient } from './lib/monaco-lsp';
 *
 * const diagnostics = lspClient.getDiagnostics(uri, source);
 * ```
 */
export { FiveLspClient } from './lsp-client';
