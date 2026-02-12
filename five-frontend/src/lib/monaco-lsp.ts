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

// Global disposables tracker
let globalDisposables: monaco.IDisposable[] = [];
let globalLspClient: FiveLspClient | null = null;

// URI cache for model consistency
const modelUriCache = new Map<string, string>();

/**
 * Initialize and register all Five LSP providers with Monaco Editor
 *
 * This sets up:
 * - Diagnostic provider (error squiggles)
 * - Hover, completion, definition, references providers
 * - Code actions, document symbols, rename providers
 *
 * @param monacoInstance - The monaco module instance
 * @returns Promise that resolves when setup is complete
 *
 * @example
 * ```typescript
 * import * as monaco from 'monaco-editor';
 * import { setupFiveLsp, teardownFiveLsp } from './lib/monaco-lsp';
 *
 * async function initializeEditor() {
 *   const editor = monaco.editor.create(document.getElementById('editor'), {
 *     language: 'five',
 *     theme: 'vs-dark'
 *   });
 *
 *   await setupFiveLsp(monaco);
 * }
 *
 * // Cleanup on unmount
 * function cleanup() {
 *   teardownFiveLsp();
 * }
 * ```
 */
export async function setupFiveLsp(
  monacoInstance: typeof monaco
): Promise<void> {
  console.log('[Monaco LSP Setup] Initializing...');

  // Clean up any existing setup
  if (globalDisposables.length > 0) {
    console.warn('[Monaco LSP Setup] Cleaning up existing providers before re-initialization');
    teardownFiveLsp();
  }

  // Create LSP client
  const lspClient = new FiveLspClient();
  globalLspClient = lspClient;

  try {
    // Initialize WASM module
    await lspClient.initialize();
    console.log('[Monaco LSP Setup] LSP client initialized');
  } catch (error) {
    console.error('[Monaco LSP Setup] Failed to initialize LSP:', error);
    globalLspClient = null;
    return;
  }

  // Register Phase 2 providers (collect disposables)
  const diagnosticsDisposable = registerDiagnosticsProvider(monacoInstance, lspClient);
  if (diagnosticsDisposable) globalDisposables.push(diagnosticsDisposable);

  const hoverDisposable = registerHoverProvider(monacoInstance, lspClient);
  if (hoverDisposable) globalDisposables.push(hoverDisposable);

  const completionDisposable = registerCompletionProvider(monacoInstance, lspClient);
  if (completionDisposable) globalDisposables.push(completionDisposable);

  const definitionDisposable = registerDefinitionProvider(monacoInstance, lspClient);
  if (definitionDisposable) globalDisposables.push(definitionDisposable);

  const referencesDisposable = registerReferencesProvider(monacoInstance, lspClient);
  if (referencesDisposable) globalDisposables.push(referencesDisposable);

  // Register Phase 3 providers
  // Note: Semantic tokens disabled - Monaco version may not support this API
  // const semanticDisposable = registerSemanticTokensProvider(monacoInstance, lspClient);
  // if (semanticDisposable) globalDisposables.push(semanticDisposable);

  const codeActionsDisposable = registerCodeActionsProvider(monacoInstance, lspClient);
  if (codeActionsDisposable) globalDisposables.push(codeActionsDisposable);

  const docSymbolsDisposable = registerDocumentSymbolsProvider(monacoInstance, lspClient);
  if (docSymbolsDisposable) globalDisposables.push(docSymbolsDisposable);

  const renameDisposable = registerRenameProvider(monacoInstance, lspClient);
  if (renameDisposable) globalDisposables.push(renameDisposable);

  console.log(
    `[Monaco LSP Setup] Complete - ${globalDisposables.length} providers registered (Phase 2: 5, Phase 3: 3)`
  );
}

/**
 * Teardown Five LSP integration and clean up resources
 *
 * Disposes all registered providers and clears caches.
 * Call this when unmounting the editor or reinitializing LSP.
 *
 * @example
 * ```typescript
 * import { teardownFiveLsp } from './lib/monaco-lsp';
 *
 * // On component unmount
 * useEffect(() => {
 *   return () => {
 *     teardownFiveLsp();
 *   };
 * }, []);
 * ```
 */
export function teardownFiveLsp(): void {
  console.log(`[Monaco LSP Setup] Tearing down ${globalDisposables.length} providers...`);

  // Dispose all registered providers
  for (const disposable of globalDisposables) {
    try {
      disposable.dispose();
    } catch (error) {
      console.error('[Monaco LSP Setup] Error disposing provider:', error);
    }
  }
  globalDisposables = [];

  // Clear LSP caches
  if (globalLspClient && globalLspClient.isInitialized()) {
    try {
      globalLspClient.clearCaches();
    } catch (error) {
      console.error('[Monaco LSP Setup] Error clearing LSP caches:', error);
    }
  }
  globalLspClient = null;

  console.log('[Monaco LSP Setup] Teardown complete');
}

/**
 * Register the diagnostics provider
 *
 * Provides real-time error squiggles as the user edits Five DSL files.
 *
 * @returns Disposable to clean up the provider
 */
function registerDiagnosticsProvider(
  monaco: typeof import('monaco-editor'),
  lspClient: FiveLspClient
): monaco.IDisposable | null {
  const modelListeners: monaco.IDisposable[] = [];

  // Set up a debounced diagnostics update
  const updateDiagnostics = debounce(
    (model: monaco.editor.ITextModel) => {
      try {
        const uri = model.uri.toString();
        const source = model.getValue();

        // Get diagnostics from LSP
        const diagnostics = lspClient.getDiagnostics(uri, source);

        // Convert to Monaco Diagnostic format
        const monacoDiagnostics = diagnostics.map((diag) => {
          const converted = convertToMonacoDiagnostic(diag);
          console.log(
            `[Monaco LSP] Diagnostic: "${diag.message}" at LSP line ${diag.range.start.line} -> Monaco line ${converted.startLineNumber}`
          );
          return converted;
        });

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
  const onCreateModelDisposable = monaco.editor.onDidCreateModel((model) => {
    if (model.getLanguageId() !== 'five') {
      return;
    }

    console.log(`[Monaco LSP] Monitoring ${model.uri.toString()}`);

    // Initial diagnostic pass
    updateDiagnostics(model);

    // Re-analyze on every change (debounced)
    const listener = model.onDidChangeContent(() => {
      updateDiagnostics(model);
    });

    modelListeners.push(listener);

    // Clean up listener when model is disposed
    model.onWillDispose(() => {
      listener.dispose();
      monaco.editor.setModelMarkers(model, 'five-lsp', []);
      const index = modelListeners.indexOf(listener);
      if (index > -1) {
        modelListeners.splice(index, 1);
      }
    });
  });

  // Analyze any already-open models
  for (const model of monaco.editor.getModels()) {
    if (model.getLanguageId() === 'five') {
      updateDiagnostics(model);
    }
  }

  // Return composite disposable
  return {
    dispose: () => {
      onCreateModelDisposable.dispose();
      for (const listener of modelListeners) {
        listener.dispose();
      }
      modelListeners.length = 0;
    },
  };
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
): number {
  // Use numeric constants instead of enum to avoid require() at runtime
  switch (lspSeverity) {
    case 1:
      return 8; // MarkerSeverity.Error
    case 2:
      return 4; // MarkerSeverity.Warning
    case 3:
      return 2; // MarkerSeverity.Info
    case 4:
      return 1; // MarkerSeverity.Hint
    default:
      return 8; // Default to Error
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
