/**
 * Monaco Editor Integration for Five LSP Go-to-Definition Provider
 *
 * Registers a definition provider with Monaco Editor that enables jumping to
 * function/type definitions via Ctrl+Click or keyboard shortcut.
 *
 * Usage:
 * ```typescript
 * import * as monaco from 'monaco-editor';
 * import { registerDefinitionProvider } from './lib/monaco-goto-definition';
 *
 * // After Monaco is initialized and LSP is set up
 * registerDefinitionProvider(monaco, lspClient);
 * ```
 */

import * as monaco from 'monaco-editor';
import { FiveLspClient } from './lsp-client';

/**
 * Register the go-to-definition provider with Monaco Editor
 *
 * Enables jumping to function and type definitions when the user:
 * - Ctrl+Clicks on a symbol
 * - Uses "Go to Definition" keyboard shortcut
 * - Uses "Go to Definition" command palette
 */
export function registerDefinitionProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): void {
    monacoInstance.languages.registerDefinitionProvider('five', {
        provideDefinition: async (model, position, token) => {
            try {
                // Get definition location from LSP
                const locationJson = await lspClient.getDefinition(
                    model.uri.toString(),
                    model.getValue(),
                    position.lineNumber - 1,  // Convert to 0-indexed
                    position.column - 1       // Convert to 0-indexed
                );

                if (!locationJson) {
                    return null;
                }

                // Parse and return location
                const location = JSON.parse(locationJson);
                return location as monaco.languages.Location;
            } catch (error) {
                console.error('[Monaco Definition] Error getting definition:', error);
                return null;
            }
        }
    });

    console.log('[Monaco Definition] Go-to-definition provider registered for Five DSL');
}
