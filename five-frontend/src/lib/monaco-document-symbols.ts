/**
 * Monaco Editor Integration for Five LSP Document Symbols Provider
 *
 * Registers a document symbols provider with Monaco Editor for outline/navigator view.
 */

import type * as monaco from 'monaco-editor';
import { FiveLspClient } from './lsp-client';

/**
 * Register the document symbols provider with Monaco Editor
 *
 * Provides document symbols (outline) for quick navigation in Five DSL files.
 */
export function registerDocumentSymbolsProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): void {
    monacoInstance.languages.registerDocumentSymbolProvider('five', {
        provideDocumentSymbols: async (model, token) => {
            try {
                const symbolsJson = await lspClient.getDocumentSymbols(
                    model.uri.toString(),
                    model.getValue()
                );

                if (!symbolsJson) {
                    return [];
                }

                const symbols = JSON.parse(symbolsJson);
                if (!Array.isArray(symbols)) {
                    return [];
                }

                // Convert LSP DocumentSymbol to Monaco format
                return symbols.map((symbol) => ({
                    name: symbol.name || 'Unknown',
                    kind: mapSymbolKind(symbol.kind || 0),
                    location: {
                        uri: model.uri,
                        range: new monaco.Range(
                            symbol.range.start.line + 1,
                            symbol.range.start.character + 1,
                            symbol.range.end.line + 1,
                            symbol.range.end.character + 1
                        ),
                    },
                    containerName: symbol.detail || undefined,
                    children: symbol.children
                        ? symbol.children.map((child) => ({
                            name: child.name || 'Unknown',
                            kind: mapSymbolKind(child.kind || 0),
                            location: {
                                uri: model.uri,
                                range: new monaco.Range(
                                    child.range.start.line + 1,
                                    child.range.start.character + 1,
                                    child.range.end.line + 1,
                                    child.range.end.character + 1
                                ),
                            },
                            containerName: child.detail || undefined,
                        }))
                        : undefined,
                })) as monaco.languages.DocumentSymbol[];
            } catch (error) {
                console.error('[Monaco Document Symbols] Error getting document symbols:', error);
                return [];
            }
        },
    });

    console.log('[Monaco Document Symbols] Document symbols provider registered for Five DSL');
}

/**
 * Map LSP SymbolKind to Monaco SymbolKind
 */
function mapSymbolKind(kind: number): monaco.languages.SymbolKind {
    // LSP SymbolKind values match Monaco in most cases
    // 1 = File, 2 = Module, 3 = Namespace, 4 = Package, 5 = Class, 6 = Struct,
    // 7 = Interface, 8 = Enum, 9 = EnumMember, 10 = Variable, 11 = Constant,
    // 12 = String, 13 = Number, 14 = Boolean, 15 = Array, 16 = Object,
    // 17 = Key, 18 = Null, 19 = EnumMember, 20 = Struct, 21 = Event,
    // 22 = Operator, 23 = TypeParameter

    // FUNCTION = 12, VARIABLE = 13, CONSTRUCTOR = 24
    if (kind === 12) return monaco.languages.SymbolKind.Function;
    if (kind === 13) return monaco.languages.SymbolKind.Variable;
    if (kind === 24) return monaco.languages.SymbolKind.Constructor;
    if (kind === 6) return monaco.languages.SymbolKind.Struct;

    // Default to Variable
    return monaco.languages.SymbolKind.Variable;
}
