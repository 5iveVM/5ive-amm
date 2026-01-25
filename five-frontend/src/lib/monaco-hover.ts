/**
 * Monaco Editor Integration for Five LSP Hover Provider
 *
 * Registers a hover provider with Monaco Editor that displays type information
 * when hovering over symbols.
 *
 * Usage:
 * ```typescript
 * import * as monaco from 'monaco-editor';
 * import { registerHoverProvider } from './lib/monaco-hover';
 *
 * // After Monaco is initialized and LSP is set up
 * registerHoverProvider(monaco, lspClient);
 * ```
 */

import * as monaco from 'monaco-editor';
import { FiveLspClient } from './lsp-client';

/**
 * Register the hover provider with Monaco Editor
 *
 * Displays type information when hovering over symbols in Five DSL files.
 */
export function registerHoverProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): void {
    monacoInstance.languages.registerHoverProvider('five', {
        provideHover: async (model, position, token) => {
            try {
                // Get hover information from LSP
                const hoverJson = await lspClient.getHover(
                    model.uri.toString(),
                    model.getValue(),
                    position.lineNumber - 1,  // Convert to 0-indexed
                    position.column - 1       // Convert to 0-indexed
                );

                if (!hoverJson) {
                    return null;
                }

                // Parse and return hover information
                const hover = JSON.parse(hoverJson);
                return hover as monaco.languages.Hover;
            } catch (error) {
                console.error('[Monaco Hover] Error getting hover:', error);
                return null;
            }
        }
    });

    console.log('[Monaco Hover] Hover provider registered for Five DSL');
}
