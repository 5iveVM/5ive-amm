# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Five SDK** is a client-agnostic TypeScript SDK for Five VM scripts on Solana. It provides a **zero-dependency, pure serialization interface** for interacting with Five scripts without requiring any specific Solana client library. The SDK focuses on **compilation, parameter encoding, and instruction generation** while maintaining full compatibility with any Solana client implementation.

This is a **production-ready SDK** with real Five VM integration, VLE (Variable Length Encoding) support, and comprehensive local WASM execution capabilities.

## Architecture

### Core Design Principles

- **Client-Agnostic**: Zero `@solana/web3.js` dependencies - works with any Solana client library
- **Static Method Design**: Primary functionality exposed through static methods for easy integration  
- **Real Compilation**: Uses actual WASM-based Five VM compiler, no placeholder implementations
- **VLE Encoding**: Implements Variable Length Encoding for optimal instruction compression
- **Local Execution**: Full WASM VM for local testing without blockchain connectivity

### Key Components

- **`FiveSDK.ts`** - Main SDK class with static compilation and instruction generation methods
- **`compiler/`** - Bytecode compilation using WASM infrastructure
- **`encoding/`** - VLE parameter encoding and type coercion
- **`crypto/`** - PDA utilities, Base58 handling, rent calculation
- **`validation/`** - Input validation and error handling
- **`testing/`** - Programmatic test runner for Five VM scripts
- **`types.ts`** - Complete type definitions with correct Five ecosystem terminology

## Essential Commands

### Development Workflow
```bash
# Build the SDK
npm run build                    # Compile TypeScript to dist/

# Run examples and tests  
npm test                        # Run basic usage examples
node examples/basic-usage.js    # Run comprehensive examples

# Development commands
tsc                             # TypeScript compilation
tsc --watch                     # Watch mode for development
```

### SDK Usage Patterns

#### Basic Setup
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

#### Compilation (Static Methods)
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
}
```

#### Local WASM Execution
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

#### Instruction Generation (Client-Agnostic)
```typescript
// Generate deployment instruction data
const deployData = await FiveSDK.generateDeployInstruction(
  bytecode,
  deployerAddress,
  { debug: true }
);

// Generate execution instruction data  
const executeData = await FiveSDK.generateExecuteInstruction(
  scriptAccount,
  'functionName',
  [param1, param2],
  [], // additional accounts
  undefined,
  { debug: true, computeUnitLimit: 50000 }
);

// Use with any Solana client library
const instruction = {
  programId: FIVE_VM_PROGRAM_ID,
  data: Buffer.from(executeData.instruction.data, 'base64'),
  accounts: executeData.instruction.accounts
};
```

## Key Features

### VLE (Variable Length Encoding)
- Function indices and parameters are VLE-encoded for optimal compression
- Type coercion based on ABI information from compilation
- Automatic parameter validation and encoding
- Integration with existing VLE encoder infrastructure

### Test Runner Integration
The SDK includes a programmatic test runner (`testing/TestRunner.ts`) that provides:
- Structured test case definitions with expected results
- Parallel test execution capabilities
- Comprehensive result reporting with compute unit tracking
- Integration with Five script test infrastructure

### Terminology Alignment
The SDK uses correct Five ecosystem terminology:
- **Scripts** (not contracts) - Five source code files (.v)
- **Script Accounts** (not program IDs) - On-chain storage for bytecode
- **Bytecode** - Compiled Five VM instructions
- **Five VM Program** - The Solana program that executes Five scripts

## File Organization

### Core Files
- **`index.ts`** - Main exports and convenience functions
- **`FiveSDK.ts`** - Primary SDK class with static methods
- **`types.ts`** - Complete type definitions (150+ lines)

### Specialized Modules
- **`compiler/BytecodeCompiler.ts`** - WASM-based compilation interface
- **`encoding/ParameterEncoder.ts`** - VLE parameter encoding with type validation
- **`crypto/index.ts`** - Cryptographic utilities for PDA and Base58 operations
- **`validation/InputValidator.ts`** - Input validation with structured error reporting

### Testing and Examples
- **`examples/basic-usage.ts`** - Comprehensive SDK usage examples (470+ lines)
- **`testing/TestRunner.ts`** - Programmatic test execution framework
- **`__tests__/`** - Unit and integration tests organized by module

## Common Development Patterns

### Error Handling
```typescript
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

### Parameter Type Coercion
```typescript
// SDK automatically handles type coercion based on ABI
const result = await FiveSDK.executeLocally(
  bytecode,
  'processData',
  [42, true, "test"], // u32, bool, string - types inferred from ABI
  { debug: true }
);
```

### Client Integration
```typescript
// Works with any Solana client library
import { Connection, Transaction } from '@solana/web3.js';
import { FiveSDK } from '@five-vm/sdk';

const deployData = await FiveSDK.generateDeployInstruction(bytecode, signer.publicKey);
const transaction = new Transaction().add({
  programId: deployData.instruction.programId,
  data: Buffer.from(deployData.instruction.data, 'base64'),
  keys: deployData.instruction.accounts
});
```

## Performance Considerations

### Local vs On-Chain Execution
- **Local WASM execution**: Instant feedback, perfect for development and testing
- **On-chain execution**: Real Solana environment with actual compute unit costs
- Use local execution for rapid iteration, on-chain for final validation

### VLE Encoding Benefits
- Reduces instruction data size by 30-50% compared to standard encoding
- Lower transaction costs on Solana
- Faster transaction processing due to smaller data payloads

### Build Optimization
- SDK compiles to ESNext modules for optimal tree-shaking
- Zero runtime dependencies reduce bundle size
- TypeScript declarations provide full IntelliSense support
