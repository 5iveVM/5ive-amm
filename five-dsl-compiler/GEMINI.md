# Five DSL Compiler Project Context

## Project Overview

**Name:** `five-dsl-compiler`
**Purpose:** A modular Rust-based compiler for the Five DSL, targeting the Five VM (Virtual Machine) running on Solana. It translates high-level `.five` (or `.stacks`) source code into optimized `STKS` bytecode.
**Ecosystem:** Part of the `five-org` suite, integrating with `five-protocol` and `five-vm-mito`.

## Architecture

The compiler follows a classic multi-stage pipeline architecture, designed for modularity and maintainability:

1.  **Tokenizer (`src/tokenizer.rs`):** Converts source text into a stream of tokens.
2.  **Parser (`src/parser.rs`):** Consumes tokens to build an Abstract Syntax Tree (AST) defined in `src/ast.rs`.
3.  **Type Checker (`src/type_checker.rs`):** Performs semantic analysis and type validation on the AST.
4.  **Bytecode Generator (`src/bytecode_generator/mod.rs`):** The core orchestrator that translates the AST into bytecode. It delegates specific tasks to specialized sub-modules:
    *   `ast_generator.rs`: Traverses the AST to emit opcodes.
    *   `account_system.rs`: Manages account definitions and field layouts.
    *   `function_dispatch.rs`: Handles function calls and dispatch tables.
    *   `opcodes.rs`: Low-level opcode emission utilities.
    *   `compression.rs` & `performance.rs`: Optimization passes.

## Key Components

*   **Unified CLI (`src/bin/five.rs`):** The primary entry point for users. It supports subcommands like:
    *   `compile`: Compiles a single script.
    *   `compile-multi`: Compiles multiple modules.
    *   `analyze`: Provides source code analysis and metrics.
    *   `inspect`: Disassembles and inspects compiled bytecode.
    *   *(Planned)*: `benchmark`, `watch`, `init`.
*   **Library (`src/lib.rs`):** Exposes the compiler's functionality as a reusable crate.

## Building and Running

The project uses standard Cargo commands:

*   **Build:** `cargo build` (debug) or `cargo build --release` (optimized).
*   **Test:** `cargo test` to run the comprehensive test suite in `tests/`.
*   **Run CLI:** `cargo run --bin five -- <command> <args>`
    *   Example: `cargo run --bin five -- compile examples/security_example.v`

## Development Conventions

*   **Modularity:** Logic is heavily compartmentalized (e.g., `bytecode_generator` has many sub-modules). New features should follow this pattern.
*   **Error Handling:** Custom error types are used throughout (e.g., `VMError`), often with integration into the `five-vm-mito` error system.
*   **Metrics:** The compiler includes a robust metrics collection system (`metrics.rs`) to track performance and bytecode stats.
*   **Feature Flags:** Security auditing (`security-audit`) and other features are gated for flexibility.

## Key Files

*   `Cargo.toml`: Project dependencies and metadata.
*   `src/lib.rs`: Library root, module exports.
*   `src/bin/five.rs`: Main CLI implementation.
*   `src/bytecode_generator/mod.rs`: Central logic for code generation.
*   `src/ast.rs`: Definition of the language syntax tree.
