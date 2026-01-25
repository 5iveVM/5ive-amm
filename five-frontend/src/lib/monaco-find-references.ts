/**
 * Monaco Editor Integration for Five LSP Find References Provider
 *
 * Registers a references provider with Monaco Editor that enables finding all usages
 * of a symbol via "Find All References" command or keyboard shortcut.
 *
 * Usage:
 * ```typescript
 * import * as monaco from 'monaco-editor';
 * import { registerReferencesProvider } from './lib/monaco-find-references';
 *
 * // After Monaco is initialized and LSP is set up
 * registerReferencesProvider(monaco, lspClient);
 * ```
 */

import * as monaco from 'monaco-editor';
import { FiveLspClient } from './lsp-client';

/**
 * Register the find references provider with Monaco Editor
 *
 * Enables finding all references to a symbol when the user:
 * - Uses "Find All References" command
 * - Uses "Shift+F12" keyboard shortcut (Ctrl+Shift+F12 on some systems)
 * - Uses "Find References" command palette
 */
export function registerReferencesProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): void {
    monacoInstance.languages.registerReferenceProvider('five', {
        provideReferences: async (model, position, context, token) => {
            try {
                // Get all references from LSP
                const referencesJson = await lspClient.findReferences(
                    model.uri.toString(),
                    model.getValue(),
                    position.lineNumber - 1,  // Convert to 0-indexed
                    position.column - 1       // Convert to 0-indexed
                );

                if (!referencesJson) {
                    return [];
                }

                // Parse and return references
                const references = JSON.parse(referencesJson);
                return references as monaco.languages.Location[];
            } catch (error) {
                console.error('[Monaco References] Error finding references:', error);
                return [];
            }
        }
    });

    console.log('[Monaco References] Find references provider registered for Five DSL');
}
