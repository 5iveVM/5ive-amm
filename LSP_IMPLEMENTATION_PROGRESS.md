# Five DSL LSP Implementation Progress

## Overview

This document tracks the implementation of the Five DSL Language Server Protocol (LSP) across phases, as defined in the comprehensive architecture plan.

## Current Status: Phase 1 Foundation Complete (MVP Skeleton)

### ✅ Completed

**Core Infrastructure (100%)**
- Created `five-lsp` crate as new workspace member
- Implemented modular architecture with clear separation of concerns
- Set up features and build configurations

**Module Structure**
- ✅ `src/lib.rs` - Main library entry point with conditional compilation
- ✅ `src/bridge.rs` - Bridge to five-dsl-compiler with AST caching
- ✅ `src/document.rs` - Document state management for open files
- ✅ `src/workspace.rs` - Workspace root and multi-file project support
- ✅ `src/error.rs` - LSP-compatible error types
- ✅ `src/server.rs` - Tower-LSP server implementation (stub)
- ✅ `src/features/mod.rs` - Feature module organization
- ✅ `src/features/diagnostics.rs` - Diagnostics feature (Phase 1)
- ✅ `src/native.rs` - Placeholder for native binary support
- ✅ `src/wasm.rs` - Placeholder for WASM support

**Compilation Status**
- ✅ Library compiles without native binary errors
- ✅ No critical compiler errors
- ✅ Minor warnings only (unused imports, dead code)
- ✅ Dependencies properly configured

### 🔄 In Progress / TODO

**Phase 1: MVP Diagnostics**
- 🔄 Tokenization errors → LSP diagnostics (partially implemented)
- 🔄 Parse errors → LSP diagnostics (partially implemented)
- ⏳ Type checking errors → LSP diagnostics (TODO)
- ⏳ Wire up diagnostics to editor in real-time (TODO)

**Phase 2: Navigation Features (blocked on Phase 1)**
- ⏳ Hover provider (type information)
- ⏳ Completion provider (keywords, identifiers)
- ⏳ Go-to-definition provider
- ⏳ Find references

**Phase 3: Advanced Features (blocked on Phase 2)**
- ⏳ Semantic tokens (syntax highlighting)
- ⏳ Code actions (quick fixes)
- ⏳ Rename refactoring
- ⏳ Document symbols (outline view)

**Phase 4: Polish & Optimization (blocked on Phase 3)**
- ⏳ Signature help
- ⏳ Workspace symbols
- ⏳ Inlay hints

**Platform Support**
- ⏳ VSCode extension with native binary (tower-lsp transport needs work)
- ⏳ Monaco integration (TypeScript WASM client)
- ⏳ WASM build and bindings

## Architecture

### Crate: `five-lsp`

```
five-lsp/
├── Cargo.toml          # Dependencies: tower-lsp, lsp-types, five-dsl-compiler
├── src/
│   ├── lib.rs          # Entry point, conditional compilation
│   ├── bridge.rs       # Bridge to compiler (tokenize, parse, cache)
│   ├── document.rs     # Document state management
│   ├── workspace.rs    # Workspace/project management
│   ├── error.rs        # LSP error types
│   ├── server.rs       # Tower-LSP server (feature-gated)
│   ├── native.rs       # Native binary support (TODO)
│   ├── wasm.rs         # WASM support (TODO)
│   ├── main.rs         # Binary entry point (disabled)
│   └── features/
│       ├── mod.rs      # Feature module organization
│       └── diagnostics.rs  # Phase 1: Error reporting
└── tests/              # (TODO) LSP scenario tests
```

### Key Design Decisions

1. **Two Build Targets**
   - Library: Always builds, no tokio/tower-lsp required
   - Binary: Optional "native" feature, disabled for now (tower-lsp stdio transport needs investigation)

2. **Compiler Bridge Pattern**
   - Reuses existing five-dsl-compiler infrastructure (tokenizer, parser, type checker)
   - Implements AST caching by source hash to avoid unnecessary recompilation
   - Direct error conversion to LSP diagnostics (no separate error collection layer yet)

3. **Document Management**
   - In-memory document store tracks open files and versions
   - Supports both full and incremental changes
   - Workspace tracks file relationships for multi-module support

4. **Feature Organization**
   - Each LSP feature in separate module (diagnostics, completion, hover, etc.)
   - Clear phase boundaries for incremental delivery
   - Disabled features return sensible defaults (None/empty collections)

## Technical Details

### CompilerBridge Implementation

The bridge reuses compiler phases:
```rust
pub fn get_diagnostics(&mut self, uri: &Url, source: &str)
    -> Result<Vec<lsp_types::Diagnostic>, LspError>
```

**Current Flow:**
1. Tokenize source → collect tokens or return tokenization error
2. Parse tokens → collect AST or return parse error
3. Cache AST by source hash
4. Convert all errors to LSP Diagnostic format
5. Return diagnostics to editor

**Optimization:** AST caching reduces recompilation when user hasn't changed source

### Type Checking Status

⚠️ **Type checking not yet integrated** - The compiler's `DslTypeChecker` requires proper error collection which the current bridge doesn't handle. This needs to be addressed in Phase 1.

**Options:**
1. Implement error collector in five-dsl-compiler integration
2. Create wrapper to collect type errors separately
3. Use compiler's existing error system more directly

## Integration Points

### Five Frontend (Monaco)
- TypeScript LSP client (not yet implemented)
- WASM bindings to call Rust LSP from browser
- Integrates with Monaco editor via provider registration

### VSCode Extension (Not Started)
- Extension manifest (`package.json`)
- TextMate grammar (`five.tmLanguage.json`)
- LSP client that spawns native binary
- Requires native binary to be fixed (tower-lsp transport)

## Build & Test

### Build the Library
```bash
# Build library only (always works)
cargo build --lib -p five-lsp

# Build with native feature enabled
cargo build --lib -p five-lsp --features native

# Run tests (TODO: create tests)
cargo test -p five-lsp
```

### Current Build Status
```
✅ Library: Builds successfully
✅ With features: Builds successfully
⚠️  Native binary: Disabled (tower-lsp transport TODO)
❌ Tests: None yet
```

## Next Steps (Priority Order)

### Immediate (Complete Phase 1)
1. **Implement Type Checking Diagnostics**
   - Integrate type checker error collection
   - Convert type errors to LSP diagnostics
   - Test with real Five DSL files

2. **Create Test Suite**
   - Unit tests for bridge (tokenize, parse, cache)
   - Integration tests for end-to-end diagnostics
   - Test files with various error types

3. **Wire Up to Editor** (Frontend)
   - Create TypeScript LSP client wrapper
   - Implement Monaco provider registrations
   - Test diagnostics in live editor

### Short Term (Phase 2 Prep)
4. **Fix Native Binary Transport**
   - Investigate tower-lsp 0.20 stdio pattern
   - Or switch to different LSP framework if simpler
   - Create VSCode extension skeleton

5. **Symbol Table Integration**
   - Extract symbol info from TypeCheckerContext
   - Prepare for hover/completion in Phase 2

### Medium Term (Phase 2)
6. **Hover Provider**
   - Use symbol table to get type information
   - Format type info for editor tooltip

7. **Completion Provider**
   - Keyword completion (function, let, pub, etc.)
   - Identifier completion from scope
   - Account constraint completion (@mut, @signer)

8. **Go-to-Definition**
   - AST walking to find symbol definitions
   - Support across multiple files

## Known Issues & Workarounds

### Issue 1: Native Binary (tower-lsp transport)
**Status:** Disabled for now
**Reason:** tower-lsp 0.20 Server::new() signature requires understanding correct socket/transport pattern
**Workaround:** Library works fine, focus on WASM first
**Resolution:** Will fix in Phase 2 when doing VSCode extension

### Issue 2: Type Checking Error Collection
**Status:** Type checking phase not yet integrated
**Reason:** Need proper error collection mechanism for diagnostics
**Workaround:** Currently only reporting tokenize/parse errors
**Resolution:** Implement error wrapper or use compiler's error system more directly

## File Structure Summary

| File | Status | Purpose |
|------|--------|---------|
| five-lsp/Cargo.toml | ✅ Complete | Dependencies and features |
| src/lib.rs | ✅ Complete | Module organization |
| src/bridge.rs | 🔄 Partial | Compiler integration (needs type checking) |
| src/document.rs | ✅ Complete | Document state management |
| src/workspace.rs | ✅ Complete | Project management |
| src/error.rs | ✅ Complete | Error types |
| src/server.rs | 🔄 Stub | Tower-LSP server (feature-gated) |
| src/features/diagnostics.rs | 🔄 Partial | Diagnostics feature |
| src/native.rs | 📋 Placeholder | Native binary utilities |
| src/wasm.rs | 📋 Placeholder | WASM bindings |
| src/main.rs | 🔒 Disabled | Binary entry point |

## Dependencies

```toml
tower-lsp = "0.20"           # LSP server framework
lsp-types = "0.94"           # LSP protocol types
five-dsl-compiler = { ... }  # Compiler bridge
tokio = { features: [...] }  # Async runtime (optional)
serde/serde_json             # Serialization
thiserror                    # Error handling
futures                      # Async utilities
tracing                      # Structured logging
```

## Success Criteria for Phase 1

- [ ] Tokenization errors appear as red squiggles in Monaco
- [ ] Parse errors appear as red squiggles in Monaco
- [ ] Type errors appear as red squiggles in Monaco
- [ ] Diagnostics update in real-time as user types
- [ ] Same errors as compiler CLI for consistency
- [ ] LSP response time < 500ms for typical files
- [ ] Unit tests for bridge (tokenize, parse, cache)
- [ ] Integration tests for end-to-end diagnostics

## Notes for Future Developers

- **Error Formatting:** Existing `LspFormatter` in five-dsl-compiler can be reused once error collection is working
- **Type Context:** `TypeCheckerContext` has `symbol_table` field - useful for Phase 2 features
- **Module Resolution:** `ModuleScope` supports multi-file type checking
- **Performance:** AST caching by source hash prevents unnecessary recompilation
- **Testing:** Use simple Five DSL files in `five-templates/` as test cases

---

**Last Updated:** 2026-01-25
**Phase:** 1 (MVP Foundation)
**Status:** Infrastructure Complete, Feature Development In Progress
