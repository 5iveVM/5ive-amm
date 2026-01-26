/**
 * Monaco Editor Integration for Five LSP Completion Provider
 *
 * Registers a completion provider with Monaco Editor that displays code suggestions
 * when the user triggers autocompletion.
 *
 * Usage:
 * ```typescript
 * import type * as monaco from 'monaco-editor';
 * import { registerCompletionProvider } from './lib/monaco-completion';
 *
 * // After Monaco is initialized and LSP is set up
 * registerCompletionProvider(monaco, lspClient);
 * ```
 */

import type * as monaco from 'monaco-editor';
import { FiveLspClient } from './lsp-client';

/**
 * Register the completion provider with Monaco Editor
 *
 * Displays code suggestions (keywords, variables, types) when the user
 * types or triggers autocompletion (Ctrl+Space).
 */
export function registerCompletionProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): void {
    monacoInstance.languages.registerCompletionItemProvider('five', {
        triggerCharacters: [],

        provideCompletionItems: async (model, position, context, token) => {
            try {
                // Get completions from LSP
                const completionsJson = await lspClient.getCompletions(
                    model.uri.toString(),
                    model.getValue(),
                    position.lineNumber - 1,  // Convert to 0-indexed
                    position.column - 1       // Convert to 0-indexed
                );

                // Parse and return completion list
                const completionList = JSON.parse(completionsJson);
                return completionList as monaco.languages.CompletionList;
            } catch (error) {
                console.error('[Monaco Completion] Error getting completions:', error);
                // Return empty list on error (don't crash)
                return { items: [], isIncomplete: false };
            }
        }
    });

    console.log('[Monaco Completion] Completion provider registered for Five DSL');
}
