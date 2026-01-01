# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Five VM WASM is a WebAssembly binding for the Five VM that provides browser and Node.js compatibility for bytecode execution and testing. It wraps the Five VM Mito engine with a comprehensive TypeScript interface, offering partial execution capabilities, deployment tooling, and performance benchmarking.

**Core Architecture:**
- Rust-based WASM module with TypeScript wrapper layer
- Partial execution with honest status reporting about system call stops
- Multi-target builds (web, Node.js, bundler)
- Comprehensive testing and deployment infrastructure
- React-based UI components for deployment management

## Build and Development Commands

### WASM Module Building
```bash
# Build all WASM targets using the comprehensive build script
./build.sh

# Build individual targets
npm run build              # Web target (ES modules)
npm run build:nodejs       # Node.js target (CommonJS)
npm run build:bundler      # Bundler target (webpack, etc.)
npm run build:all          # Build all targets

# Development builds (faster, less optimized)
npm run dev
```

### Testing Commands
```bash
# Run all tests (wasm-pack headless Firefox + Jest suites)
npm test

# Run specific test categories
npm run test:integration   # Integration tests
npm run test:wasm          # WASM compiler integration tests
npm run test:partial       # Partial execution tests
npm run test:validation    # Bytecode validation tests
npm run test:deployment    # Deployment service tests
npm run test:playground    # Playground functionality tests
npm run test:ts            # All TypeScript/Jest tests

# Run WASM-specific tests
npm run test:node          # Node.js environment tests
wasm-pack test --headless --firefox  # Browser environment tests (used by npm test)

# Run Rust integration tests against Five DSL fixtures
npm test                    # Standard WASM execution tests
FIVE_TEST_MODE=wasm npm test        # Explicit WASM-only mode
FIVE_TEST_MODE=localnet npm test    # Test against running localnet validator
```

### Development Tools
```bash
# Run CLI tools
npm run wasm-test          # WASM test CLI
npm run deploy             # Deployment CLI
npm run example            # Run integration example
npm run playground         # Run playground demo

# Performance analysis
npm run benchmark          # Performance benchmarking
npm run size-analysis      # Bundle size analysis

# Code quality
npm run lint               # ESLint
npm run type-check         # TypeScript type checking
```

## Architecture Overview

### WASM Module Core (`src/lib.rs`)
The Rust core provides WASM bindings for the Five VM with these key components:

**Execution Engine:**
- `FiveVMWasm` - Main VM wrapper with execution capabilities
- `TestResult` - Detailed execution results with honest status reporting
- `ExecutionStatus` - Enum for execution outcomes (Completed, StoppedAtSystemCall, etc.)
- Partial execution support for system calls that can't run in WASM

**Account System:**
- `WasmAccount` - WASM-compatible account representation
- Account state management with proper serialization
- Support for signer, writable, and ownership constraints

**Analysis Tools:**
- `BytecodeAnalyzer` - Instruction-level bytecode analysis
- `FiveVMState` - VM state inspection and debugging
- Validation utilities for bytecode format checking

### TypeScript Wrapper Layer (`wrapper/index.ts`)
Provides a high-level TypeScript interface:

**Core Wrapper Classes:**
- `FiveVMWrapper` - Main TypeScript VM interface with type safety
- `VMPerformanceBenchmark` - Performance measurement utilities
- `BundleAnalyzer` - WASM bundle size analysis

**Type System:**
- `FiveAccount` - TypeScript-friendly account interface
- `PartialExecutionResult` - Honest execution result reporting
- `BytecodeAnalysis` - Structured bytecode analysis results
- `FiveVMConstants` - VM constants with proper typing

### Application Layer (`app/`)
Complete deployment and testing infrastructure:

**Core Services:**
- `wasm-compiler.ts` - WASM-based execution service (not compilation)
- `deployment-service.ts` - Bytecode deployment management
- `deployment-ui.ts` - React-based deployment interface

**CLI Tools:**
- `wasm-test-cli.ts` - Command-line testing interface
- `deploy-cli.ts` - Deployment automation CLI

**React Components:**
- `deployment-panel.tsx` - Main deployment UI
- `deployment-playground.tsx` - Interactive testing playground
- `deployment-history.tsx` - Deployment tracking and history

### Multi-Target Build System
**Build Outputs:**
- `pkg/` - Web target (ES modules) for browser use
- `pkg-node/` - Node.js target (CommonJS) for server applications
- `pkg-bundler/` - Bundler target for webpack/rollup integration
- `pkg-template/` - TypeScript definitions template

**Build Configuration:**
- Optimized WASM with `-Oz` flag and `wasm-opt` post-processing
- Feature flags for debug logging and enhanced error reporting
- Size-optimized builds with LTO and panic=abort

### Test Infrastructure (`tests/`)
Comprehensive test coverage across multiple dimensions:

**Integration Testing:**
- `integration.test.ts` - End-to-end WASM execution tests
- `partial_execution.test.ts` - Partial execution and system call handling
- `validation.test.ts` - Bytecode validation and error handling

**Application Testing:**
- `deployment.test.ts` - Deployment service functionality
- `playground.test.ts` - Interactive playground features
- `wasm_compiler_integration.test.ts` - WASM module integration

### Test Scripts Collection (`test-scripts/`)
Extensive collection of Five DSL test cases organized by feature:

**Language Features:**
- `01-language-basics/` - Functions, variables, basic operations
- `02-operators-expressions/` - Arithmetic, boolean logic, comparisons
- `03-control-flow/` - Conditionals, loops, branching

**Blockchain Integration:**
- `04-account-system/` - Account constraints, state management
- `05-blockchain-integration/` - PDA operations, clock access, tuple handling
- `06-advanced-features/` - Arrays, multiple parameters, string operations

**Error Handling and Pattern Matching:**
- `07-error-system/` - Error messages and syntax validation
- `08-match-expressions/` - Option/Result pattern matching

## Code Organization Guide

### Directory Structure at a Glance
- **`src/`** - Rust source code (builds to WASM via wasm-pack)
  - `lib.rs` - Main WASM bindings and VM wrapper
  - `tests.rs` - Rust integration tests that execute DSL fixtures
- **`wrapper/`** - TypeScript wrapper around WASM module; re-exported from `wrapper/index.ts`
- **`app/`** - TypeScript services, CLIs, and React UI
  - `wasm-compiler.ts` - Execution service (not compilation)
  - `deployment-service.ts` - Bytecode deployment management
  - `wasm-test-cli.ts`, `deploy-cli.ts` - Command-line tools
  - `components/`, `hooks/`, `styles/`, `ui/` - React UI structure
- **`tests/`** - Jest test suites for TypeScript code
- **`test-scripts/`** - Five DSL source files used by Rust integration tests
- **`pkg*/`** - Build output directories (regenerate via `npm run build*`, do not edit)
  - `pkg/` - Web target (ES modules)
  - `pkg-node/` - Node.js target (CommonJS)
  - `pkg-bundler/` - Bundler target (webpack, rollup, etc.)
- **`scripts/`**, **`examples/`**, **`reports/`** - Utilities, demos, and build artifacts

## Development Patterns

### WASM Integration Best Practices
**Memory Management:**
- Always call `dispose()` on VM instances to free WASM memory
- Use try-catch blocks around WASM calls for proper error handling
- Leverage TypeScript interfaces for type safety across the WASM boundary

**Partial Execution Handling:**
```typescript
// Use executePartial() for honest status reporting
const result = await vm.executePartial(inputData, accounts);
if (result.status === 'StoppedAtSystemCall') {
    console.log(`Stopped at ${result.stoppedAtOpcodeName} - this requires real Solana context`);
}
```

### Testing Strategy
**Multi-Environment Testing:**
- Test in both browser (via wasm-pack test) and Node.js environments
- Use the extensive test-scripts collection for regression testing
- Implement both unit tests and integration tests

**Performance Monitoring:**
- Use `VMPerformanceBenchmark` for execution profiling
- Monitor WASM bundle sizes across different targets
- Track compute unit consumption for cost analysis

### Deployment Workflow
**Development Cycle:**
1. Develop Five DSL scripts in `test-scripts/`
2. Test execution using the WASM playground or CLI
3. Validate partial execution behavior for system calls
4. Deploy using the deployment service
5. Monitor performance and resource usage

**Build Optimization:**
- Use release builds for production with full optimization
- Enable specific features only when needed (`debug-logs`, `enhanced-errors`)
- Optimize bundle sizes for target environments

## Test Infrastructure Details

### DSL Test Fixtures
The `test-scripts/` directory contains Five DSL test cases organized by feature category:
- `01-language-basics/` through `03-control-flow/` - Language fundamentals
- `04-account-system/` through `06-advanced-features/` - Blockchain integration features
- `07-error-system/` and `08-match-expressions/` - Error handling and pattern matching

Rust integration tests in `src/tests.rs` execute these fixtures. Individual test parameters can be specified via `// @test-params` comments in fixture files.

### Test Mode Configuration
The `FIVE_TEST_MODE` environment variable controls test execution context:
- **`FIVE_TEST_MODE=wasm` (default)** - Execute bytecode in the WASM VM environment; system calls that require real Solana context will stop execution
- **`FIVE_TEST_MODE=localnet`** - Execute against a running Solana localnet validator; requires `five-cli` and active validator with `solana-test-validator`

Example: `FIVE_TEST_MODE=localnet npm test` to test against localnet.

## Honest Execution Model

### System Call Limitations
The WASM environment cannot execute certain operations that require real Solana context:

**Operations that Stop Execution:**
- `INIT_PDA` - Requires actual Solana account creation
- `INVOKE` / `INVOKE_SIGNED` - Requires cross-program invocations
- System calls requiring RPC access

**Execution Status Reporting:**
- `Completed` - All operations executed successfully
- `StoppedAtSystemCall` - Hit operation requiring real Solana context
- `StoppedAtInitPDA` - Specifically stopped at PDA initialization
- `StoppedAtInvoke` - Stopped at cross-program invocation
- `ComputeLimitExceeded` - Hit compute unit limit
- `Failed` - Execution error occurred

### Testing Philosophy
**What Can Be Tested:**
- All arithmetic and logical operations
- Stack and memory management
- Control flow and function calls
- Account constraint validation (with mock accounts)
- Local variable and parameter handling

**What Requires Real Solana:**
- Actual account initialization and modification
- Cross-program invocations
- Real-time system variable access (clock, rent)
- Network-dependent operations

This honest execution model ensures developers understand exactly what's being tested versus what requires full Solana deployment.

## Dependencies and External Modules

This repository depends on sibling workspace modules that must be available locally:

**Rust Dependencies (from workspace):**
- **`five-vm-mito`** (../five-vm-mito) - The core Five VM Mito execution engine
  - Feature flags: `debug-logs`, `enhanced-errors` (passed through from CLAUDE.md build)
  - Default features disabled for WASM compatibility
- **`five-dsl-compiler`** (../five-dsl-compiler) - DSL parsing, tokenization, and compilation to bytecode
  - Provides `DslCompiler`, `DslParser`, `DslTokenizer`, `ModuleMerger`
  - Metrics collection and export for bytecode analysis
- **`five-protocol`** (../five-protocol) - Protocol types, constants, and encoding utilities
  - Bytecode magic numbers, opcodes, type system, VLE encoding

**JavaScript/TypeScript Dependencies:**
- `@solana/web3.js` - Solana blockchain interaction
- `wasm-bindgen` - WASM JavaScript interop
- React and related UI libraries for deployment components
- Jest for TypeScript testing

Build dependencies like `wasm-pack` and Rust toolchain are documented in `build.sh`.