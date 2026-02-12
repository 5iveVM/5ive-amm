# Five LSP WASM API Contract

This document defines the stable API contract for Five LSP WASM bindings. All TypeScript clients must adhere to these signatures and JSON formats.

## Version

- **LSP Version:** 1.0.0
- **Last Updated:** 2026-02-12

## Core Principles

1. **Position Indexing:** All positions use 0-based indexing (line and character)
2. **URI Format:** File URIs must use the `file://` scheme (e.g., `file:///workspace/test.v`)
3. **JSON Serialization:** All complex return types are JSON strings that must be parsed by the client
4. **Error Handling:** Methods return `Result<T, JsValue>` where errors are formatted strings
5. **Immutability:** Most methods take `&self` or `&mut self` but don't persist state across calls

---

## WASM Method Signatures

### Constructor

```rust
pub fn new() -> FiveLspWasm
```

Creates a new LSP instance. Initializes the compiler bridge and panic hooks for better error reporting in browsers.

**Usage:**
```typescript
const lsp = FiveLspWasm.new();
```

---

### get_diagnostics

```rust
pub fn get_diagnostics(&mut self, uri: &str, source: &str) -> Result<String, JsValue>
```

Analyzes source code and returns compilation diagnostics (syntax and semantic errors).

**Parameters:**
- `uri`: File URI (e.g., `"file:///workspace/test.v"`)
- `source`: Complete source code content

**Returns:**
JSON string containing array of `Diagnostic` objects:

```typescript
interface Diagnostic {
  range: Range;
  severity: DiagnosticSeverity; // 1=Error, 2=Warning, 3=Info, 4=Hint
  code?: string | number;
  message: string;
  source?: string;
  relatedInformation?: DiagnosticRelatedInformation[];
}

interface Range {
  start: Position;
  end: Position;
}

interface Position {
  line: number;      // 0-indexed
  character: number; // 0-indexed
}
```

**Example:**
```typescript
const diagnosticsJson = lsp.get_diagnostics('file:///test.v', 'init { let x = 5; }');
const diagnostics: Diagnostic[] = JSON.parse(diagnosticsJson);
```

---

### get_hover

```rust
pub fn get_hover(
    &mut self,
    uri: &str,
    source: &str,
    line: u32,
    character: u32,
) -> Result<Option<String>, JsValue>
```

Provides hover information (type, documentation) for a symbol at the given position.

**Parameters:**
- `uri`: File URI
- `source`: Complete source code
- `line`: 0-indexed line number
- `character`: 0-indexed character offset

**Returns:**
- `Some(json_string)` if hover information is available
- `None` if no symbol at position

JSON format:
```typescript
interface Hover {
  contents: MarkupContent | MarkedString | MarkedString[];
  range?: Range;
}

interface MarkupContent {
  kind: 'plaintext' | 'markdown';
  value: string;
}
```

---

### get_completions

```rust
pub fn get_completions(
    &self,
    uri: &str,
    source: &str,
    line: u32,
    character: u32,
) -> Result<String, JsValue>
```

Returns code completion suggestions at the given position.

**Parameters:**
- `uri`: File URI
- `source`: Complete source code
- `line`: 0-indexed line number
- `character`: 0-indexed character offset

**Returns:**
JSON string containing `CompletionList`:

```typescript
interface CompletionList {
  isIncomplete: boolean;
  items: CompletionItem[];
}

interface CompletionItem {
  label: string;
  kind: CompletionItemKind; // 1=Text, 3=Function, 6=Variable, etc.
  detail?: string;
  documentation?: string | MarkupContent;
  insertText?: string;
  insertTextFormat?: InsertTextFormat; // 1=PlainText, 2=Snippet
  filterText?: string;
  sortText?: string;
}
```

---

### get_definition

```rust
pub fn get_definition(
    &mut self,
    uri: &str,
    source: &str,
    line: u32,
    character: u32,
) -> Result<Option<String>, JsValue>
```

Finds the definition location of a symbol.

**Parameters:**
- `uri`: File URI
- `source`: Complete source code
- `line`: 0-indexed line number
- `character`: 0-indexed character offset

**Returns:**
- `Some(json_string)` if definition found
- `None` if no definition

JSON format:
```typescript
interface Location {
  uri: string;
  range: Range;
}
```

---

### find_references

```rust
pub fn find_references(
    &mut self,
    uri: &str,
    source: &str,
    line: u32,
    character: u32,
) -> Result<String, JsValue>
```

Finds all references to a symbol.

**Parameters:**
- `uri`: File URI
- `source`: Complete source code
- `line`: 0-indexed line number
- `character`: 0-indexed character offset

**Returns:**
JSON string containing array of `Location` objects (see `get_definition` for format).

---

### get_semantic_tokens

```rust
pub fn get_semantic_tokens(
    &self,
    uri: &str,
    source: &str,
) -> Result<String, JsValue>
```

Returns semantic tokens for AST-based syntax highlighting.

**Parameters:**
- `uri`: File URI
- `source`: Complete source code

**Returns:**
JSON string containing array of semantic tokens:

```typescript
interface SerializableSemanticToken {
  line: number;              // 0-indexed
  start_character: number;   // 0-indexed
  length: number;
  token_type: number;        // Index into token types legend
  token_modifiers: number;   // Bitfield of modifiers
}
```

**Token Types Legend:**
```typescript
const TOKEN_TYPES = [
  'keyword', 'function', 'variable', 'parameter', 'property',
  'type', 'interface', 'namespace', 'operator', 'comment',
  'string', 'number'
];

const TOKEN_MODIFIERS = [
  'declaration', 'readonly', 'static', 'mutable', 'public'
];
```

---

### get_document_symbols

```rust
pub fn get_document_symbols(
    &self,
    uri: &str,
    source: &str,
) -> Result<String, JsValue>
```

Returns document outline symbols (functions, variables, accounts).

**Parameters:**
- `uri`: File URI
- `source`: Complete source code

**Returns:**
JSON string containing array of `DocumentSymbol`:

```typescript
interface DocumentSymbol {
  name: string;
  detail?: string;
  kind: SymbolKind; // 5=Class, 6=Method, 12=Function, 13=Variable, etc.
  range: Range;
  selectionRange: Range;
  children?: DocumentSymbol[];
}
```

---

### get_code_actions

```rust
pub fn get_code_actions(
    &self,
    uri: &str,
    source: &str,
    diagnostic_json: &str,
) -> Result<String, JsValue>
```

Provides quick fix code actions for a diagnostic.

**Parameters:**
- `uri`: File URI
- `source`: Complete source code
- `diagnostic_json`: JSON-serialized `Diagnostic` object

**Returns:**
JSON string containing array of `CodeAction`:

```typescript
interface CodeAction {
  title: string;
  kind?: string; // 'quickfix', 'refactor', etc.
  diagnostics?: Diagnostic[];
  edit?: WorkspaceEdit;
}

interface WorkspaceEdit {
  changes?: { [uri: string]: TextEdit[] };
  documentChanges?: (TextDocumentEdit | CreateFile | RenameFile | DeleteFile)[];
}

interface TextEdit {
  range: Range;
  newText: string;
}
```

---

### prepare_rename

```rust
pub fn prepare_rename(
    &self,
    uri: &str,
    source: &str,
    line: u32,
    character: u32,
) -> Result<Option<String>, JsValue>
```

Validates that a symbol at the given position can be renamed.

**Parameters:**
- `uri`: File URI (for multi-file context)
- `source`: Complete source code
- `line`: 0-indexed line number
- `character`: 0-indexed character offset

**Returns:**
- `Some(identifier_name)` if symbol can be renamed
- `None` if position is invalid or symbol cannot be renamed

**Note:** This method was updated to include `uri` parameter for Phase 0 bug fix.

---

### rename

```rust
pub fn rename(
    &mut self,
    uri: &str,
    source: &str,
    line: u32,
    character: u32,
    new_name: &str,
) -> Result<Option<String>, JsValue>
```

Renames a symbol and returns workspace edits.

**Parameters:**
- `uri`: File URI
- `source`: Complete source code
- `line`: 0-indexed line number
- `character`: 0-indexed character offset
- `new_name`: New identifier name

**Returns:**
- `Some(json_string)` containing `WorkspaceEdit`
- `None` if rename is not possible

---

### clear_caches

```rust
pub fn clear_caches(&mut self)
```

Clears all internal caches (AST, symbol tables, diagnostics).

**Parameters:** None

**Returns:** None

**Usage:**
```typescript
lsp.clear_caches();
```

---

## Phase 1+ Methods (To Be Added)

The following methods will be added in future phases:

### set_document

```rust
pub fn set_document(&mut self, uri: &str, source: &str) -> Result<(), JsValue>
```

Registers a document in the workspace for cross-file analysis.

### remove_document

```rust
pub fn remove_document(&mut self, uri: &str) -> Result<(), JsValue>
```

Removes a document from the workspace.

### get_workspace_symbols

```rust
pub fn get_workspace_symbols(&self, query: &str, workspace_json: &str) -> Result<String, JsValue>
```

Searches for symbols across all workspace files.

### get_signature_help

```rust
pub fn get_signature_help(
    &self,
    uri: &str,
    source: &str,
    line: u32,
    character: u32,
) -> Result<Option<String>, JsValue>
```

Provides function signature help while typing.

### get_inlay_hints

```rust
pub fn get_inlay_hints(
    &self,
    uri: &str,
    source: &str,
    start_line: u32,
    end_line: u32,
) -> Result<String, JsValue>
```

Returns inline type and parameter hints.

### format_document

```rust
pub fn format_document(&self, uri: &str, source: &str) -> Result<String, JsValue>
```

Formats entire document and returns formatted source.

### format_range

```rust
pub fn format_range(
    &self,
    uri: &str,
    source: &str,
    start_line: u32,
    start_char: u32,
    end_line: u32,
    end_char: u32,
) -> Result<String, JsValue>
```

Formats a specific range in the document.

---

## Error Handling

All methods return `Result<T, JsValue>`. Errors are formatted as strings and can be caught in TypeScript:

```typescript
try {
  const result = lsp.get_diagnostics(uri, source);
  const diagnostics = JSON.parse(result);
} catch (error) {
  console.error('LSP error:', error);
}
```

Common error patterns:
- `"Invalid URI: ..."` - URI parsing failed
- `"Compilation error: ..."` - Source code compilation failed
- `"Serialization error: ..."` - JSON serialization failed

---

## Capability Advertisement (server.rs)

The LSP server advertises capabilities during initialization. Only advertise features that are fully implemented:

**Current (Phase 0):**
- ✅ `textDocumentSync` (FULL)
- ✅ `hoverProvider`
- ✅ `completionProvider`
- ✅ `definitionProvider`
- ✅ `referencesProvider`
- ✅ `semanticTokensProvider` (full mode only)
- ✅ `codeActionProvider`
- ✅ `renameProvider`
- ✅ `documentSymbolProvider`
- ✅ `signatureHelpProvider`
- ✅ `workspaceSymbolProvider`
- ✅ `inlayHintProvider`
- ✅ `documentFormattingProvider`

**Not Implemented:**
- ❌ `documentRangeFormattingProvider` (Phase 2)
- ❌ `declarationProvider`
- ❌ `typeDefinitionProvider`
- ❌ `implementationProvider`
- ❌ `callHierarchyProvider`
- ❌ `typeHierarchyProvider`
- ❌ `linkedEditingRangeProvider`
- ❌ `foldingRangeProvider`
- ❌ `selectionRangeProvider`
- ❌ `documentHighlightProvider`

---

## Testing Contract Compliance

All WASM methods should have tests verifying:
1. JSON round-trip (serialize → deserialize)
2. Error path handling (invalid URI, malformed source)
3. Edge cases (empty source, EOF position, null pointers)
4. Schema validation (TypeScript interfaces match Rust types)

Example test pattern:

```rust
#[test]
fn test_get_hover_contract() {
    let lsp = FiveLspWasm::new();
    let uri = "file:///test.v";
    let source = "let x: u64 = 5;";

    let result = lsp.get_hover(uri, source, 0, 4).unwrap();
    assert!(result.is_some());

    // Verify JSON deserialization
    let json = result.unwrap();
    let hover: lsp_types::Hover = serde_json::from_str(&json).unwrap();
    assert!(hover.contents.is_some());
}
```

---

## Change Log

### v1.0.0 (2026-02-12) - Phase 0
- Initial API contract documentation
- **BREAKING CHANGE:** Added `uri` parameter to `prepare_rename()` for multi-file support
- Documented all current WASM methods
- Defined JSON schemas for all return types
- Established capability advertisement rules
