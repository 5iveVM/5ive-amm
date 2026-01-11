# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Five** is a blockchain virtual machine ecosystem for Solana, consisting of:
- **Five DSL** - A domain-specific language for writing smart contracts
- **Five VM (Mito)** - A stack-based virtual machine optimized for Solana execution
- **Five Protocol** - Shared protocol definitions, opcodes, and types
- **Five SDK** - TypeScript SDK for client-side interaction
- **Five CLI** - Command-line tools for compilation, deployment, and execution

The system compiles Five DSL source code (`.v` files) to compact bytecode that executes on-chain via the Five Solana program.

## Repository Structure

```
five-mono/
├── five-protocol/       # Shared protocol: opcodes, types, VLE encoding, headers
├── five-dsl-compiler/   # Rust compiler: DSL → bytecode
├── five-vm-mito/        # Core VM: zero-allocation Solana execution engine
├── five-solana/         # Solana program wrapper for on-chain execution
├── five-wasm/           # WASM bindings for browser/Node.js execution
├── five-sdk/            # TypeScript SDK (client-agnostic)
├── five-cli/            # CLI tools and test infrastructure
├── five-templates/      # Example contracts (counter, token, etc.)
└── third_party/         # Vendored dependencies (pinocchio fork)
```

## Essential Commands

### Building

```bash
# Build all Rust crates
cargo build --release

# Build specific crate
cargo build -p five-dsl-compiler
cargo build -p five-vm-mito
cargo build -p five --release  # Solana program (five-solana)

# Build WASM bindings
cd five-wasm && ./build.sh

# Build TypeScript SDK
cd five-sdk && npm run build

# Build CLI
cd five-cli && npm run build
```

### Testing

```bash
# Run all Rust tests
cargo test

# Test specific crate
cargo test -p five-protocol
cargo test -p five-dsl-compiler
cargo test -p five-vm-mito

# Run compiler tests with output
cargo test -p five-dsl-compiler -- --nocapture

# Run E2E template tests (requires localnet)
cd five-templates/counter && node e2e-counter-test.mjs
cd five-templates/token && node e2e-token-test.mjs

# Run CLI test suite
cd five-cli && npm run test:scripts
```

### Five CLI Usage

```bash
# Compile Five DSL to bytecode
five compile script.v
five compile script.v -o script.five

# Project-based workflow
five init my-project
five build
five deploy --project .
five execute --project .

# Local WASM execution
five local execute script.v 0

# On-chain execution
five deploy script.five
five execute <SCRIPT_ACCOUNT> -f 0 --params "[10, 20]"
```

## Architecture

### Compilation Pipeline

```
Five DSL (.v) → Tokenizer → Parser → Type Checker → Bytecode Generator → .fbin/.five
```

1. **Tokenization** - Lexical analysis into tokens
2. **Parsing** - Build Abstract Syntax Tree (AST)
3. **Type Checking** - Semantic analysis with cross-module symbol resolution
4. **Bytecode Generation** - Emit optimized bytecode with VLE encoding

### VM Execution Model

The Five VM is a **stack-based virtual machine** with:
- **64-byte temp buffer** for intermediate values
- **Zero-allocation design** for Solana compute efficiency
- **Lazy-loading** for account data (AccountRef pattern)
- **VLE encoding** reduces bytecode size by 30-50%

Key opcode categories (see `five-protocol/OPCODE_SPEC.md`):
- Control flow: `HALT`, `JUMP`, `JUMP_IF`, `REQUIRE`, `RETURN`
- Stack ops: `PUSH_U8/U16/U32/U64`, `POP`, `DUP`, `SWAP`
- Arithmetic: `ADD`, `SUB`, `MUL`, `DIV`, checked variants
- Memory: `LOAD_FIELD`, `STORE_FIELD`, `LOAD_FIELD_PUBKEY`
- Accounts: `GET_KEY`, `CHECK_SIGNER`, `CHECK_WRITABLE`, `TRANSFER`
- System: `INVOKE`, `DERIVE_PDA`, `INIT_ACCOUNT`

### On-Chain Execution Flow

```
Client → five-solana program → Five VM (Mito) → Account state changes
```

The `five-solana` crate wraps the VM and handles:
- Instruction parsing and dispatch
- Account constraint validation
- System program CPI for account creation

## Key Files by Component

### five-protocol
- `src/opcodes.rs` - All VM opcode definitions
- `src/types.rs` - Type constants and `ImportableAccountHeader`
- `src/encoding.rs` - VLE encoding/decoding
- `OPCODE_SPEC.md` - RFC-1 opcode specification

### five-dsl-compiler
- `src/compiler/pipeline.rs` - Unified compilation pipeline
- `src/parser/` - DSL parser (expressions, statements, blocks)
- `src/type_checker/` - Type validation and inference
- `src/bytecode_generator/` - Bytecode emission (modular by AST node type)
- `src/error/` - Structured error system with templates

### five-vm-mito
- `src/lib.rs` - VM entry point and execution loop
- `src/context.rs` - ExecutionContext and state management
- `src/handlers/` - Opcode handlers (memory, arithmetic, accounts, system)
- `src/utils.rs` - Stack operations and utilities

### five-solana
- `src/lib.rs` - Solana program entry point
- `src/instructions.rs` - Instruction parsing and dispatch

### five-sdk
- `src/FiveSDK.ts` - Main SDK class (compilation, execution, instruction generation)
- `src/encoding/ParameterEncoder.ts` - VLE parameter encoding
- `src/lib/vle-encoder.ts` - VLE utility implementation

## Five DSL Language

### Basic Syntax

```v
// Global state
mut counter: u64;

// Initialization block
init {
    counter = 0;
}

// Public function (callable on-chain)
pub increment() -> u64 {
    counter = counter + 1;
    return counter;
}

// Internal function
fn helper(x: u64) -> u64 {
    return x * 2;
}
```

### Account Constraints

```v
pub transfer(
    from: account @mut @signer,
    to: account @mut,
    amount: u64
) {
    // @mut = writable, @signer = must sign transaction
    // @init(payer=X, space=N) for account creation
}
```

### Module System

```v
use lib;                    // Import local module
use utils::helpers;         // Nested import
use "PubkeyAddress"::{fn};  // External contract import

pub main() {
    lib::calculate(10);     // Qualified function call
}
```

## Development Guidelines

### Adding New Opcodes

1. Define opcode constant in `five-protocol/src/opcodes.rs`
2. Add handler in `five-vm-mito/src/handlers/`
3. Update compiler emission in `five-dsl-compiler/src/bytecode_generator/`
4. Add tests in both crates
5. Update `OPCODE_SPEC.md`

### Modifying Bytecode Generation

- AST generation is modular: `bytecode_generator/ast_generator/*.rs`
- Each AST node type has its own module (expressions, statements, functions)
- Test with `cargo test --test golden_bytecode` for regressions

### Error Handling

- Compiler errors go through `five-dsl-compiler/src/error/`
- Use error codes from `error/registry.rs`
- VM errors use `five_protocol::VMError`

### Testing Workflow

1. Write Five DSL test script in `five-templates/` or `five-cli/test-scripts/`
2. Add `// @test-params X Y` comments for parameterized tests
3. Run locally with WASM: `five local execute script.v 0`
4. Test on-chain with localnet after `solana-test-validator`

### SDK Usage with Parameter Encoding

When using `FiveSDK.generateExecuteInstruction()` with functions that have mixed account/data parameters:

```javascript
import { FiveSDK } from 'five-sdk';

// Load the ABI from compiled .five file
const fiveFile = JSON.parse(fs.readFileSync('build/contract.five', 'utf-8'));
const abi = fiveFile.abi;

// Get function definition to determine parameter order
const functionDef = abi.functions.find(f => f.name === 'myFunction');

// Build merged parameters array in correct order (accounts and data mixed per ABI)
const mergedParams = [];
functionDef.parameters.forEach(param => {
  if (param.is_account || param.isAccount) {
    mergedParams.push(accountPublicKey);  // Account parameter
  } else {
    mergedParams.push(dataValue);          // Data parameter (u64, pubkey, string, etc.)
  }
});

// Generate instruction with ABI metadata
const instruction = await FiveSDK.generateExecuteInstruction(
  scriptAccountPubkey,
  functionIndex,
  mergedParams,         // All parameters in correct order
  accountPubkeys,       // Also pass account list
  connection,
  {
    scriptMetadata: abi,  // IMPORTANT: Pass ABI for proper parameter mapping
    vmStateAccount: vmStatePda,
    fiveVMProgramId: programId,
    adminAccount: payerPubkey
  }
);
```

**Key Points:**
- Always pass `scriptMetadata: abi` in options
- Merge account and data parameters in correct order from function definition
- The SDK will identify accounts via ABI and map them to indices
- All parameters are encoded via WASM encoder for reliability

## Current Status

### Working
- Full compilation pipeline
- Local WASM execution
- Basic on-chain deployment and execution
- **SDK parameter encoding for mixed-type functions (FIXED)**

### SDK Parameter Encoding Fix (COMPLETED)

The Five SDK parameter encoding issue for functions with mixed account/data parameters has been resolved:

**Root Cause:** Test was not passing ABI metadata to SDK, which prevented proper account parameter mapping

**Solution Applied:**
1. Pass `scriptMetadata: counterABI` in options to `FiveSDK.generateExecuteInstruction()`
2. Merge account and data parameters in correct order based on ABI function definition:
   ```javascript
   // Example: for increment(counter, owner) function
   const mergedParameters = [counterAccount, ownerAccount];  // All in order
   ```
3. SDK now correctly:
   - Identifies which parameters are accounts via ABI
   - Maps account pubkeys to indices
   - Encodes all parameters via WASM encoder

**Files Modified:**
- `five-templates/counter/e2e-counter-test.mjs` - Pass ABI to SDK, merge parameters by ABI order

**Test Results:** Parameters now encode with 77 bytes (previously 0), function calls reach VM bytecode execution

### Known Issues

None currently known. The following were previously issues but have been resolved:
- ✅ **SDK parameter encoding** - Fixed by passing `scriptMetadata` ABI to SDK
- ✅ **@init constraint** - Works correctly (counter template demonstrates this)
- ✅ **Token template** - Was blocked by string parameter handling in DSL, not @init

**Note on @init:** The `@init(payer=X, space=N)` constraint works correctly in Five and is demonstrated in the counter template. Previous token template issues were related to how the compiler handled string parameters (URI field), not account initialization itself.

See `HANDOFF.md` for detailed current status and next steps.

## Deployment

### Local Development

```bash
# Start Solana localnet
solana-test-validator

# Deploy Five VM program
cd five-solana
cargo build --release
solana program deploy target/deploy/five.so --url http://127.0.0.1:8899
```

### Program IDs
- Five VM Program: `HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg` (localnet)
