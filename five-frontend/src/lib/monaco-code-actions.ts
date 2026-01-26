/**
 * Monaco Editor Integration for Five LSP Code Actions Provider
 *
 * Registers a code actions provider with Monaco Editor for quick fixes.
 */

import * as monaco from 'monaco-editor';
import { FiveLspClient } from './lsp-client';

/**
 * Register the code actions provider with Monaco Editor
 *
 * Provides quick fixes and code actions for Five DSL files.
 */
export function registerCodeActionsProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): void {
    monacoInstance.languages.registerCodeActionProvider('five', {
        provideCodeActions: async (model, range, context, token) => {
            try {
                const actions: monaco.languages.CodeAction[] = [];

                // Get code actions for each diagnostic in the range
                for (const diagnostic of context.markers) {
                    // Check if diagnostic is in our range
                    if (
                        diagnostic.startLineNumber > range.endLineNumber ||
                        diagnostic.endLineNumber < range.startLineNumber
                    ) {
                        continue;
                    }

                    // Create diagnostic object matching LSP format
                    const lspDiagnostic = {
                        range: {
                            start: {
                                line: diagnostic.startLineNumber - 1,
                                character: diagnostic.startColumn - 1,
                            },
                            end: {
                                line: diagnostic.endLineNumber - 1,
                                character: diagnostic.endColumn - 1,
                            },
                        },
                        severity: diagnostic.severity,
                        message: diagnostic.message,
                        source: diagnostic.source,
                        code: diagnostic.code,
                    };

                    const codeActionsJson = await lspClient.getCodeActions(
                        model.uri.toString(),
                        model.getValue(),
                        JSON.stringify(lspDiagnostic)
                    );

                    if (!codeActionsJson) {
                        continue;
                    }

                    const codeActionList = JSON.parse(codeActionsJson);
                    if (!Array.isArray(codeActionList)) {
                        continue;
                    }

                    // Convert LSP code actions to Monaco format
                    for (const action of codeActionList) {
                        actions.push({
                            title: action.title || 'Code Action',
                            kind: action.kind || monaco.languages.CodeActionKind.QuickFix,
                            diagnostics: [diagnostic],
                            isPreferred: action.isPreferred || false,
                            edit: action.edit ? convertWorkspaceEdit(action.edit) : undefined,
                        } as monaco.languages.CodeAction);
                    }
                }

                return { actions, dispose: () => {} };
            } catch (error) {
                console.error('[Monaco Code Actions] Error getting code actions:', error);
                return { actions: [], dispose: () => {} };
            }
        },
    });

    console.log('[Monaco Code Actions] Code actions provider registered for Five DSL');
}

/**
 * Convert LSP WorkspaceEdit to Monaco format
 */
function convertWorkspaceEdit(edit: any): monaco.languages.WorkspaceEdit {
    const changes: { [key: string]: monaco.languages.TextEdit[] } = {};

    if (edit.changes) {
        for (const [uri, textEdits] of Object.entries(edit.changes)) {
            changes[uri] = (textEdits as any[]).map((te) => ({
                range: new monaco.Range(
                    te.range.start.line + 1,
                    te.range.start.character + 1,
                    te.range.end.line + 1,
                    te.range.end.character + 1
                ),
                text: te.new_text || te.newText || '',
            }));
        }
    }

    return { edits: Object.entries(changes).map(([resource, edits]) => ({ resource: monaco.Uri.parse(resource), edits })) };
}
