# Five SDK - Developer Context (GEMINI.md)

## Project Overview

**Five SDK** (`five-sdk`) is a client-agnostic TypeScript library designed for interacting with the **Five VM** ecosystem on Solana. It provides a standardized interface for compiling Five scripts (.v), generating optimized bytecode, encoding parameters using Variable Length Encoding (VLE), and constructing Solana instructions.

**Key Characteristics:**
*   **Zero Dependencies:** Designed to work with *any* Solana client (web3.js, Anchor, Metaplex, etc.) by outputting raw instruction data rather than depending on a specific `Connection` or `Provider` implementation.
*   **WASM-Powered:** Leverages `five_vm_wasm_bg.wasm` (compiled from Rust) for the heavy lifting: bytecode compilation, static analysis, and local VM execution.
*   **VLE Encoding:** Implements custom parameter encoding to minimize transaction size (30-50% reduction).
*   **Local Execution:** Includes a full WASM-based VM for off-chain testing and development.

## Architecture & Core Components

The SDK is organized around static utility classes and a central `FiveSDK` entry point.

### 1. Main Entry Point: `src/FiveSDK.ts`
This massive class (~4k lines) acts as the public facade. It uses a **static method pattern** to avoid state management issues and simplify usage.
*   **Compilation:** `FiveSDK.compile()` delegates to `BytecodeCompiler`.
*   **Execution:** `FiveSDK.executeLocally()` delegates to `FiveVM`.
*   **Deployment:** `FiveSDK.generateDeployInstruction()` creates the specific byte sequence required by the on-chain Five program (Discriminator `8`).
*   **Interaction:** `FiveSDK.generateExecuteInstruction()` encodes function calls (Discriminator `9`) using VLE.

### 2. WASM Integration (`src/wasm/`)
The SDK wraps raw WASM bindings to provide a TypeScript-friendly API.
*   **`loader.ts`**: A sophisticated loader that resolves the WASM binary path across different environments (Node.js vs Browser) and handles initialization (`initSync` vs `default`).
*   **`vm.ts`**: Wraps `FiveVMWasm`. Handles `WasmAccount` conversion and result parsing (converting Rust `Ok/Err` strings to JS objects).
*   **`compiler.ts`**: Wraps `FiveCompiler`. Handles source validation, multi-module compilation, and ABI generation.

### 3. Compilation Pipeline (`src/compiler/`)
*   **`BytecodeCompiler.ts`**: Manages the compilation process. It lazy-loads the WASM compiler, normalizes ABI output, and formats error messages into `CompilationSDKError` objects.

### 4. Testing Infrastructure (`src/testing/`)
*   **`TestRunner.ts`**: A programmatic test runner capable of executing test suites defined in JSON or code. It runs tests in parallel, enforces Compute Unit (CU) limits, and validates return values.

## Building and Running

**Prerequisites:**
*   Node.js (v18+)
*   The `assets/vm/` directory must contain valid WASM binaries (`five_vm_wasm_bg.wasm`, etc.). These are typically built from the `five-vm-wasm` Rust project.

**Commands:**
*   **Build SDK:**
    ```bash
    npm run build
    # Compiles TS to dist/ and copies WASM assets from src/assets/vm/ to dist/assets/vm/
    ```
*   **Run Tests:**
    ```bash
    npm test
    # Executes the basic usage example as a smoke test
    ```
*   **Run Examples:**
    ```bash
    node examples/basic-usage.js
    ```

## Key Development Conventions

1.  **Static over Instance:** Prefer static methods on `FiveSDK` for stateless operations (compilation, instruction generation).
2.  **Lazy WASM Loading:** Do not import or initialize WASM modules at the top level. Use `initializeComponents()` or similar lazy-loading patterns to ensure the SDK works in environments where WASM might not be immediately needed.
3.  **Error Handling:** Use specific error classes (`FiveSDKError`, `CompilationSDKError`) rather than generic Errors.
4.  **Client Agnostic:** Never import `@solana/web3.js` in production code unless absolutely necessary for type definitions. The SDK should output `Buffer` or `Uint8Array` data, not specific Solana web3 objects.
5.  **VLE Everywhere:** All parameter encoding for execution *must* use the VLE encoder. Do not use standard borsh/layout encoding for Five VM instructions.

## Directory Map

*   `src/FiveSDK.ts`: **Core.** Main API surface.
*   `src/wasm/`: **Bridge.** Connects TS to Rust/WASM.
*   `src/compiler/`: **Logic.** Bytecode compilation logic.
*   `src/encoding/`: **Logic.** VLE parameter encoding.
*   `src/testing/`: **Tooling.** Test runner and fixtures.
*   `src/assets/wasm/`: **Binary.** Location of the pre-compiled WASM binary.
