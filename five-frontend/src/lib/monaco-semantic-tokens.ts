/**
 * Monaco Editor Integration for Five LSP Semantic Tokens Provider
 *
 * Registers a semantic tokens provider with Monaco Editor for AST-based syntax highlighting.
 */

import * as monaco from 'monaco-editor';
import { FiveLspClient } from './lsp-client';

/**
 * Semantic token types for Five DSL
 */
const TOKEN_TYPES = [
    'function',
    'variable',
    'type',
    'keyword',
    'modifier',
    'comment',
    'string',
    'number',
    'account',
    'operator',
];

/**
 * Semantic token modifiers for Five DSL
 */
const TOKEN_MODIFIERS = [
    'declaration',
    'definition',
    'readonly',
    'deprecated',
    'public',
    'mutable',
];

/**
 * Register the semantic tokens provider with Monaco Editor
 *
 * Provides AST-based syntax highlighting for Five DSL files.
 */
export function registerSemanticTokensProvider(
    monacoInstance: typeof monaco,
    lspClient: FiveLspClient
): void {
    const legend: monaco.languages.SemanticTokensLegend = {
        tokenTypes: TOKEN_TYPES,
        tokenModifiers: TOKEN_MODIFIERS,
    };

    monacoInstance.languages.registerSemanticTokensProvider('five', {
        getLegend: () => legend,
        provideSemanticTokens: async (model, lastResultId, token) => {
            try {
                const semanticTokensJson = await lspClient.getSemanticTokens(
                    model.uri.toString(),
                    model.getValue()
                );

                if (!semanticTokensJson) {
                    return { data: new Uint32Array() };
                }

                const tokens = JSON.parse(semanticTokensJson);
                if (!Array.isArray(tokens) || tokens.length === 0) {
                    return { data: new Uint32Array() };
                }

                // Convert tokens to Uint32Array format expected by Monaco
                // Format: [line, startCharacter, length, tokenType, tokenModifiers]
                const data: number[] = [];
                for (const token of tokens) {
                    data.push(token.line || 0);
                    data.push(token.startCharacter || 0);
                    data.push(token.length || 1);
                    data.push(token.tokenType || 0);
                    data.push(token.tokenModifiers || 0);
                }

                return { data: new Uint32Array(data) };
            } catch (error) {
                console.error('[Monaco Semantic Tokens] Error getting semantic tokens:', error);
                return { data: new Uint32Array() };
            }
        },
        releaseSemanticTokens: () => {
            // No cleanup needed
        },
    });

    console.log('[Monaco Semantic Tokens] Semantic tokens provider registered for Five DSL');
}
