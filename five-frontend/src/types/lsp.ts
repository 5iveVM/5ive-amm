/**
 * Five LSP Type Definitions
 *
 * TypeScript type definitions that match the JSON schemas defined in
 * five-lsp/docs/LSP_CONTRACT.md
 *
 * These types ensure type safety when working with LSP responses.
 */

// ============================================================================
// Core LSP Types
// ============================================================================

/**
 * Position in a text document (0-indexed)
 */
export interface LspPosition {
  line: number;      // 0-indexed line number
  character: number; // 0-indexed character offset
}

/**
 * Range in a text document
 */
export interface LspRange {
  start: LspPosition;
  end: LspPosition;
}

/**
 * Text edit for modifying a document
 */
export interface LspTextEdit {
  range: LspRange;
  newText: string;
}

/**
 * Location in a document (URI + range)
 */
export interface LspLocation {
  uri: string;
  range: LspRange;
}

// ============================================================================
// Diagnostics
// ============================================================================

/**
 * Diagnostic severity levels
 */
export enum LspDiagnosticSeverity {
  Error = 1,
  Warning = 2,
  Information = 3,
  Hint = 4,
}

/**
 * Related information for a diagnostic
 */
export interface LspDiagnosticRelatedInformation {
  location: LspLocation;
  message: string;
}

/**
 * Diagnostic (error, warning, or info message)
 */
export interface LspDiagnostic {
  range: LspRange;
  severity?: LspDiagnosticSeverity;
  code?: string | number;
  source?: string;
  message: string;
  relatedInformation?: LspDiagnosticRelatedInformation[];
  tags?: number[]; // DiagnosticTag enum values
}

// ============================================================================
// Hover
// ============================================================================

/**
 * Markup content kind
 */
export type MarkupKind = 'plaintext' | 'markdown';

/**
 * Markup content for rich text display
 */
export interface LspMarkupContent {
  kind: MarkupKind;
  value: string;
}

/**
 * Marked string (legacy format)
 */
export interface LspMarkedString {
  language?: string;
  value: string;
}

/**
 * Hover contents (union type)
 */
export type LspHoverContents =
  | LspMarkupContent
  | LspMarkedString
  | LspMarkedString[]
  | string;

/**
 * Hover information
 */
export interface LspHover {
  contents: LspHoverContents;
  range?: LspRange;
}

// ============================================================================
// Completion
// ============================================================================

/**
 * Completion item kind
 */
export enum LspCompletionItemKind {
  Text = 1,
  Method = 2,
  Function = 3,
  Constructor = 4,
  Field = 5,
  Variable = 6,
  Class = 7,
  Interface = 8,
  Module = 9,
  Property = 10,
  Unit = 11,
  Value = 12,
  Enum = 13,
  Keyword = 14,
  Snippet = 15,
  Color = 16,
  File = 17,
  Reference = 18,
  Folder = 19,
  EnumMember = 20,
  Constant = 21,
  Struct = 22,
  Event = 23,
  Operator = 24,
  TypeParameter = 25,
}

/**
 * Insert text format
 */
export enum LspInsertTextFormat {
  PlainText = 1,
  Snippet = 2,
}

/**
 * Completion item
 */
export interface LspCompletionItem {
  label: string;
  kind?: LspCompletionItemKind;
  detail?: string;
  documentation?: string | LspMarkupContent;
  deprecated?: boolean;
  preselect?: boolean;
  sortText?: string;
  filterText?: string;
  insertText?: string;
  insertTextFormat?: LspInsertTextFormat;
  textEdit?: LspTextEdit;
  additionalTextEdits?: LspTextEdit[];
  commitCharacters?: string[];
  command?: LspCommand;
  data?: any;
}

/**
 * Completion list
 */
export interface LspCompletionList {
  isIncomplete: boolean;
  items: LspCompletionItem[];
}

// ============================================================================
// Code Actions
// ============================================================================

/**
 * Command
 */
export interface LspCommand {
  title: string;
  command: string;
  arguments?: any[];
}

/**
 * Text document edit
 */
export interface LspTextDocumentEdit {
  textDocument: {
    uri: string;
    version?: number;
  };
  edits: LspTextEdit[];
}

/**
 * Create file operation
 */
export interface LspCreateFile {
  kind: 'create';
  uri: string;
  options?: {
    overwrite?: boolean;
    ignoreIfExists?: boolean;
  };
}

/**
 * Rename file operation
 */
export interface LspRenameFile {
  kind: 'rename';
  oldUri: string;
  newUri: string;
  options?: {
    overwrite?: boolean;
    ignoreIfExists?: boolean;
  };
}

/**
 * Delete file operation
 */
export interface LspDeleteFile {
  kind: 'delete';
  uri: string;
  options?: {
    recursive?: boolean;
    ignoreIfNotExists?: boolean;
  };
}

/**
 * Document change (union type)
 */
export type LspDocumentChange =
  | LspTextDocumentEdit
  | LspCreateFile
  | LspRenameFile
  | LspDeleteFile;

/**
 * Workspace edit
 */
export interface LspWorkspaceEdit {
  changes?: { [uri: string]: LspTextEdit[] };
  documentChanges?: LspDocumentChange[];
}

/**
 * Code action kind constants
 */
export const LspCodeActionKind = {
  Empty: '',
  QuickFix: 'quickfix',
  Refactor: 'refactor',
  RefactorExtract: 'refactor.extract',
  RefactorInline: 'refactor.inline',
  RefactorRewrite: 'refactor.rewrite',
  Source: 'source',
  SourceOrganizeImports: 'source.organizeImports',
} as const;

export type LspCodeActionKindType = typeof LspCodeActionKind[keyof typeof LspCodeActionKind];

/**
 * Code action
 */
export interface LspCodeAction {
  title: string;
  kind?: LspCodeActionKindType;
  diagnostics?: LspDiagnostic[];
  isPreferred?: boolean;
  disabled?: {
    reason: string;
  };
  edit?: LspWorkspaceEdit;
  command?: LspCommand;
  data?: any;
}

// ============================================================================
// Symbols
// ============================================================================

/**
 * Symbol kind
 */
export enum LspSymbolKind {
  File = 1,
  Module = 2,
  Namespace = 3,
  Package = 4,
  Class = 5,
  Method = 6,
  Property = 7,
  Field = 8,
  Constructor = 9,
  Enum = 10,
  Interface = 11,
  Function = 12,
  Variable = 13,
  Constant = 14,
  String = 15,
  Number = 16,
  Boolean = 17,
  Array = 18,
  Object = 19,
  Key = 20,
  Null = 21,
  EnumMember = 22,
  Struct = 23,
  Event = 24,
  Operator = 25,
  TypeParameter = 26,
}

/**
 * Document symbol (hierarchical)
 */
export interface LspDocumentSymbol {
  name: string;
  detail?: string;
  kind: LspSymbolKind;
  tags?: number[]; // SymbolTag enum values
  deprecated?: boolean;
  range: LspRange;
  selectionRange: LspRange;
  children?: LspDocumentSymbol[];
}

/**
 * Symbol information (flat)
 */
export interface LspSymbolInformation {
  name: string;
  kind: LspSymbolKind;
  tags?: number[];
  deprecated?: boolean;
  location: LspLocation;
  containerName?: string;
}

// ============================================================================
// Signature Help
// ============================================================================

/**
 * Parameter information in a signature
 */
export interface LspParameterInformation {
  label: string | [number, number]; // Label or offset range
  documentation?: string | LspMarkupContent;
}

/**
 * Signature information
 */
export interface LspSignatureInformation {
  label: string;
  documentation?: string | LspMarkupContent;
  parameters?: LspParameterInformation[];
  activeParameter?: number;
}

/**
 * Signature help
 */
export interface LspSignatureHelp {
  signatures: LspSignatureInformation[];
  activeSignature?: number;
  activeParameter?: number;
}

// ============================================================================
// Inlay Hints
// ============================================================================

/**
 * Inlay hint kind
 */
export enum LspInlayHintKind {
  Type = 1,
  Parameter = 2,
}

/**
 * Inlay hint label part
 */
export interface LspInlayHintLabelPart {
  value: string;
  tooltip?: string | LspMarkupContent;
  location?: LspLocation;
  command?: LspCommand;
}

/**
 * Inlay hint
 */
export interface LspInlayHint {
  position: LspPosition;
  label: string | LspInlayHintLabelPart[];
  kind?: LspInlayHintKind;
  textEdits?: LspTextEdit[];
  tooltip?: string | LspMarkupContent;
  paddingLeft?: boolean;
  paddingRight?: boolean;
  data?: any;
}

// ============================================================================
// Semantic Tokens
// ============================================================================

/**
 * Semantic token (from Five LSP)
 */
export interface LspSemanticToken {
  line: number;
  start_character: number;
  length: number;
  token_type: number;        // Index into token types legend
  token_modifiers: number;   // Bitfield of modifiers
}

/**
 * Semantic token types (must match LSP server legend)
 */
export const LspSemanticTokenTypes = [
  'keyword',
  'function',
  'variable',
  'parameter',
  'property',
  'type',
  'interface',
  'namespace',
  'operator',
  'comment',
  'string',
  'number',
] as const;

/**
 * Semantic token modifiers (must match LSP server legend)
 */
export const LspSemanticTokenModifiers = [
  'declaration',
  'readonly',
  'static',
  'mutable',
  'public',
] as const;

// ============================================================================
// Rename
// ============================================================================

/**
 * Rename location (for prepareRename)
 */
export interface LspRenameLocation {
  range: LspRange;
  placeholder: string;
}

/**
 * Prepare rename result (union type)
 */
export type LspPrepareRenameResult = LspRange | LspRenameLocation | { defaultBehavior: boolean };

// ============================================================================
// Type Guards
// ============================================================================

/**
 * Type guard for MarkupContent
 */
export function isLspMarkupContent(value: any): value is LspMarkupContent {
  return (
    typeof value === 'object' &&
    value !== null &&
    'kind' in value &&
    'value' in value &&
    (value.kind === 'plaintext' || value.kind === 'markdown')
  );
}

/**
 * Type guard for MarkedString
 */
export function isLspMarkedString(value: any): value is LspMarkedString {
  return (
    typeof value === 'object' &&
    value !== null &&
    'value' in value &&
    typeof value.value === 'string'
  );
}

/**
 * Type guard for DiagnosticRelatedInformation
 */
export function isLspDiagnosticRelatedInformation(
  value: any
): value is LspDiagnosticRelatedInformation {
  return (
    typeof value === 'object' &&
    value !== null &&
    'location' in value &&
    'message' in value
  );
}
