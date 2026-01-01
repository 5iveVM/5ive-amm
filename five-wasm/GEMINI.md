# Five VM WASM Project Context

## Project Overview
**five-vm-wasm** provides WebAssembly bindings for the Five VM, enabling the execution and testing of Five DSL smart contracts in browser and Node.js environments. It serves as a bridge between the core Rust-based Five VM (MitoVM) and JavaScript/TypeScript applications, facilitating local development, testing, and deployment workflows.

**Key Capabilities:**
*   **WASM Compilation:** Compiles the Rust VM and DSL compiler into optimized WebAssembly modules.
*   **Hybrid Execution:** Supports "partial execution," allowing the VM to run locally until it hits a system call (like `INIT_PDA` or `INVOKE`) that requires a live Solana cluster, at which point it reports the specific stop reason.
*   **Tooling:** Includes a CLI for testing and deployment, a playground for experimentation, and extensive TypeScript wrappers.

## Architecture
*   **Core (Rust):** Located in `src/lib.rs`. Wraps `five-vm-mito` and `five-dsl-compiler`. Exposes functions for compilation, execution, and bytecode analysis.
*   **Wrapper (TypeScript):** Located in `wrapper/`. Provides a type-safe API (`FiveVMWrapper`) over the raw WASM functions.
*   **Applications:**
    *   `app/`: Contains the logic for the deployment CLI, test runner, and playground.
    *   `five-cli/`: (Structure implies a separate CLI tool, likely consuming the WASM build).
*   **Test Scripts:** `test-scripts/` contains a large suite of `.v` (Five DSL) files for validation.

## Build & Run Commands

### Building
The project uses `wasm-pack` to build targets for different environments.

*   **Full Build:** `./build.sh` (Builds all targets)
*   **Web Target:** `npm run build` (Outputs to `pkg/`)
*   **Node.js Target:** `npm run build:nodejs` (Outputs to `pkg-node/`)
*   **Bundler Target:** `npm run build:bundler` (Outputs to `pkg-bundler/`)

### Testing
*   **Run All Tests:** `npm test`
*   **Integration Tests:** `npm run test:integration`
*   **WASM-Specific Tests:** `npm run test:wasm`
*   **Browser Tests:** `wasm-pack test --headless --firefox`

### CLI Tools
*   **WASM Test CLI:** `npm run wasm-test`
*   **Deploy CLI:** `npm run deploy`
*   **Benchmarks:** `npm run benchmark`

## Key Conventions
*   **Language:** Rust for core logic, TypeScript for tooling and wrappers.
*   **Error Handling:** The VM uses an "Honest Execution" model. It does not mock Solana system calls; instead, it pauses execution and reports `StoppedAtSystemCall`, allowing the caller to handle the external dependency.
*   **File Extensions:** Five DSL files use `.v`.
*   **Magic Bytes:** Valid Five bytecode starts with `5IVE`.

## Important Files
*   `src/lib.rs`: Main Rust entry point for WASM bindings.
*   `wrapper/index.ts`: Main TypeScript entry point.
*   `package.json`: NPM dependencies and scripts.
*   `Cargo.toml`: Rust dependencies and crate configuration.
*   `CLAUDE.md`: Detailed developer guide and architectural notes.
