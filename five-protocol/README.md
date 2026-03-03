# five-protocol

**Shared protocol definitions for the Five blockchain virtual machine ecosystem.**

## Overview

`five-protocol` is the foundational crate that defines the common protocol, types, and utilities shared between the Five DSL compiler (`five-dsl-compiler`) and the Five VM (`five-vm-mito`). It ensures consistency and compatibility across the entire Five toolchain.

## Architecture

```
┌─────────────────────┐
│  five-dsl-compiler  │  (Compiles Five DSL → Bytecode)
└──────────┬──────────┘
           │
           │ uses
           ▼
    ┌──────────────┐
    │five-protocol │  ◀── Shared protocol layer
    └──────┬───────┘
           │ uses
           ▼
┌─────────────────────┐
│   five-vm-mito      │  (Executes bytecode)
└─────────────────────┘
```

## Core Components

### 1. Opcodes (`opcodes.rs`)
Defines all VM operation codes used for bytecode execution:
- **Stack Operations**: PUSH, POP, DUP, SWAP
- **Arithmetic**: ADD, SUB, MUL, DIV, MOD
- **Logical**: AND, OR, XOR, NOT
- **Control Flow**: JUMP, JUMP_IF_TRUE, JUMP_IF_FALSE, CALL, RETURN
- **Memory**: LOAD, STORE, LOAD_LOCAL, STORE_LOCAL
- **Account Operations**: LOAD_ACCOUNT, STORE_ACCOUNT, LOAD_FIELD
- **System Calls**: CREATE_PDA, INVOKE_PROGRAM, GET_CLOCK
- **Advanced**: POLY_ADD, POLY_MUL, OPTION_UNWRAP, RESULT_UNWRAP

100+ opcodes total, all with comprehensive documentation.

### 2. Types (`types.rs`)
Common type definitions and constants:
- Type constants: `U8`, `U16`, `U32`, `U64`, `U128`, `I64`, `BOOL`, `STRING`, `PUBKEY`, `ACCOUNT`, `ARRAY`
- `ImportableAccountHeader` - Structure for account imports
- Type validation and conversion utilities

### 3. Value System (`value.rs`)
Runtime value representation and operations:
- `Value` enum supporting all Five types
- Value conversion (`to_u64()`, `to_bool()`, `to_string()`)
- Type checking and validation
- Arithmetic and logical operations on values

### 4. Headers (`lib.rs`)
Bytecode header structures for program metadata:
- **`ScriptBytecodeHeaderV1`** (10 bytes) - Locked v1 bytecode header format
  - Magic bytes: `5IVE` (4 bytes)
  - Features: Optimization flags (`u32`, 4 bytes)
  - Function counts: public + total (2 bytes)
- **`OptimizedHeader`** - Deprecated compatibility alias for `ScriptBytecodeHeaderV1`
- **`FIVEScriptHeaderV2/V3`** - Legacy compatibility headers with extended metadata
- Header validation and parsing logic

### 5. Call Convention (`call_convention.rs`)
Function calling protocol and parameter passing:
- Parameter marshalling and unmarshalling
- Return value handling
- Stack frame management conventions
- Register usage conventions

### 6. Transport (`transport.rs`)
Cross-program invocation (CPI) utilities:
- Program interface definitions
- Cross-program call structures
- Account and data serialization for CPI
- Error handling for cross-program calls

## Usage

### In Compiler (five-dsl-compiler)

```rust
use five_protocol::{opcodes::*, ScriptBytecodeHeaderV1, Value};

// Emit opcodes
bytecode.push(PUSH);
bytecode.push(42);

// Emit header
let header = ScriptBytecodeHeaderV1 {
    magic: [b'5', b'I', b'V', b'E'],
    features: FEATURE_FUSED_BRANCH | FEATURE_COLD_START_OPT,
    public_function_count: 2,
    total_function_count: 5,
};

// Encode fixed-width values
bytecode.extend_from_slice(&address.to_le_bytes());
```

### In VM (five-vm-mito)

```rust
use five_protocol::{
    opcodes::*,
    ScriptBytecodeHeaderV1,
    Value,
};

// Parse header
let (header, code_start) = five_protocol::parse_header(&bytecode)?;

// Execute opcodes
match opcode {
    PUSH => stack.push(Value::U64(operand)),
    ADD => {
        let b = stack.pop()?;
        let a = stack.pop()?;
        stack.push(a + b);
    }
    // ...
}
```

## Design Principles

1. **Zero-Copy Where Possible**: Headers and structures are `repr(C, packed)` for direct memory mapping
2. **Const Functions**: Many functions are `const fn` for compile-time evaluation
3. **No Allocations**: Core protocol types avoid heap allocations for performance
4. **Backward Compatibility**: Keeps compatibility aliases while the v1 header naming becomes canonical
5. **Comprehensive Coverage**: Defines everything needed for bytecode generation and execution

## Feature Flags

```toml
[features]
default = []
serde = ["dep:serde"]  # Enable serde serialization support
debug-logs = []        # Optional debug logging integration hook
test-fixtures = []     # Test-only payload fixtures
```

## Versioning

five-protocol uses semantic versioning. Breaking changes to opcodes or protocol structures will result in a major version bump to ensure compatibility across the toolchain.

### Current Version
- **Opcode Set**: Stable v1.0
- **Header Format**: ScriptBytecodeHeaderV1 (production)
- **Encoding**: Fixed-width immediates

## Testing

Run the test suite:
```bash
cargo test -p five-protocol
```

## Dependencies

Minimal dependencies for portability:
- `serde` - Serialization framework (optional)

## Performance

- **Header Size**: 10 bytes (`ScriptBytecodeHeaderV1`) vs 23+ bytes (legacy versioned headers)
- **Encoding**: Fixed-width immediates for predictable decoding
- **Parse Speed**: Zero-copy header parsing in <10ns
- **Validation**: Const-time opcode validation

## Contributing

When adding new opcodes or protocol features:
1. Update `opcodes.rs` with new opcode constants
2. Add corresponding documentation
3. Update both compiler and VM to support the new opcode
4. Add tests for the new functionality
5. Update this README

## License

MIT License - See LICENSE file for details
