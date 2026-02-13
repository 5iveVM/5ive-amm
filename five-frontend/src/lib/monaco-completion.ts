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
 *
 * @returns Disposable to clean up the provider
 */
export function registerCompletionProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): monaco.IDisposable {
    return monacoInstance.languages.registerCompletionItemProvider('five', {
        triggerCharacters: ['@'],  // Trigger completion when typing '@' for constraint annotations

        provideCompletionItems: async (model, position, context, token) => {
            try {
                // Get completions from LSP
                const completionsJson = await lspClient.getCompletions(
                    model.uri.toString(),
                    model.getValue(),
                    position.lineNumber - 1,  // Convert to 0-indexed
                    position.column - 1       // Convert to 0-indexed
                );

                // Parse and convert to Monaco format
                const response = JSON.parse(completionsJson);
                const items = response.items || [];

                // Normalize completion items to ensure all required fields are present
                const suggestions = items.map((item: any) => ({
                    label: item.label || item.name || 'unknown',
                    insertText: item.insertText || item.label || item.name || 'unknown',
                    kind: item.kind || 5,  // Default to Class (5)
                    detail: item.detail || item.description || undefined,
                    documentation: item.documentation || undefined,
                    sortText: item.sortText || item.label || item.name,
                    filterText: item.filterText || item.label || item.name,
                    isPreferred: item.isPreferred || false,
                    preselect: item.preselect || false,
                } as monaco.languages.CompletionItem));

                return {
                    suggestions,
                    isIncomplete: response.isIncomplete || false
                } as monaco.languages.CompletionList;
            } catch (error) {
                console.error('[Monaco Completion] Error getting completions:', error);
                // Return empty list on error (don't crash)
                return { suggestions: [], isIncomplete: false };
            }
        }
    });

    console.log('[Monaco Completion] Completion provider registered for Five DSL');
}
