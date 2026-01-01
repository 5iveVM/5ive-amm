# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Five SDK** is a client-agnostic TypeScript SDK for Five VM scripts on Solana. It provides a **zero-dependency, pure serialization interface** for interacting with Five scripts without requiring any specific Solana client library. The SDK focuses on **compilation, parameter encoding, and instruction generation** while maintaining full compatibility with any Solana client implementation.

**Key Facts:**
- **Package Name:** `five-sdk` v1.1.2
- **Type:** ESM Module with zero production dependencies
- **Target:** ES2020+ JavaScript
- **Entry Points:** dist/index.js, dist/index.d.ts
- **Production-Ready:** Real Five VM integration with WASM compilation and execution

## Architecture

### Core Design Principles

1. **Client-Agnostic**: Zero `@solana/web3.js` dependencies - works with any Solana client library (web3.js, Anchor, Metaplex, custom RPC clients, etc.)
2. **Static Method Design**: Primary functionality exposed through static methods for easy integration without instance creation
3. **Real Compilation**: Uses actual WASM-based Five VM compiler, no placeholder implementations
4. **VLE Encoding**: Implements Variable Length Encoding for optimal instruction compression (30-50% size reduction)
5. **Local Execution**: Full WASM VM for local testing and instant feedback without blockchain

### Key Components

- **`FiveSDK.ts` (4,313 lines)** - Main SDK class with static compilation, instruction generation, and local execution methods
- **`compiler/`** - Bytecode compilation using WASM infrastructure
- **`encoding/ParameterEncoder.ts`** - VLE parameter encoding with automatic type coercion based on ABI
- **`crypto/`** - PDA utilities, Base58 handling, rent calculation
- **`validation/InputValidator.ts`** - Input validation with structured FiveSDKError reporting
- **`testing/`** - Programmatic test runners, account fixtures, and state serialization
- **`wasm/`** - Cross-platform WASM module loading for compiler and VM integration
- **`types.ts` (408+ lines)** - Comprehensive type definitions with correct Five ecosystem terminology
- **`metadata/`** - Script metadata parsing and caching for performance
- **`lib/vle-encoder.ts`** - Variable Length Encoding utility for optimal parameter compression

### Error Handling Architecture

- `FiveSDKError` - Base error class with code and details
- `CompilationSDKError` - Compilation-specific errors with bytecode context
- `ExecutionSDKError` - Execution-specific errors with VM output
- `ParameterEncodingError` - Type coercion and encoding errors
- All errors inherit structured error reporting with rich context

## Essential Commands

### Development Workflow
```bash
# Build the SDK
npm run build                    # Full TypeScript compilation + asset copying to dist/

# Development mode
tsc --watch                     # Watch TypeScript compilation

# Testing
npm test                        # Runs examples/basic-usage.js as test suite
node examples/basic-usage.ts    # Run comprehensive usage examples
```

### Build Details
- **Compilation:** `tsc` compiles src/ to dist/
- **Asset Copying:** WASM binary (`five_vm_wasm_bg.wasm`) copied from src/assets/wasm/ to dist/assets/vm/
- **Distribution:** npm includes dist/, README.md, and LICENSE

## SDK Usage Patterns

### Basic Setup
```typescript
import { FiveSDK } from '@five-vm/sdk';

// Factory methods for common networks
const devnetSDK = FiveSDK.devnet();
const mainnetSDK = FiveSDK.mainnet();
const localSDK = FiveSDK.localnet();

// Custom configuration
const sdk = new FiveSDK({
  fiveVMProgramId: '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo',
  debug: true,
  network: 'devnet'
});
```

### Compilation (Static Methods)
```typescript
// Compile Five source to bytecode
const compilation = await FiveSDK.compile(scriptSource, {
  optimize: true,
  debug: false
});

// Check compilation result
if (compilation.success) {
  console.log(`Bytecode: ${compilation.bytecode?.length} bytes`);
  console.log(`Functions: ${compilation.metadata?.functions.length}`);
} else {
  compilation.errors?.forEach(error => {
    console.log(`${error.severity}: ${error.message} (line ${error.line})`);
  });
}
```

### Local WASM Execution
```typescript
// One-step: compile and execute locally
const result = await compileAndExecuteLocally(
  scriptSource,
  'functionName',
  [param1, param2],
  { debug: true, trace: true }
);

// Two-step: separate compilation and execution
const compilation = await FiveSDK.compile(scriptSource);
const result = await FiveSDK.executeLocally(
  compilation.bytecode,
  'functionName',
  [param1, param2],
  { debug: true }
);
```

### Instruction Generation (Client-Agnostic)
```typescript
// Generate deployment instruction data
const deployData = await FiveSDK.generateDeployInstruction(
  bytecode,
  deployerAddress,
  { debug: true }
);

// Generate execution instruction data with VLE-encoded parameters
const executeData = await FiveSDK.generateExecuteInstruction(
  scriptAccount,
  'functionName',
  [param1, param2],
  [], // additional accounts
  undefined,
  { debug: true, computeUnitLimit: 50000 }
);

// Use with any Solana client library - SDK generates compatible instruction data
const instruction = {
  programId: FIVE_VM_PROGRAM_ID,
  data: Buffer.from(executeData.instruction.data, 'base64'),
  accounts: executeData.instruction.accounts
};
```

## Key Features

### VLE (Variable Length Encoding)
- Function indices and parameters are VLE-encoded for optimal compression
- Type coercion automatically handled based on ABI information from compilation
- Reduces instruction data size by 30-50% compared to standard encoding
- Results in lower Solana transaction costs and faster processing

### Parameter Type Coercion
The SDK automatically coerces parameters to correct types based on compiled ABI:
```typescript
// SDK infers types (u32, bool, string) from ABI
const result = await FiveSDK.executeLocally(
  bytecode,
  'processData',
  [42, true, "test"],
  { debug: true }
);
```

### Test Runner Integration
The SDK includes a programmatic test runner (`testing/TestRunner.ts`) supporting:
- Structured test case definitions with expected results
- Parallel test execution capabilities
- Comprehensive result reporting with compute unit tracking
- Account fixture management and state serialization
- Integration with Five script test infrastructure

### Terminology Alignment
The SDK uses correct Five ecosystem terminology (not Solana-centric):
- **Scripts** (not contracts) - Five source code files
- **Script Accounts** (not program IDs) - On-chain storage for compiled bytecode
- **Bytecode** - Compiled Five VM instructions
- **Five VM Program** - The Solana program that executes Five scripts

## File Organization

### Root Directory Structure
```
src/
├── FiveSDK.ts                    # Main SDK class (4,313 lines)
├── index.ts                      # Public exports
├── types.ts                      # Type definitions (408+ lines)
├── compiler/BytecodeCompiler.ts  # WASM compilation interface
├── encoding/ParameterEncoder.ts  # VLE encoding + type coercion
├── crypto/index.ts               # PDA utilities, Base58, rent calculation
├── validation/InputValidator.ts  # Input validation + error reporting
├── testing/                      # Test runners, fixtures, account management
├── wasm/                         # WASM loader (cross-platform support)
├── metadata/index.ts             # Metadata parsing + caching
├── lib/vle-encoder.ts            # Variable Length Encoding utility
├── config/ConfigManager.ts       # Configuration management
├── project/                      # TOML parsing, project config
├── accounts/index.ts             # Account handling utilities
├── utils/abi.ts                  # ABI normalization
├── logging/index.ts              # Logging infrastructure
├── examples/basic-usage.ts       # Comprehensive usage examples
├── assets/wasm/                  # WASM binaries (five_vm_wasm_bg.wasm)
└── __tests__/                    # Unit and integration tests
```

## Common Development Patterns

### Error Handling
```typescript
import { FiveSDK, CompilationSDKError } from '@five-vm/sdk';

try {
  const compilation = await FiveSDK.compile(source);
  if (!compilation.success) {
    compilation.errors?.forEach(error => {
      console.log(`${error.severity}: ${error.message} (line ${error.line})`);
    });
  }
} catch (error) {
  if (error instanceof CompilationSDKError) {
    // Handle compilation-specific errors
  }
}
```

### Client Integration Example
Works with any Solana client library (web3.js, Anchor, Metaplex, custom):
```typescript
import { Connection, Transaction } from '@solana/web3.js';
import { FiveSDK } from '@five-vm/sdk';

const connection = new Connection('https://api.devnet.solana.com');
const deployData = await FiveSDK.generateDeployInstruction(bytecode, signer.publicKey);

const transaction = new Transaction().add({
  programId: deployData.instruction.programId,
  data: Buffer.from(deployData.instruction.data, 'base64'),
  keys: deployData.instruction.accounts
});

await connection.sendTransaction(transaction, [signer]);
```

## Performance Considerations

### Local vs On-Chain Execution
- **Local WASM execution**: Instant feedback, perfect for development and rapid iteration
- **On-chain execution**: Real Solana environment with actual compute unit costs
- Use local execution during development; validate on-chain before production

### VLE Encoding Benefits
- Reduces instruction data size by 30-50% compared to standard encoding
- Lower transaction costs on Solana due to smaller data payloads
- Faster transaction processing and propagation

### Build Optimization
- SDK compiles to ESNext modules for optimal tree-shaking
- Zero production dependencies minimize bundle size
- TypeScript declarations provide full IntelliSense support
- Metadata caching prevents re-parsing compiled information

## WASM Integration

The SDK includes custom WASM loader (`src/wasm/loader.ts`) for cross-platform support:
- Loads `five_vm_wasm_bg.wasm` (Five VM binary) at runtime
- Supports both development and production modes
- Compatible with browser and Node.js environments
- WASM binaries copied to `dist/assets/vm/` during build

## Testing

**Testing Infrastructure:**
- Test files located in `src/__tests__/` (unit and integration tests)
- Test runner: `npm test` executes `examples/basic-usage.js`
- TestRunner class supports programmatic test execution
- AccountTestFixture provides test account state management
- No formal test framework configured (can integrate Vitest/Jest as needed)

**Running Tests:**
```bash
npm test                        # Basic tests via examples
node examples/basic-usage.js    # Detailed examples with output
```

## Module Dependencies

### Production Dependencies
- **None** - True zero-dependency design for maximum compatibility

### Peer Dependencies
- `@solana/web3.js ^1.90.0` - Optional for Solana integration (not required for SDK usage)

### Dev Dependencies
- TypeScript 5.3.3
- @types/node 20.11.16

## Important Implementation Notes

1. **Static Methods Over Instances**: Most SDK functionality is accessed via static methods (`FiveSDK.compile()`, `FiveSDK.executeLocally()`, etc.) rather than instance methods. This simplifies usage and doesn't require SDK instantiation for common tasks.

2. **Real WASM Compilation**: The SDK uses the actual Five VM compiler via WASM, not placeholder implementations. This ensures bytecode compatibility with on-chain execution.

3. **Configuration Flexibility**: The SDK supports custom Five VM Program IDs via ConfigManager for networks beyond devnet/mainnet/testnet.

4. **Metadata Caching**: Compiled metadata is cached (in MetadataCache) to avoid re-parsing the same script multiple times, improving performance in batch operations.

5. **Type-Safe Execution**: Parameter encoding is fully type-safe with automatic coercion based on compiled ABI information, reducing runtime errors.
