# 5IVE AMM Project - Gemini Context

This project is a 5IVE VM application, utilizing the 5IVE DSL to build compact bytecode programs for Solana.

## Project Overview

- **Purpose:** A decentralized automated market maker (AMM) built with 5IVE VM.
- **Main Technologies:** 5IVE DSL (`.v` files), 5IVE CLI (`@5ive-tech/cli`), Node.js.
- **Architecture:** 
  - `src/`: 5IVE DSL source files. `main.v` currently contains a basic counter and AMM logic.
  - `tests/`: Automated tests for DSL functions using the `@test-params` convention.
  - `build/`: Target directory for compiled `.five` bytecode and ABI artifacts.
  - `five.toml`: Central configuration for project settings, optimizations, and deployment.

## Building and Running

Commands are managed via `package.json` scripts using the `5ive` CLI:

- **Build:** `npm run build` (Compiles all `.v` files in `src/`)
- **Build (Release):** `npm run build:release` (Optimized build)
- **Build (Debug):** `npm run build:debug` (Build with debug symbols)
- **Test:** `npm test` (Runs `5ive test` which discovers and executes `test_*` functions)
- **Watch:** `npm run watch` (Auto-compiles on changes)
- **Deploy:** `npm run deploy` (Deploys to the network specified in `five.toml`)

## Development Conventions

### 5IVE DSL Syntax Rules
- **Account Fields:** All fields in an `account` block MUST end with a semicolon `;`.
- **Authorization:** Use `account @signer` for parameters that must sign the transaction. Access the public key via `.key` (e.g., `caller.key`).
- **Initialization:** Use the attribute stack for init: `Type @mut @init(payer=name, space=bytes) @signer`.
- **Assertions:** Use `require(condition)` for safety checks.
- **Return Types:** Declare return types using `-> Type` (e.g., `fn get_val() -> u64`).
- **Immutability:** Variables defined with `let` are immutable. Use `let mut` for reassignable variables.

### Testing Practices
- Test functions must be public and prefixed with `test_` (e.g., `pub test_add`).
- Use `// @test-params <args> <expected>` comments above test functions to define test cases for the `5ive test` runner.

## Key Files
- `five.toml`: Project configuration including optimization levels and deployment targets.
- `AGENTS.md`: Technical specification and "Source of Truth" for the 5IVE DSL language features and agent workflows. **Refer to this first for language syntax questions.**
- `src/main.v`: Primary contract logic.
- `tests/main.test.v`: Core test suite.

## Reference
- [5IVE VM Documentation](https://five-vm.dev)
- [AGENTS.md](./AGENTS.md) for deep DSL feature inventory and canonical patterns.
