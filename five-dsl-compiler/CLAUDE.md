# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the **Five DSL Compiler** — an off-chain Rust compiler for the Five DSL, which compiles `.five` DSL syntax into STKS bytecode for execution by the FIVE VM (Five VM MITO). The compiler is modular and supports multi-file compilation, type checking, bytecode optimization, and ABI generation.

## Quick Commands

### Build & Test
```bash
cargo build                                    # Compile with default features (abi-pack enabled)
cargo build --features security-audit         # Include security audit features
cargo test                                     # Run all unit and integration tests
cargo test --test <test_name>                  # Run specific integration test
cargo test -- --nocapture                     # Show DSL compiler output in tests
cargo test --lib type_checker::                # Run unit tests in specific module
```

### Run CLI Tools
```bash
cargo run --bin five -- compile <file.five>                    # Main compiler CLI
cargo run --bin five -- compile <file.five> --mode deployment  # Compile for production
cargo run --bin five -- compile-multi <main.five> <lib.five>   # Multi-file compilation
cargo run --bin five -- metrics <file.five> --metrics-output metrics.json  # Collect metrics
cargo run --bin debug_compile <file.five>                      # Debug compilation pipeline
cargo run --bin extract_abi <file.five>                        # Extract ABI from bytecode
```

### Code Quality
```bash
cargo fmt                                     # Format code
cargo clippy --all-targets --all-features    # Lint all targets with all features
cargo clippy --lib type_checker::             # Lint specific module
```

## Codebase Architecture

### Compiler Pipeline (Five Stages)
The compiler processes DSL source through a unified pipeline in `src/compiler/`:

1. **Tokenization** (`src/tokenizer.rs`) — Lexical analysis into tokens
2. **Parsing** (`src/parser/`) — Token stream into Abstract Syntax Tree (AST)
   - `mod.rs` — Main parser orchestration
   - `expressions.rs` — Expression parsing
   - `statements.rs` — Statement parsing
   - `blocks.rs` — Block/scope parsing
3. **Type Checking** (`src/type_checker/`) — Semantic analysis and type validation
   - `mod.rs` — Main type checker (facade to `types.rs::TypeCheckerContext`)
   - `module_scope.rs` — Cross-module symbol resolution
   - `expressions.rs` — Expression type inference
   - `statements.rs` — Statement type validation
   - `functions.rs` — Function signature and call validation
   - `inference.rs` — Type inference engine
4. **Bytecode Generation** (`src/bytecode_generator/`) — AST to bytecode (heavily modularized)
5. **Output** — Bytecode serialization, ABI generation, metrics collection

The pipeline is unified in `src/compiler/pipeline.rs::CompilationPipeline` to eliminate duplication across multiple compilation APIs. All public compiler methods delegate to this pipeline.

### Bytecode Generator (Modular Architecture)
The bytecode generator is split into focused modules under `src/bytecode_generator/`:

**Core Components:**
- `types.rs` — Data structures (FunctionInfo, FieldInfo, AccountRegistry, FIVEABI)
- `opcodes.rs` — Opcode emission and utilities
- `function_dispatch.rs` — O(1) function routing via dispatch tables
- `account_system.rs` — Account type definitions and validation
- `account_utils.rs` — Unified account type detection

**Generation & Optimization:**
- `ast_generator/` — Modular AST→bytecode conversion
  - `expressions.rs`, `statements.rs`, `functions.rs`, etc. (organized by AST node type)
  - Includes array literals, control flow, field access, initialization
- `abi_generator.rs` — Client-side ABI generation
- `scope_analyzer.rs` — Local variable optimization
- `constraint_optimizer.rs` — Account constraint validation
- `compression.rs` — Size optimization (VLE encoding, compact fields)
- `module_merger.rs` — Multi-file compilation support
- `performance.rs` — Runtime optimization patterns
- `bytecode_analyzer.rs` — Bytecode introspection and analysis

**Disassembly & Diagnostics:**
- `disassembler/` — Bytecode disassembly and debugging tools
  - `disasm.rs` — Main disassembler
  - `decoder.rs` — Instruction decoding
  - `pretty.rs` — Formatted output
  - `diagnostics.rs` — Analysis tools

### Error System (`src/error/`)
Comprehensive, modular error handling:

- `types.rs` — Error type definitions (CompilerError, ParseError, etc.)
- `registry.rs` — Centralized error code registry
- `formatting.rs` — Pluggable formatters (Terminal, JSON, LSP)
- `suggestions.rs` — Intelligent error fix suggestions
- `context.rs` — Error location tracking
- `templates/` — Error message templates by category
  - `codegen_errors.rs`, `parse_errors.rs`, `type_errors.rs`
- `integration.rs` — Error conversion between VMError and CompilerError

Key design: Use `ErrorCategory::` enum and error codes for structured errors. The `ERROR_SYSTEM` global provides centralized access.

### Type System (`src/type_checker/types.rs`)
- `TypeCheckerContext` — Main type checker (aliased as `DslTypeChecker`)
- `TypeNode` — Representation of Five types (u64, Account, Struct, etc.)
- `InterfaceInfo`, `InterfaceMethod` — Cross-program invocation support
- Type inference via `inference.rs` unifies implicit and explicit type handling

### Module System
- `src/module_resolver.rs` — Module discovery and import resolution
- `src/import_discovery.rs` — Import statement analysis
- `src/interface_registry.rs` — Cross-program interface tracking

### Configuration & Project Management
- `src/config/project_config.rs` — ProjectConfig for multi-file builds
- `src/five_file.rs` — .five file format (combined DSL + metadata)
- `src/metrics.rs` — Compilation metrics (TOML/JSON export)

### CLI Entrypoints (`src/bin/`)
- `five.rs` — Primary compiler CLI (compile, compile-multi, disasm, metrics subcommands)
- `compile_script.rs` — Simple single-file compilation
- `debug_compile.rs` — Pipeline debugging
- `extract_abi.rs` — Extract ABI from compiled bytecode
- `test_bytecode.rs` — Bytecode testing utility
- `enhanced_error_cli.rs` — Error formatting demo
- `import_discovery_demo.rs`, `standalone_demo.rs` — Feature demos

## Key Architectural Patterns

### Modular AST Generation
**File:** `src/bytecode_generator/ast_generator/mod.rs`

The AST generator breaks down bytecode generation by AST node type (one module per major type):
- `expressions.rs` — Literal, identifier, binary/unary ops, function calls
- `statements.rs` — Assignment, control flow
- `functions.rs` — Function definitions and dispatch
- `control_flow.rs` — If/else, match, loops
- `fields.rs` — Field definitions and access
- `types.rs` — Type checking during codegen
- `helpers.rs` — Common utilities (symbol resolution, constraint generation)

Each module exposes focused functions, e.g., `generate_expression()`, `generate_statement()`. The main `ast_generator()` function in `mod.rs` orchestrates these.

### Feature Flags
Three compilation modes control what gets included:
```
default = ["abi-pack"]
call-metadata = []      # Enable detailed function call tracking
security-audit = []     # Enable security rule validation
abi-pack = []           # Enable ABI packing (default)
```

When `security-audit` is disabled, the `security_rules` module provides a no-op stub (see `src/lib.rs` conditional compilation).

### Compilation Modes
- **Testing** — Includes test functions, enables diagnostic capture for better error messages
- **Deployment** — Excludes test functions, optimized for production bytecode size

### Multi-File Compilation
- Supported via `DslCompiler::compile_to_five_file_multi()` or `five compile-multi`
- Uses `ModuleMerger` to combine multiple AST modules
- Symbol resolution handled by `ModuleSymbolTable` in `type_checker/module_scope.rs`
- Function dispatch table is global across all modules

### Module Namespaces

Functions and definitions from different modules are **namespaced using qualified names** (`module::function`) to prevent name collisions and improve code organization.

**How it Works:**
1. **Namespace Qualification** (`src/bytecode_generator/module_merger.rs:qualify_with_module()`)
   - Functions from imported modules get prefixed with module name: `module_name::function_name`
   - Applies to: InstructionDefinition, FieldDefinition, EventDefinition, AccountDefinition
   - Prevents collisions: `helper1::calculate` and `helper2::calculate` are distinct

2. **ModuleScope Integration** (`src/type_checker/module_scope.rs`)
   - Cross-module symbol resolution with visibility enforcement
   - Type checker uses `resolve_symbol()` to find qualified names
   - Respects visibility: Public and Internal functions are importable, Private are not

3. **Feature Flag Control** (`src/compiler/pipeline.rs::CompilationConfig`)
   - `enable_module_namespaces: bool` (defaults: true)
   - Use `with_module_namespaces(false)` for backward compatibility (flat namespace mode)

**Examples:**

Module Structure:
```
src/
  math/add.v       → module: math
  math/multiply.v  → module: math
  utils/log.v      → module: utils
  main.v           → entry point (no prefix)
```

Function Names:
```
math::add()           // From math module
math::multiply()      // From math module
utils::log()          // From utils module
initialize()          // From main (entry point, no prefix)
```

**Backward Compatibility:**
```bash
# New behavior: qualified names (default)
five compile src/main.v

# Legacy behavior: flat namespace
five compile src/main.v --flat-namespace
```

**Tests:**
- `test_namespace_collision_prevention` — Verifies no collisions with qualified names
- `test_backward_compatibility_flat_namespace` — Validates legacy flat namespace mode
- `test_multi_module_compilation.rs` — 14 comprehensive multi-module tests

## Testing Strategy

### Integration Tests (`tests/`)
Located in `tests/`, each file tests end-to-end compilation:
- `lib.rs` — Main test suite (1600+ lines with 40+ test functions)
- `golden_bytecode.rs` — Bytecode snapshot regression tests
- `test_account_system.rs` — Account type validation
- `test_function_dispatch.rs` — Function routing
- `test_compression.rs` — Size optimization
- `test_visibility_system.rs` — Module visibility & privacy
- `test_multi_module_compilation.rs` — Multi-file builds
- etc.

**Running a specific test:**
```bash
cargo test --test lib resolve_nested_imports -- --nocapture
cargo test --test test_visibility_system -- --nocapture
```

### Unit Tests
Colocated with modules (e.g., `src/type_checker/mod.rs` has tests at the bottom). Run with:
```bash
cargo test --lib type_checker::
cargo test --lib parser::expressions::
```

### Test Fixtures
Examples and test data in `examples/` (e.g., `examples/multi_module/`). These are wired into integration tests to prevent regressions.

## Modifying Core Modules

### Adding a New Compiler Feature
1. **AST Representation** — Add node variant to `src/ast.rs` (AstNode enum)
2. **Parser** — Add parsing logic to `src/parser/` (usually `statements.rs` or `expressions.rs`)
3. **Type Checker** — Add validation to `src/type_checker/` (may need new module)
4. **Bytecode Generation** — Add opcode emission to appropriate `src/bytecode_generator/ast_generator/*.rs` file
5. **Tests** — Add end-to-end test to `tests/lib.rs` and unit tests to relevant modules
6. **Error Handling** — Add error codes to `src/error/templates/` if new error conditions arise

### Modifying Bytecode Generation
- Keep AST generation modular: each AST node type has a corresponding function/module
- Update `src/bytecode_generator/types.rs` if adding new bytecode metadata
- Update `src/bytecode_generator/opcodes.rs` if adding new opcodes
- Test with `cargo test --test golden_bytecode` to catch regressions

### Adding Optimization
- Register new optimization in `src/bytecode_generator/performance.rs` or create new module
- Benchmark with real-world examples in `examples/`
- Add feature flag if it's optional (use `#[cfg(feature = "...")]`)

## CPI (Cross-Program Invocation) Implementation

The compiler fully supports Cross-Program Invocation for calling external Solana programs via interface definitions.

### How CPI Works

**Interface Definition (DSL):**
```v
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (mint: pubkey, to: pubkey, authority: pubkey, amount: u64);
}
```

**Usage (Function Call):**
```v
pub mint_tokens(mint: account @mut, dest: account @mut) {
    // Arguments: 3 accounts (mint, dest, mint) + 1 data value (1000)
    SPLToken.mint_to(mint, dest, mint, 1000);
}
```

### CPI Account/Data Partitioning

When compiling CPI calls, the compiler automatically separates parameters:

- **Pubkey parameters** (`pubkey`) → Account arguments (must be simple identifiers referring to function parameters)
- **Other parameters** (u64, u8, bool, etc.) → Data arguments (serialized into instruction_data)

**Stack Contract Emitted (for VM handler):**
1. program_id (bottom)
2. instruction_data (array of bytes: discriminator + serialized data)
3. account_indices[] (parameter indices of account arguments, in order)
4. accounts_count (number of accounts)
5. INVOKE opcode (top)

### Key Implementation Files

**Compiler-side (bytecode generation):**
- `src/bytecode_generator/ast_generator/functions.rs`:
  - `is_pubkey_type()` — Identifies account parameters
  - `resolve_account_argument()` — Maps account arguments to parameter indices
  - `partition_interface_arguments()` — Separates accounts from data
  - `serialize_argument_to_buffer()` — Encodes data arguments
  - `serialize_instruction_data_at_compile_time()` — Builds instruction_data
  - `emit_interface_invoke()` — Emits correct stack contract

**VM-side (execution):**
- `five-vm-mito/src/handlers/system/invoke.rs` — INVOKE opcode handler (no changes needed)

### MVP Limitations

Current implementation supports:

✅ Account arguments (pubkey type parameters)
✅ Literal data arguments (u64, u32, u16, u8, bool, pubkey)
✅ Discriminators (single-byte and multi-byte)
✅ Borsh/Bincode serializers

❌ Non-literal data arguments (function parameters, expressions)
❌ Raw serializer mode
❌ Return value handling from CPI
❌ Account constraint enforcement (@signer, @mut, @initialized)

### Testing CPI

Run integration tests for CPI functionality:
```bash
cargo test --test lib test_cpi
```

Tests cover:
- SPL Token mint_to call (3 accounts + 1 data)
- Pure data calls (0 accounts)
- Local variable rejection
- Parameter count validation
- Duplicate account indices

### Future Enhancements

1. **Runtime data serialization** — Support non-literal arguments (function parameters, expressions)
2. **Return data handling** — Capture and pass return values from invoked programs
3. **Account constraints** — Validate @signer, @mut, @initialized attributes
4. **Raw serializer** — Support for custom serialization formats
5. **Account aliasing** — Better handling of same account in multiple slots

## Import Verification for Five Bytecode Accounts

Five VM now supports **import verification** to prevent bytecode substitution attacks. When importing another Five bytecode account via `use`, the import metadata is embedded in bytecode and verified at runtime.

### How It Works

**Compile-Time (Compiler):**
1. DSL source declares imports: `use "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";`
2. Compiler populates `ImportTable` with address or PDA seed information
3. Feature flag `FEATURE_IMPORT_VERIFICATION` set in bytecode header
4. Import metadata appended to bytecode (after main code, before function metadata)

**Runtime (VM):**
1. VM parses import metadata on bytecode load (zero-copy, no allocations)
2. During `CALL_EXTERNAL`, VM calls `verify_account()` with account key
3. Account address compared against authorized imports (address or PDA-derived)
4. Call rejected with `UnauthorizedBytecodeInvocation` error if not authorized
5. Backward compatible: bytecode without flag accepts any account

### DSL Syntax Support

```five
// Direct address import
use "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

// Future: PDA seed-based imports (metadata support ready)
use ["vault_seed", "user_seed"];

pub call_imported_function() {
    // CALL_EXTERNAL to imported bytecode
    // VM verifies account matches declared import
}
```

### Implementation Details

**Metadata Format (embedded in bytecode):**
```
[import_count: u8]
For each import:
  [import_type: u8]           (0 = address, 1 = PDA seeds)
  If address:
    [pubkey: 32 bytes]
  If PDA:
    [seed_count: u8]
    For each seed:
      [seed_len: u8]
      [seed_bytes: variable]
  [function_name_len: u8]
  [function_name: variable]
```

**Compiler Files:**
- `src/bytecode_generator/import_table.rs` — ImportTable struct and serialization
- `src/bytecode_generator/function_dispatch.rs` — Populates import_table during AST processing
- `src/bytecode_generator/header.rs` — Sets FEATURE_IMPORT_VERIFICATION flag
- `src/bytecode_generator/mod.rs` — Emits import metadata after main bytecode

**VM Files:**
- `five-vm-mito/src/metadata.rs` — Zero-copy ImportMetadata parser (no allocations)
- `five-vm-mito/src/context.rs` — ExecutionContext stores import_metadata
- `five-vm-mito/src/handlers/functions.rs` — CALL_EXTERNAL handler calls verify_account()
- `five-vm-mito/src/error.rs` — UnauthorizedBytecodeInvocation error type

### Performance Characteristics

**Compiler (Local/WASM):**
- Serialization: ~5-10ms for typical imports (can use allocations, speed not critical)
- Memory: HashMap + Vec structures allowed

**VM (On-Chain):**
- Verification: <1μs for address imports (direct memcmp)
- PDA mode: ~2μs (PDA derivation cost, unavoidable)
- Memory: Zero allocations, stack-only, direct bytecode references
- No HashMap, no Vec, no String allocation during verification

### Security Properties

✅ **Prevents Bytecode Substitution** — Attacker cannot swap Five bytecode account at runtime
✅ **PDA Support** — Works with PDA-derived Five bytecode accounts
✅ **Compile-Time Authorization** — Import declarations are security policy
✅ **Backward Compatible** — Old bytecode without flag continues to work
✅ **Zero-Trust Runtime** — VM doesn't trust transaction caller's account ordering
✅ **Auditability** — Disassembly shows authorized Five bytecode accounts

### Testing

**Compiler Tests:**
```bash
cargo test test_import_verification_bytecode_generation_address
cargo test test_import_verification_bytecode_generation_pda
cargo test test_no_imports_no_verification_flag
```

**VM Tests:**
```bash
cargo test --lib call_external_verification
```

**End-to-End Tests:**
```bash
cargo test test_import_verification_end_to_end_address
cargo test test_import_verification_prevents_attack
```

### Future Enhancements

- **PDA Callback Integration** — Implement Solana SDK PDA derivation in VM
- **Multiple Import Modes** — Support different serialization formats
- **Import Constraints** — Combine with account constraint validation (@signer, @mut)
- **Runtime Caching** — Optional caching for repeated calls (with feature flag)

## Debugging & Diagnostics

### Bytecode Disassembly
```bash
cargo run --bin five -- disasm <bytecode.fbin>
cargo run --bin five -- disasm <bytecode.fbin> --show-accounts
```

### Compilation Debugging
```bash
cargo run --bin debug_compile <file.five>  # Prints AST + bytecode generation steps
RUST_LOG=debug cargo test -- --nocapture    # Enable debug output in tests
```

### Verbose Type Checking
Type checker context has a `symbol_table` field. Add debug prints in `type_checker/mod.rs` to inspect scope resolution:
```rust
eprintln!("Symbol table: {:?}", self.symbol_table);
```

### Metrics Collection
```bash
cargo run --bin five -- compile test.five --metrics --metrics-output metrics.json
```
Supports JSON, TOML, CSV export formats. See `src/metrics.rs`.

## Dependencies
- **five-protocol** — Five Protocol library (same org)
- **five-vm-mito** — Five VM MITO for error types and compatibility
- **heapless** — Stack-safe String<32> for WASM compatibility
- **clap** — CLI argument parsing
- **serde/serde_json/toml** — Serialization (metrics, configs)
- **web-time** — Cross-platform timing (metrics)

## Common Issues & Fixes

### Compilation fails with "script name identifier" error
This parser debugging message indicates the script block is missing its name. Check that your `.five` file has `script MyName { ... }` syntax.

### Type mismatch in account access
Check `src/type_checker/types.rs` for account type definitions and ensure all account field types match between definition and usage.

### Bytecode size explosion
Run with `--v2-preview` and check `src/bytecode_generator/compression.rs` for optimization opportunities. Use `five metrics` to identify large functions.

### Multi-module import resolution fails
Check `src/module_resolver.rs` and `src/import_discovery.rs`. Ensure import targets are valid Solana pubkeys or local paths. Validate with `five import-discovery <file>`.

## Development Notes

- **Hot Path Optimization** — Tokenizer and parser are performance-sensitive; avoid unnecessary allocations in these modules (check clippy warnings).
- **Error Messages** — All user-facing errors go through the `ERROR_SYSTEM` in `src/error/mod.rs`. Update templates in `src/error/templates/` for new error messages, not ad-hoc error strings.
- **Binary Size** — The compiler targets WASM, so binary size matters. Use `heapless::String` where appropriate, and check `src/bytecode_generator/compression.rs` for code bloat patterns.
- **Feature Interactions** — The `security-audit` feature can be toggled off; ensure new code gracefully degrades when disabled (see `src/lib.rs` conditional compilation).
