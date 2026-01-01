# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Five CLI** is a high-performance command-line interface for Five VM development, featuring WebAssembly integration and comprehensive Solana blockchain support. The CLI enables developers to compile, test, deploy, and execute Five VM scripts with both local WASM execution and on-chain Solana deployment capabilities.

This is a **production-ready TypeScript CLI** with real Five VM integration, VLE (Variable Length Encoding) compression, and comprehensive testing infrastructure. All operations work with actual Five VM bytecode - no placeholder implementations.

## Architecture

### Core Components

- **`src/commands/`** - CLI command implementations (compile, execute, deploy, test)
- **`src/sdk/`** - Five SDK for VM integration and Solana operations
- **`src/config/`** - Configuration management system
- **`src/utils/`** - File management and logging utilities
- **`assets/vm/`** - WebAssembly bindings for Five VM
- **`test-scripts/`** - Comprehensive test suite organized by categories

### Key Design Principles

- **Hybrid execution**: Supports both local WASM execution and on-chain Solana deployment
- **VLE encoding**: Uses Variable Length Encoding for optimal instruction compression
- **Client-agnostic SDK**: Generates serialized transaction data for any Solana client library
- **Real bytecode execution**: All operations work with actual Five VM bytecode
- **Global state support**: Five VM supports global variables with proper initialization patterns

## Essential Commands

### Development Workflow
```bash
# Build the CLI
npm run build                    # Full build (WASM + TypeScript)
npm run build:js:dev            # TypeScript build with watch mode
npm run build:wasm              # Build Five VM WASM bindings

# Run tests
npm run test:scripts             # Run Five script test suite
npm run test:scripts:verbose     # Verbose test output with debug info
npm run test:scripts:errors      # Show only errors
./test-runner.sh --onchain --network localnet --category 01-language-basics --verbose

# Development commands
npm run lint                     # ESLint checking
npm run format                  # Code formatting
npm run dev                      # TypeScript development mode
```

### Five CLI Usage
```bash
# Compilation
five compile script.v            # Compile Five source to bytecode
five compile script.v --output script.five  # Compile with specific output
five compile --project .         # Use five.toml (entry_point/source_dir/build_dir/output name)
five compile --project path/to/five.toml   # Explicit config path

# Local execution (WASM)
five local execute script.v 0    # Execute function 0 locally
five execute script.five --local # Execute compiled bytecode locally

# On-chain operations
five deploy script.five          # Deploy to Solana
five execute <SCRIPT_ACCOUNT> -f 0 --params "[10, 20]"  # Execute on-chain
five deploy-and-execute script.v --function 0 --params "[10, 20]"  # Deploy and execute
five deploy --project .          # Use manifest/five.toml defaults (cluster/rpc/keypair/program_id)
five execute --project .         # Use manifest/five.toml defaults (cluster/rpc/keypair)

# Configuration
five config set target localnet  # Set target network
five config get                  # Show current configuration
```

### Test Runner Usage
```bash
# Local WASM testing (default)
./test-runner.sh                 # Run all tests locally
./test-runner.sh --category 01-language-basics  # Run specific category
./test-runner.sh --verbose       # Show detailed output

# On-chain testing
./test-runner.sh --onchain --network localnet  # Test on-chain execution
./test-runner.sh --onchain --verbose --category 01-language-basics

# Test parameters are read from @test-params comments in .v files
# Example: // @test-params 10 20
```

## Testing Infrastructure

### Test Organization
- **`01-language-basics/`** - Basic language features (add, multiply, functions)
- **`02-operators-expressions/`** - Arithmetic and logical operations  
- **`03-control-flow/`** - Conditionals and loops
- **`04-account-system/`** - Account constraints and state management
- **`05-blockchain-integration/`** - PDA operations and Solana features
- **`06-advanced-features/`** - Arrays, strings, complex operations
- **`07-error-system/`** - Error handling and validation
- **`08-match-expressions/`** - Pattern matching and Result/Option types

### Test Script Format
Five test scripts use `.v` extension and include test parameters:
```v
// @test-params 10 20
pub add(a: u64, b: u64) -> u64 {
    return a + b;
}
```

### VLE Encoding
The CLI uses Variable Length Encoding for instruction compression:
- Function indices and parameters are VLE-encoded
- Instruction format: `[discriminator, vle_function_index, vle_parameters]`
- Working examples show proper VLE encoding: `[2, 0, 2, 30, 40]` for function 0 with params [30, 40]

## Key Files

### CLI Entry Points
- **`src/index.ts`** - Main CLI application with command registration
- **`src/commands/index.ts`** - Command registry and discovery system

### Core SDK
- **`src/sdk/FiveSDK.ts`** - Main SDK class for Five VM operations
- **`src/sdk/encoding/ParameterEncoder.ts`** - VLE parameter encoding
- **`src/lib/vle-encoder.ts`** - Variable Length Encoding implementation

### Configuration
- **`src/config/ConfigManager.ts`** - Network and keypair configuration management
- **`five.toml`** - Project configuration format (for `five init`); compile/deploy/execute/test consume this when `--project` is set or found via discovery
- **`.five/build.json`** - Build manifest emitted by `five compile` when a project is loaded (records artifact path, format, source files, hash, target)
  - Discovery order: `--project` dir/file > nearest five.toml upward from cwd.
  - Artifact preference: `.five` (ABI + bytecode) preferred over `.bin`; manifest records format.

### Utilities
- **`src/utils/FiveFileManager.ts`** - Handles .bin, .five, and .v file formats
- **`test-runner.sh`** - Comprehensive test script with on-chain and local modes

## Development Guidelines

### Command Implementation
- All commands extend `CommandDefinition` interface
- Support both local WASM and on-chain Solana execution modes
- Use configuration system for network/keypair management
- Include comprehensive help text and examples

### Error Handling
- Use structured error types from Five VM
- Provide clear error messages with context
- Support debug mode for detailed execution information

### Five VM Language Features

#### Global Variables and State
Five VM supports global variables with proper initialization:
```v
    // Global state variables  
    mut counter: u64;
    mut total_value: u64;
    
    // Constructor for global initialization
    init {
        counter = 0;
        total_value = 100;
    }
    
    // Functions can read and modify globals
pub increment() -> u64 {
        counter = counter + 1;
        return counter;
    }
```

#### Function Definitions
- Use `pub` for public functions accessible from outside
- Private functions can be called internally
- Support for parameters, return values, and local variables

### File Formats
- **`.v`** - Five source code
- **`.five`** - Compiled bytecode with ABI (JSON format)
- **`.bin`** - Raw bytecode (legacy format)
- **`.fbin`** - Five binary format with metadata

### Testing Requirements
- All new features must have corresponding test scripts
- Tests should cover both local WASM and on-chain execution
- Use `@test-params` comments for parameterized tests
- Verify VLE encoding works correctly for parameter passing

## Common Issues

### Configuration Problems
- Ensure `~/.config/solana/id.json` exists for on-chain operations
- Use `five config set target localnet` to configure local testing
- Check network connectivity for on-chain operations

### VLE Encoding Issues
- Parameters must be passed as JSON arrays: `"[10, 20]"`
- The test runner converts space-separated params to JSON format
- Debug mode shows VLE encoding details for troubleshooting

### Build Issues
- Run `npm run build:wasm` if WASM bindings are missing
- Use `npm run copy:wasm-assets` to ensure assets are in correct locations
- Check Node.js version (requires >=18.0.0)
