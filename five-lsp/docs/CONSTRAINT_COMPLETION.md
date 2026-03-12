# Five LSP Constraint Annotation Completion

## Overview

The Five LSP now provides intelligent autocomplete for constraint annotations used in Five DSL function parameters. When you type `@` in an account parameter declaration, the LSP automatically suggests all available constraint annotations with detailed documentation.

## Supported Constraints

### `@signer`
**Description:** Requires the account to be a signer of the transaction
**Usage:** Required for accounts that must authorize the transaction
**Example:**
```v
pub transfer(from: account @signer, to: account, amount: u64) {
    // 'from' account must sign the transaction
}
```

### `@mut`
**Description:** Marks the account as mutable/writable
**Usage:** Required for accounts that will be modified during execution
**Example:**
```v
pub transfer(from: account @mut, to: account @mut, amount: u64) {
    // Both 'from' and 'to' accounts will be modified
}
```

### `@init`
**Description:** Initializes a new account
**Usage:** Creates and initializes a new account
**Syntax:** `@init(payer=<account>, space=<bytes>)`
**Example:**
```v
pub create_account(
    new_account: account @init(payer=payer, space=100),
    payer: account @signer @mut
) {
    // Creates a new account with 100 bytes of space
}
```

### `@close`
**Description:** Closes a script-owned account and refunds rent to a target account
**Usage:** Use for account cleanup on successful instruction completion
**Syntax:** `@close(to=<recipient_account>)`
**Example:**
```v
pub close_vault(vault: account @mut @close(to=recipient), recipient: account @mut) {
    // vault lamports are transferred to recipient on success
}
```

### `@writable`
**Description:** Alias for `@mut` - marks account as writable
**Usage:** Alternate syntax for the `@mut` constraint
**Example:**
```v
pub update(data_account: account @writable) {
    // Same as @mut
}
```

## How It Works

### Trigger Character
The completion provider is triggered automatically when you type `@` in a function parameter.

### Context Detection
The LSP detects that you're in a constraint annotation context by:
1. Finding the `@` character before the cursor
2. Checking if you're in a function parameter declaration
3. Extracting any partial constraint name you've already typed

### Filtering
As you type after `@`, the suggestions are filtered to match your input:
- Typing `@` shows all 5 constraints
- Typing `@s` shows only `@signer`
- Typing `@m` shows only `@mut`
- Typing `@i` shows only `@init`
- Typing `@w` shows only `@writable`

### Documentation
Each suggestion includes:
- **Label:** The full constraint syntax (e.g., `@signer`)
- **Detail:** Short description of what the constraint does
- **Documentation:** Comprehensive explanation of when and how to use it

## Implementation Details

### LSP Backend (Rust)
**File:** `five-lsp/src/features/completion.rs`

The completion provider has two main functions:

1. **`try_get_constraint_suggestions()`** - Detects if cursor is after `@` symbol
   - Looks backwards from cursor to find `@`
   - Extracts partial constraint name
   - Returns constraint suggestions if in correct context

2. **`get_constraint_suggestions()`** - Generates constraint completion items
   - Filters constraints by prefix match
   - Includes detailed documentation for each constraint
   - Returns properly formatted LSP `CompletionItem` objects

### Monaco Frontend (TypeScript)
**File:** `five-frontend/src/lib/monaco-completion.ts`

The Monaco integration:
- Registers `@` as a trigger character
- Calls LSP `getCompletions()` when user types
- Converts LSP response to Monaco `CompletionItem` format
- Displays suggestions with documentation in the IDE

## Testing

### Unit Tests
**File:** `five-lsp/src/features/completion.rs` (tests module)

Five comprehensive tests validate the constraint completion:

1. **`test_constraint_suggestions_after_at_symbol`**
   - Verifies all 5 constraints are suggested after typing `@`

2. **`test_constraint_suggestions_partial_match`**
   - Verifies filtering works (e.g., `@si` matches only `@signer`)

3. **`test_constraint_suggestions_has_documentation`**
   - Ensures all constraints have detail and documentation fields

4. **`test_no_constraint_suggestions_without_at`**
   - Verifies constraints are NOT suggested without `@` prefix

5. **`test_constraint_suggestions_multiple_params`**
   - Verifies suggestions work in multi-parameter functions

All tests pass with `cargo test -p five-lsp --lib completion::tests`.

### Demo File
**File:** `five-lsp/tests/constraint_completion_demo.v`

A demonstration Five DSL file showing all constraint annotation patterns.

## Usage Example

1. Open a Five DSL file in the Monaco editor
2. Start typing a function with account parameters:
   ```v
   pub transfer(from: account @
   ```
3. After typing `@`, the autocomplete menu appears with all 5 constraints
4. Select a constraint or continue typing to filter
5. Press Enter to insert the selected constraint

## Performance

- **Constraint detection:** O(n) where n = characters from cursor to `@` (typically < 20 chars)
- **Filtering:** O(m) where m = number of constraints (fixed at 5)
- **No AST parsing required** - uses lightweight text-based heuristic
- **Instant response** - constraint suggestions appear immediately

## Future Enhancements

Potential improvements for Phase 2+:

1. **Context-aware suggestions**
   - Only suggest `@signer` for authority accounts
   - Only suggest `@init` for new account parameters

2. **Constraint validation**
   - Check for conflicting constraints (e.g., `@init` and `@mut`)
   - Validate `@init` syntax (payer and space parameters)

3. **Snippet expansion**
   - Insert full `@init(payer=$1, space=$2)` template
   - Tab through parameter placeholders

4. **Semantic context**
   - Use AST to determine if we're truly in a parameter declaration
   - Validate constraint usage against function semantics

## Related Features

This constraint completion integrates with:
- **Diagnostics:** Errors shown for invalid constraint usage
- **Hover:** Tooltip documentation for constraints
- **Code Actions:** Quick fixes for constraint-related errors (future)

## API Contract

The constraint completion is exposed through the standard LSP `textDocument/completion` request:

**Request:**
```json
{
  "textDocument": { "uri": "file:///workspace/example.v" },
  "position": { "line": 5, "character": 28 }
}
```

**Response:**
```json
{
  "isIncomplete": false,
  "items": [
    {
      "label": "@signer",
      "kind": 14,
      "detail": "Requires the account to be a signer of the transaction",
      "documentation": "Required for accounts that must authorize the transaction",
      "insertText": "@signer"
    }
    // ... other constraints
  ]
}
```

## Credits

Implemented as part of the Five LSP Phase 1 semantic analysis integration, following the architecture documented in `LSP_CONTRACT.md`.
