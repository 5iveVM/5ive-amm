# WARP.md

This file provides guidance for working with the 5ive DSL compiler in this repository.

## Project Overview

This is the **5ive DSL Compiler** — an off-chain Rust compiler that compiles `.five` DSL syntax into 5IVE bytecode for execution by the 5ive VM (five-vm-mito). The compiler supports multi-file compilation, type checking, bytecode optimization, and ABI generation.

## Essential Commands

### Build & Test
```bash
cargo build                                    # Compile with default features (abi-pack enabled)
cargo build --features security-audit         # Include security audit features
cargo test                                     # Run all unit and integration tests
cargo test -- --nocapture                     # Show DSL compiler output in tests
cargo test --test <test_name>                  # Run specific integration test (e.g., lib, golden_bytecode)
cargo test --lib type_checker::                # Run unit tests in specific module
```

### Run CLI Tools
```bash
# Main compiler CLI
cargo run --bin five -- compile <file.five>
cargo run --bin five -- compile <file.five> --mode deployment
cargo run --bin five -- compile-multi <main.five> <lib.five>
cargo run --bin five -- metrics <file.five> --metrics-output metrics.json

# Debugging & analysis
cargo run --bin debug_compile <file.five>     # Debug compilation pipeline
cargo run --bin five -- disasm <bytecode.fbin>  # Disassemble bytecode
cargo run --bin extract_abi <file.five>        # Extract ABI from bytecode
cargo run --bin dump_tokens <file.five>        # Show tokenization output
```

### Code Quality
```bash
cargo fmt                                     # Format code
cargo clippy --all-targets --all-features    # Lint all targets with all features
```

### Running Single Tests
```bash
# Run specific integration test function
cargo test --test lib resolve_nested_imports -- --nocapture
cargo test --test test_visibility_system -- --nocapture

# Run unit tests for specific module
cargo test --lib parser::expressions::
```

## Architecture Overview

### Compiler Pipeline (5ive Stages)
The compiler processes DSL source through a unified pipeline in `src/compiler/pipeline.rs::CompilationPipeline`:

1. **Tokenization** (`src/tokenizer.rs`) — Lexical analysis into tokens
2. **Parsing** (`src/parser/`) — Token stream → Abstract Syntax Tree (AST)
   - `mod.rs` — Main parser orchestration
   - `expressions.rs` — Expression parsing
   - `statements.rs` — Statement parsing
   - `blocks.rs` — Block/scope parsing
3. **Type Checking** (`src/type_checker/`) — Semantic analysis and type validation
   - `mod.rs` — Main type checker facade
   - `types.rs` — TypeCheckerContext (the actual implementation)
   - `module_scope.rs` — Cross-module symbol resolution
   - `inference.rs` — Type inference engine
   - `expressions.rs`, `statements.rs`, `functions.rs` — Per-node type validation
4. **Bytecode Generation** (`src/bytecode_generator/`) — AST to bytecode
5. **Output** — Bytecode serialization, ABI generation, metrics collection

All public compiler methods delegate to `CompilationPipeline` to ensure consistent behavior across different compilation APIs. This design eliminated ~300 lines of duplicate code.

### Bytecode Generator (Modular Architecture)
The bytecode generator (`src/bytecode_generator/`) is split into focused modules:

**Core Components:**
- `types.rs` — Data structures (FunctionInfo, FieldInfo, AccountRegistry, FIVEABI)
- `opcodes.rs` — Opcode emission and utilities
- `function_dispatch.rs` — O(1) function routing via dispatch tables
- `account_system.rs` — Account type definitions and validation
- `account_utils.rs` — Unified account type detection

**AST Generation (Modular):**
- `ast_generator/` — One module per AST node type:
  - `expressions.rs` — Literals, identifiers, binary/unary ops, function calls
  - `statements.rs` — Assignment, control flow
  - `functions.rs` — Function definitions and dispatch
  - `control_flow.rs` — If/else, match, loops
  - `fields.rs` — Field definitions and access
  - `helpers.rs` — Symbol resolution, constraint generation

**Optimization & Analysis:**
- `abi_generator.rs` — Client-side ABI generation
- `scope_analyzer.rs` — Local variable optimization
- `constraint_optimizer.rs` — Account constraint validation
- `compression.rs` — Size optimization (VLE encoding, compact fields)
- `module_merger.rs` — Multi-file compilation support
- `performance.rs` — Runtime optimization patterns
- `bytecode_analyzer.rs` — Bytecode introspection and analysis

**Disassembly:**
- `disassembler/` — Bytecode disassembly and debugging tools

### Error System (`src/error/`)
Comprehensive, modular error handling:

- `types.rs` — Error type definitions (CompilerError, ParseError, etc.)
- `registry.rs` — Centralized error code registry
- `formatting.rs` — Pluggable formatters (Terminal, JSON, LSP)
- `suggestions.rs` — Intelligent error fix suggestions
- `context.rs` — Error location tracking
- `templates/` — Error message templates by category
- `integration.rs` — Error conversion between VMError and CompilerError

**Key pattern:** Use `ErrorCategory::` enum and error codes for structured errors. Access via `ERROR_SYSTEM` global.

### Module System
- `src/module_resolver.rs` — Module discovery and import resolution
- `src/import_discovery.rs` — Import statement analysis
- `src/interface_registry.rs` — Cross-program interface tracking
- `src/config/project_config.rs` — ProjectConfig for multi-file builds

### Feature Flags
Three compilation modes control what gets included:
```
default = ["abi-pack"]
call-metadata = []      # Enable detailed function call tracking
security-audit = []     # Enable security rule validation
abi-pack = []           # Enable ABI packing (default)
```

When `security-audit` is disabled, `src/security_rules.rs` provides a no-op stub (see `src/lib.rs` conditional compilation).

### Compilation Modes
- **Testing** — Includes test functions, enables diagnostic capture
- **Deployment** — Excludes test functions, optimized for production bytecode size

### Multi-File Compilation
- Use `five compile-multi` or `DslCompiler::compile_to_five_file_multi()`
- `ModuleMerger` combines multiple AST modules
- Symbol resolution via `ModuleSymbolTable` in `type_checker/module_scope.rs`
- Function dispatch table is global across all modules

## Key Development Patterns

### Modifying the Compiler Pipeline
When adding a new feature:

1. **AST Representation** — Add node variant to `src/ast.rs` (AstNode enum)
2. **Parser** — Add parsing logic to `src/parser/` (usually `statements.rs` or `expressions.rs`)
3. **Type Checker** — Add validation to `src/type_checker/` (may need new module)
4. **Bytecode Generation** — Add opcode emission to appropriate `src/bytecode_generator/ast_generator/*.rs` file
5. **Tests** — Add end-to-end test to `tests/lib.rs` and unit tests to relevant modules
6. **Error Handling** — Add error codes to `src/error/templates/` if needed

### Testing Strategy
- Integration tests in `tests/` test end-to-end compilation scenarios
- Unit tests are colocated with modules in `src/`
- Test fixtures and examples in `examples/` directory
- Always add tests to prevent regressions when adding features

### Bytecode Generation
- Keep AST generation modular: each AST node type has a corresponding function/module
- Update `src/bytecode_generator/types.rs` for new bytecode metadata
- Update `src/bytecode_generator/opcodes.rs` for new opcodes
- Test with `cargo test --test golden_bytecode` to catch regressions

### Error Messages
All user-facing errors must go through the `ERROR_SYSTEM` in `src/error/mod.rs`. Update templates in `src/error/templates/` for new error messages, not ad-hoc error strings.

## Dependencies
- **five-protocol** — 5ive Protocol library (sibling in five-org)
- **five-vm-mito** — 5ive VM MITO for error types and compatibility
- **heapless** — Stack-safe String<32> for WASM compatibility
- **clap** — CLI argument parsing
- **serde/serde_json/toml** — Serialization (metrics, configs)

## Coding Style
- Rust 2021 edition, 4-space indentation
- `snake_case` for modules/functions, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for consts
- Keep functions small and composable per compiler stage
- Prefer explicit types around parsing/type-checking boundaries
- Avoid unnecessary allocations in hot paths (tokenizer/parser)
- Follow `rustfmt` defaults and `clippy` guidance

## Commit Practices
- Short, imperative summaries (e.g., "Add TOML metrics export", "Fix dispatcher HALT logic")
- Commits should be scoped and buildable
- Include test updates with logic changes
