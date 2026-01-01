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
- **`OptimizedHeader`** (7 bytes) - Production header format
  - Magic bytes: `5IVE` (4 bytes)
  - Features: Optimization flags (1 byte)
  - Function counts: public + total (2 bytes)
- **`ScriptHeader`** - Legacy header format
- **`FIVEScriptHeaderV2/V3`** - Versioned headers with extended metadata
- Header validation and parsing logic

### 5. Encoding (`encoding.rs`)
Variable-Length Encoding (VLE) for compact bytecode:
- `VLE::encode_u32()` - Encode u32 in 1-5 bytes
- `VLE::decode_u32()` - Decode VLE-encoded u32
- LEB128-style encoding for space efficiency
- Reduces bytecode size by ~30-40% for typical programs

### 6. Call Convention (`call_convention.rs`)
Function calling protocol and parameter passing:
- Parameter marshalling and unmarshalling
- Return value handling
- Stack frame management conventions
- Register usage conventions

### 7. Transport (`transport.rs`)
Cross-program invocation (CPI) utilities:
- Program interface definitions
- Cross-program call structures
- Account and data serialization for CPI
- Error handling for cross-program calls

## Usage

### In Compiler (five-dsl-compiler)

```rust
use five_protocol::{
    opcodes::*,
    OptimizedHeader,
    VLE,
    Value,
};

// Emit opcodes
bytecode.push(PUSH);
bytecode.push(42);

// Emit header
let header = OptimizedHeader {
    magic: [b'5', b'I', b'V', b'E'],
    features: FEATURE_FUSED_BRANCH | FEATURE_COLD_START_OPT,
    public_function_count: 2,
    total_function_count: 5,
};

// Use VLE encoding
let (len, bytes) = VLE::encode_u32(address);
bytecode.extend_from_slice(&bytes[..len]);
```

### In VM (five-vm-mito)

```rust
use five_protocol::{
    opcodes::*,
    OptimizedHeader,
    Value,
};

// Parse header
let header = OptimizedHeader::parse(&bytecode)?;

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
4. **Backward Compatibility**: Supports multiple header versions for seamless upgrades
5. **Comprehensive Coverage**: Defines everything needed for bytecode generation and execution

## Feature Flags

```toml
[features]
default = []
std = []          # Enable std library (off by default for no_std compatibility)
serde = ["dep:serde"]  # Enable serde serialization support
```

## Versioning

five-protocol uses semantic versioning. Breaking changes to opcodes or protocol structures will result in a major version bump to ensure compatibility across the toolchain.

### Current Version
- **Opcode Set**: Stable v1.0
- **Header Format**: OptimizedHeader v2 (production)
- **VLE Encoding**: LEB128-compatible v1.0

## Testing

Run the test suite:
```bash
cargo test -p five-protocol
```

## Dependencies

Minimal dependencies for portability:
- `borsh` - Binary serialization (optional)
- `serde` - Serialization framework (optional)
- `solana-program` - Solana SDK types (for Pubkey, etc.)

## Performance

- **Header Size**: 7 bytes (OptimizedHeader) vs 115 bytes (legacy) - 94% smaller
- **VLE Encoding**: 1-5 bytes per u32 vs fixed 4 bytes - ~30-40% size reduction
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
