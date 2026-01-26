/**
 * Monaco Editor Integration for Five LSP Rename Provider
 *
 * Registers a rename provider with Monaco Editor for safe symbol renaming.
 */

import type * as monaco from 'monaco-editor';
import { FiveLspClient } from './lsp-client';

/**
 * Register the rename provider with Monaco Editor
 *
 * Enables safe renaming of symbols across all their occurrences in the file.
 */
export function registerRenameProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): void {
    monacoInstance.languages.registerRenameProvider('five', {
        provideRenameEdits: async (model, position, newName, token) => {
            try {
                const renameJson = await lspClient.rename(
                    model.uri.toString(),
                    model.getValue(),
                    position.lineNumber - 1,  // Convert to 0-indexed
                    position.column - 1,      // Convert to 0-indexed
                    newName
                );

                if (!renameJson) {
                    return null;
                }

                const workspaceEdit = JSON.parse(renameJson);
                return convertWorkspaceEdit(monacoInstance, workspaceEdit);
            } catch (error) {
                console.error('[Monaco Rename] Error renaming symbol:', error);
                return null;
            }
        },
        resolveRenameLocation: async (model, position, token) => {
            try {
                const symbolName = await lspClient.prepareRename(
                    model.uri.toString(),
                    model.getValue(),
                    position.lineNumber - 1,  // Convert to 0-indexed
                    position.column - 1       // Convert to 0-indexed
                );

                if (!symbolName) {
                    return null;
                }

                // Return the range of the symbol at the position
                const word = model.getWordAtPosition(position);
                if (!word) {
                    return null;
                }

                return {
                    range: new monaco.Range(
                        position.lineNumber,
                        word.startColumn,
                        position.lineNumber,
                        word.endColumn
                    ),
                    text: symbolName,
                };
            } catch (error) {
                console.error('[Monaco Rename] Error preparing rename:', error);
                return null;
            }
        },
    });

    console.log('[Monaco Rename] Rename provider registered for Five DSL');
}

/**
 * Convert LSP WorkspaceEdit to Monaco format
 */
function convertWorkspaceEdit(
    monacoInstance: typeof monaco,
    edit: any
): monaco.languages.WorkspaceEdit | null {
    if (!edit.changes) {
        return null;
    }

    const edits: monaco.languages.ResourceTextEdit[] = [];

    for (const [uri, textEdits] of Object.entries(edit.changes)) {
        const resourceUri = monacoInstance.Uri.parse(uri);
        const resourceEdits = (textEdits as any[]).map((te) => ({
            range: new monacoInstance.Range(
                te.range.start.line + 1,
                te.range.start.character + 1,
                te.range.end.line + 1,
                te.range.end.character + 1
            ),
            text: te.new_text || te.newText || '',
        }));

        edits.push({
            resource: resourceUri,
            edits: resourceEdits,
        });
    }

    return { edits };
}
