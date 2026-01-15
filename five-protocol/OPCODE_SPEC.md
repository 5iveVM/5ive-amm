---
rfc: 1
title: Five VM Opcode Specification
author: Five Team
status: Living
type: Standards Track
created: 2024-08-01
---

# RFC-1: Five VM Opcode Specification

## Abstract

This document defines the instruction set architecture (ISA) for the Five Virtual Machine (VM). It specifies the opcodes, their binary encoding, argument types, stack effects, and operational semantics. The Five VM is a stack-based virtual machine optimized for the Solana blockchain, focusing on efficiency, compactness, and safety.

## Motivation

To build a robust and efficient smart contract layer on Solana, a clearly defined and optimized instruction set is required. This specification aims to provide a single source of truth for the Five VM's capabilities, enabling:

1.  **Interoperability**: Ensuring compilers, debuggers, and VMs (Mito, Wasm) all adhere to the same behavior.
2.  **Optimization**: Providing a basis for performance improvements and binary size reduction (VLE, pattern fusion).
3.  **Security**: Clearly defining the constraints and safety guarantees of each operation.

## Specification

The Five VM uses a byte-oriented instruction stream. Instructions consist of a 1-byte opcode followed by variable-length arguments.

### Data Types
- **U8**: 8-bit unsigned integer (1 byte).
- **U16**: 16-bit unsigned integer (2 bytes, little-endian).
- **U32**: 32-bit unsigned integer (4 bytes, little-endian).
- **U64**: 64-bit unsigned integer (8 bytes, little-endian).
- **VLE**: Variable Length Encoding (1-9 bytes) for integers to save space.
- **Pubkey**: 32-byte Solana public key.

### Opcode Tables

#### 1. Control Flow Operations (0x00-0x0F)

These opcodes control the execution flow of the program.

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x00 | `HALT` | None | 0 | Stops execution immediately. | Essential for terminating programs cleanly or stopping execution after a specific branch. |
| 0x01 | `JUMP` | `offset` (U16) | 0 | Unconditional jump to a relative offset. | Enabled loops and unconditional branching logic. |
| 0x02 | `JUMP_IF` | `offset` (U16) | -1 | Jump if top of stack is true. | Basic conditional branching construct (if/else). |
| 0x03 | `JUMP_IF_NOT` | `offset` (U16) | -1 | Jump if top of stack is false. | Often more efficient for "unless" logic or guard clauses. |
| 0x04 | `REQUIRE` | None | -1 | Traps if top of stack is false. | Critical for assertions and security checks (e.g., `require(isAdmin)`). Fails transaction on false. |
| 0x05 | `ASSERT` | None | -1 | Traps if top of stack is false. | Similar to REQUIRE but semantically used for internal invariants. |
| 0x06 | `RETURN` | None | 0 | Returns from the current function. | Standard function return mechanism. |
| 0x07 | `RETURN_VALUE` | None | -1 | Returns from function with a value. | Allows functions to return data to their caller. |
| 0x08 | `NOP` | None | 0 | No Operation. | Useful for padding, debugging, or placeholders during compilation. |
| 0x09 | `BR_EQ_U8` | `value` (U8), `offset` (U16) | -1 | Jump if top (U8) == `value`. | **Optimization**: Fuses `PUSH_U8`, `EQ`, `JUMP_IF` into one instruction. Highly common in state machines and enum matching. |

#### 2. Stack Operations (0x10-0x1F)

These opcodes manipulate the value stack.

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x10 | `POP` | None | -1 | Discards the top value. | Cleaning up the stack after operations where the result is not needed. |
| 0x11 | `DUP` | None | +1 | Duplicates the top value. | Essential when a value is needed for multiple subsequent operations (e.g., `x + x`). |
| 0x12 | `DUP2` | None | +2 | Duplicates the top 2 values. | **Optimization**: Efficient for copying pairs of values (e.g., coordinates, key-value pairs) without multiple swaps/dups. |
| 0x13 | `SWAP` | None | 0 | Swaps the top 2 values. | Reordering operands for non-commutative operations (SUB, DIV) or setting up function calls. |
| 0x14 | `PICK` | `index` (U8) | +1 | Copies the N-th item to top. | Accessing values deep in the stack without destroying the stack layout. |
| 0x15 | `ROT` | None | 0 | Rotates top 3 values. | Reordering 3 items, common in complex logic or shuffling arguments. |
| 0x16 | `DROP` | None | -1 | Same as POP. | Alias for POP, included for compatibility/readability. |
| 0x17 | `OVER` | None | +1 | Copies 2nd item to top. | Common stack pattern (`a b -> a b a`), useful for binary operations where one operand is reused. |
| 0x18 | `PUSH_U8` | `value` (U8) | +1 | Pushes a U8 constant. | Space-efficient way to push small integers (0-255). |
| 0x19 | `PUSH_U16` | `value` (VLE) | +1 | Pushes a U16 constant. | Pushes medium integers. |
| 0x1A | `PUSH_U32` | `value` (VLE) | +1 | Pushes a U32 constant. | Pushes large integers. |
| 0x1B | `PUSH_U64` | `value` (VLE) | +1 | Pushes a U64 constant. | Pushes 64-bit integers (standard Solana number type). |
| 0x1C | `PUSH_I64` | `value` (VLE) | +1 | Pushes an I64 constant. | Pushes signed 64-bit integers. |
| 0x1D | `PUSH_BOOL` | `value` (U8) | +1 | Pushes a boolean. | Pushes true/false. |
| 0x1E | `PUSH_PUBKEY` | `value` (32B) | +1 | Pushes a Pubkey literal. | Essential for hardcoding program IDs or authority keys. |
| 0x1F | `PUSH_U128` | `value` (16B) | +1 | Pushes a U128 constant. | Support for large numbers (e.g., high precision math). |

#### 3. Arithmetic Operations (0x20-0x2F)

Standard arithmetic on U64/I64 values.

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x20 | `ADD` | None | -1 | Addition. | Basic math. |
| 0x21 | `SUB` | None | -1 | Subtraction. | Basic math. |
| 0x22 | `MUL` | None | -1 | Multiplication. | Basic math. |
| 0x23 | `DIV` | None | -1 | Division. | Basic math. |
| 0x24 | `MOD` | None | -1 | Modulo. | Essential for cycles, limits, and hash maps. |
| 0x25 | `GT` | None | -1 | Greater Than. | Comparison. |
| 0x26 | `LT` | None | -1 | Less Than. | Comparison. |
| 0x27 | `EQ` | None | -1 | Equality. | Comparison. |
| 0x28 | `GTE` | None | -1 | Greater Than or Equal. | Comparison. |
| 0x29 | `LTE` | None | -1 | Less Than or Equal. | Comparison. |
| 0x2A | `NEQ` | None | -1 | Not Equal. | Comparison. |
| 0x2B | `NEG` | None | 0 | Negate (unary). | Changes sign of number. |
| 0x2C | `ADD_CHECKED` | None | -1 | Checked Addition. | **Safety**: Errors on overflow instead of wrapping. Crucial for financial math. |
| 0x2D | `SUB_CHECKED` | None | -1 | Checked Subtraction. | **Safety**: Errors on underflow. Crucial for financial math. |
| 0x2E | `MUL_CHECKED` | None | -1 | Checked Multiplication. | **Safety**: Errors on overflow. Crucial for financial math. |

#### 4. Logical & Bitwise Operations (0x30-0x3F)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x30 | `AND` | None | -1 | Logical AND (&&). | Boolean logic. |
| 0x31 | `OR` | None | -1 | Logical OR (\|\|). | Boolean logic. |
| 0x32 | `NOT` | None | 0 | Logical NOT (!). | Boolean logic. |
| 0x33 | `XOR` | None | -1 | Logical XOR. | Boolean logic. |
| 0x34 | `BITWISE_NOT` | None | 0 | Bitwise NOT (~). | Low-level bit manipulation. |
| 0x35 | `BITWISE_AND` | None | -1 | Bitwise AND (&). | Masking bits/flags. |
| 0x36 | `BITWISE_OR` | None | -1 | Bitwise OR (\|). | Setting bits/flags. |
| 0x37 | `BITWISE_XOR` | None | -1 | Bitwise XOR (^). | Toggling bits/flags. |
| 0x38 | `SHIFT_LEFT` | None | -1 | Left Shift (<<). | Fast multiplication by powers of 2, bit packing. |
| 0x39 | `SHIFT_RIGHT` | None | -1 | Logical Right Shift (>>). | Fast division by powers of 2 (unsigned). |
| 0x3A | `SHIFT_RIGHT_ARITH` | None | -1 | Arithmetic Right Shift. | Fast division by powers of 2 (signed, preserves sign). |
| 0x3B | `ROTATE_LEFT` | None | -1 | Rotate Left. | Cryptographic primitives, hash functions. |
| 0x3C | `ROTATE_RIGHT` | None | -1 | Rotate Right. | Cryptographic primitives, hash functions. |
| 0x3D | `BYTE_SWAP_16` | None | 0 | Swap bytes in U16. | Endianness conversion. |
| 0x3E | `BYTE_SWAP_32` | None | 0 | Swap bytes in U32. | Endianness conversion. |
| 0x3F | `BYTE_SWAP_64` | None | 0 | Swap bytes in U64. | Endianness conversion. |

#### 5. Memory Operations (0x40-0x4F)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x40 | `STORE` | `offset` (U32) | -1 | Store to memory. | Writing to global/heap memory. |
| 0x41 | `LOAD` | `offset` (U32) | +1 | Load from memory. | Reading from global/heap memory. |
| 0x42 | `STORE_FIELD` | `acct_idx` (U8), `offset` (VLE) | -1 | Zero-copy store to account data. | **Optimization**: Direct write to account buffer without loading full data. Critical for performance. |
| 0x43 | `LOAD_FIELD` | `acct_idx` (U8), `offset` (VLE) | +1 | Zero-copy load from account data. | **Optimization**: Direct read from account buffer. |
| 0x44 | `LOAD_INPUT` | `index` (U8) | +1 | Load instruction input. | Accessing raw instruction data. |
| 0x45 | `STORE_GLOBAL` | `id` (U16) | -1 | Store to global var. | Persisting state across function calls within a transaction. |
| 0x46 | `LOAD_GLOBAL` | `id` (U16) | +1 | Load from global var. | Accessing global state. |
| 0x47 | `LOAD_EXTERNAL_FIELD` | None | -1 | Load field from external account. | Reading state from other programs/accounts dynamically. |
| 0x48 | `LOAD_FIELD_PUBKEY` | `acct_idx`, `offset` | +1 | Load Pubkey from account data. | Specialized zero-copy load for 32-byte keys. |

#### 6. Account Operations (0x50-0x5F)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x50 | `CREATE_ACCOUNT` | None | Dynamic | Create new account. | Fundamental Solana capability. |
| 0x51 | `LOAD_ACCOUNT` | `acct_idx` | +1 | Push account ref to stack. | Needed to pass accounts to other functions or ops. |
| 0x52 | `SAVE_ACCOUNT` | `acct_idx` | -1 | Save account modifications. | Committing changes to account data. |
| 0x53 | `GET_ACCOUNT` | `acct_idx` | +1 | Get account info struct. | Accessing full account metadata. |
| 0x54 | `GET_LAMPORTS` | `acct_idx` | +1 | Get balance. | Checking account funds. |
| 0x55 | `SET_LAMPORTS` | `acct_idx` | -1 | Set balance. | Modifying balance (usually via transfer, but internal use too). |
| 0x56 | `GET_DATA` | `acct_idx` | +1 | Get data slice. | Reading account state. |
| 0x57 | `GET_KEY` | `acct_idx` | +1 | Get account Pubkey. | Identifying accounts. |
| 0x58 | `GET_OWNER` | `acct_idx` | +1 | Get owner Pubkey. | Verifying account ownership. |
| 0x59 | `TRANSFER` | None | -3 | Transfer SOL. | Moving funds (from, to, amount). |
| 0x5A | `TRANSFER_SIGNED` | None | -3 | Transfer SOL with seeds. | Moving funds from a PDA. |

#### 7. Array & String Operations (0x60-0x6F)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x60 | `CREATE_ARRAY` | `capacity` (U8) | +1 | Allocate new array. | Dynamic data structures. |
| 0x61 | `PUSH_ARRAY_LITERAL` | `len` (U8) | +1 | Push array from bytecode. | Constant arrays. |
| 0x62 | `ARRAY_INDEX` | None | -1 | Get item at index. | Accessing elements. |
| 0x63 | `ARRAY_LENGTH` | None | 0 | Get length. | Iterating or bounds checking. |
| 0x64 | `ARRAY_SET` | None | -3 | Set item at index. | Modifying arrays. |
| 0x65 | `ARRAY_GET` | None | -1 | Get item (alias/variant). | Accessing elements. |
| 0x66 | `PUSH_STRING_LITERAL` | `len` (U8) | +1 | Push string from bytecode. | Constant strings. |
| 0x67 | `PUSH_STRING` | `len` (VLE) | +1 | Push long string. | Large text data. |

#### 8. Constraint Operations (0x70-0x7F)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x70 | `CHECK_SIGNER` | `acct_idx` (U8) | 0 | Assert account is signer. | **Security**: Authentication checks. |
| 0x71 | `CHECK_WRITABLE` | `acct_idx` (U8) | 0 | Assert account is writable. | **Security**: Ensuring mutation is allowed. |
| 0x72 | `CHECK_OWNER` | `acct_idx` | -1 | Assert account owner. | **Security**: verifying program ownership. |
| 0x73 | `CHECK_INITIALIZED` | `acct_idx` (U8) | 0 | Assert account has data. | Preventing overwrites or use of uninit accounts. |
| 0x74 | `CHECK_PDA` | `acct_idx` | -1 | Assert account is PDA. | **Security**: Verifying address derivation. |
| 0x75 | `CHECK_UNINITIALIZED` | `acct_idx` (U8) | 0 | Assert account empty. | Ensuring safe new account creation. |
| 0x76 | `CHECK_DEDUPE_TABLE` | None | -1 | Verify uniqueness. | Advanced validation patterns. |
| 0x77 | `CHECK_CACHED` | None | -1 | Check cache. | Performance optimization for repeated checks. |
| 0x78 | `CHECK_COMPLEXITY_GROUP` | None | -1 | Validate complexity. | Resource management. |
| 0x79 | `CHECK_DEDUPE_MASK` | None | -1 | Verify via bitmask. | Efficient batch uniqueness checks. |

#### 9. System Operations (0x80-0x8F)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x80 | `INVOKE` | None | Dynamic | Cross-Program Invocation. | Interacting with other Solana programs. |
| 0x81 | `INVOKE_SIGNED` | None | Dynamic | CPI with seeds. | Interacting as a PDA. |
| 0x82 | `GET_CLOCK` | None | +1 | Get SysvarClock. | Accessing time/slot. |
| 0x83 | `GET_RENT` | None | +1 | Get SysvarRent. | Calculating rent exemption. |
| 0x84 | `INIT_ACCOUNT` | None | -4 | SystemProgram::CreateAccount. | Standard account creation helper. |
| 0x85 | `INIT_PDA_ACCOUNT` | None | Dynamic | SystemProgram::CreateAccountWithSeed. | Creating PDAs. |
| 0x86 | `DERIVE_PDA` | None | +1 | Compute PDA address. | Finding addresses. |
| 0x87 | `FIND_PDA` | None | +1 | Find PDA + bump. | Finding valid PDA bumps. |
| 0x88 | `DERIVE_PDA_PARAMS` | None | Dynamic | Derive with explicit params. | Flexible derivation. |
| 0x89 | `FIND_PDA_PARAMS` | None | Dynamic | Find with explicit params. | Flexible finding. |

#### 10. Function Transport (0x90-0x9F)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0x90 | `CALL` | `params` (U8), `addr` (U16) | 0 | Call internal function. | Modular code structure. |
| 0x91 | `CALL_EXTERNAL` | `acct`, `off`, `params` | 0 | Call into another Five program. | Composability between Five contracts. |
| 0x92 | `CALL_NATIVE` | None | 0 | Call native host function. | Performance or system access. |
| 0x93 | `PREPARE_CALL` | None | 0 | Setup call stack. | Call overhead management. |
| 0x94 | `FINISH_CALL` | None | 0 | Teardown call stack. | Call overhead management. |

#### 11. Local Variables (0xA0-0xAF)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0xA0 | `ALLOC_LOCALS` | None | 0 | Allocate stack frame locals. | Function local state isolation. |
| 0xA1 | `DEALLOC_LOCALS` | None | 0 | Free locals. | Cleanup. |
| 0xA2 | `SET_LOCAL` | `idx` (U8) | -1 | Set local var. | Storing temporary values without stack juggling. |
| 0xA3 | `GET_LOCAL` | `idx` (U8) | +1 | Get local var. | Retrieving temporary values. |
| 0xA4 | `CLEAR_LOCAL` | `idx` (U8) | 0 | Clear local var. | Security/cleanup. |
| 0xA5 | `LOAD_PARAM` | `idx` (U8) | +1 | Load function parameter. | Accessing arguments passed to function. |
| 0xA6 | `STORE_PARAM` | `idx` (U8) | -1 | Store to parameter slot. | Modifying arguments (if mutable). |
| 0xA7 | `WRITE_DATA` | None | -2 | Write bytes to buffer. | Low-level data construction. |
| 0xA8 | `DATA_LEN` | None | +1 | Get buffer length. | Data construction utility. |
| 0xA9 | `EMIT_EVENT` | None | -1 | Emit log/event. | Indexing and off-chain tracking. |
| 0xAA | `LOG_DATA` | None | -1 | Log raw data. | Debugging. |
| 0xAB | `GET_SIGNER_KEY` | None | +1 | Get signer pubkey. | Auth utility. |
| 0xAF | `CAST` | `type` (U8) | 0 | Type cast value. | Type system safety/conversion. |

#### 12. Register Operations (0xB0-0xBF)

Hybrid VM optimizations to reduce stack traffic.

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0xB0 | `LOAD_REG_U8` | `reg`, `val` | 0 | Load immediate U8 to reg. | Fast constant access. |
| 0xB1 | `LOAD_REG_U32` | `reg`, `val` | 0 | Load immediate U32. | Fast constant access. |
| 0xB2 | `LOAD_REG_U64` | `reg`, `val` | 0 | Load immediate U64. | Fast constant access. |
| 0xB3 | `LOAD_REG_BOOL` | `reg`, `val` | 0 | Load immediate Bool. | Fast constant access. |
| 0xB4 | `LOAD_REG_PUBKEY` | `reg`, `val` | 0 | Load immediate Pubkey. | Fast constant access. |
| 0xB5 | `ADD_REG` | `dest`, `s1`, `s2` | 0 | Register addition. | **Performance**: Math without stack pop/push overhead. |
| 0xB6 | `SUB_REG` | `dest`, `s1`, `s2` | 0 | Register subtraction. | Performance optimization. |
| 0xB7 | `MUL_REG` | `dest`, `s1`, `s2` | 0 | Register multiplication. | Performance optimization. |
| 0xB8 | `DIV_REG` | `dest`, `s1`, `s2` | 0 | Register division. | Performance optimization. |
| 0xB9 | `EQ_REG` | `dest`, `s1`, `s2` | 0 | Register equality. | Performance optimization. |
| 0xBA | `GT_REG` | `dest`, `s1`, `s2` | 0 | Register > check. | Performance optimization. |
| 0xBB | `LT_REG` | `dest`, `s1`, `s2` | 0 | Register < check. | Performance optimization. |
| 0xBC | `PUSH_REG` | `reg` | +1 | Push register to stack. | Moving from fast registers to stack. |
| 0xBD | `POP_REG` | `reg` | -1 | Pop stack to register. | Moving from stack to fast registers. |
| 0xBE | `COPY_REG` | `dest`, `src` | 0 | Copy register. | Fast local data movement. |
| 0xBF | `CLEAR_REG` | `reg` | 0 | Zero register. | Cleanup. |

#### 13. Nibble/Compressed Operations (0xD0-0xDF)

Highly optimized single-byte instructions for common operations.

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0xD0-D3| `GET_LOCAL_0..3`| None | +1 | Get local 0-3. | **Compression**: Replaces `GET_LOCAL + idx` (2 bytes) with 1 byte. 50% savings. |
| 0xD4-D7| `SET_LOCAL_0..3`| None | -1 | Set local 0-3. | **Compression**: 50% savings for most common locals. |
| 0xD8-DB| `PUSH_0..3` | None | +1 | Push 0, 1, 2, or 3. | **Compression**: Replaces `PUSH_U64 + val` (2-9 bytes) with 1 byte. Huge savings for counters/indices. |
| 0xDC-DF| `LOAD_PARAM_0..3`| None | +1 | Load param 0-3. | **Compression**: 50% savings for argument access. |

#### 14. Advanced/Experimental (0xF0-0xFF)

| Opcode | Name | Arguments | Stack Effect | Description | Rationale/Utility |
|:---:|:---|:---|:---:|:---|:---|
| 0xF0 | `RESULT_OK` | None | 0 | Wrap in Result::Ok. | Error handling. |
| 0xF1 | `RESULT_ERR` | None | 0 | Wrap in Result::Err. | Error handling. |
| 0xF2 | `OPTIONAL_SOME` | None | 0 | Wrap in Option::Some. | Null safety. |
| 0xF3 | `OPTIONAL_NONE` | None | +1 | Push Option::None. | Null safety. |
| 0xF4 | `OPTIONAL_UNWRAP`| None | 0 | Unwrap or panic. | Accessing optional values. |
| 0xF5 | `OPTIONAL_IS_SOME`| None | 0 | Check if some. | Conditional logic on optionals. |
| 0xF6 | `OPTIONAL_GET_VALUE`| None | 0 | Unsafe get. | Performance (if checked externally). |
| 0xF7 | Reserved | - | - | Reserved. | - |
| 0xF8 | `CREATE_TUPLE` | `n` | -N+1 | Create tuple of n items. | Grouping values. |
| 0xF9 | `TUPLE_GET` | `idx` | 0 | Get item from tuple. | Accessing grouped values. |
| 0xFA | `UNPACK_TUPLE` | None | +N-1 | Explode tuple to stack. | Using grouped values. |
| 0xFB | Reserved | - | - | Reserved. | - |
| 0xFC | Reserved | - | - | Reserved. | - |
| 0xFD | `OPTIONAL_IS_NONE`| None | 0 | Check if none. | Conditional logic. |
| 0xFE | `RESULT_IS_OK` | None | 0 | Check if ok. | Conditional logic. |
| 0xFF | `RESULT_IS_ERR` | None | 0 | Check if err. | Conditional logic. |

## Rationale

The design of the Five VM opcode set balances several competing requirements:

1.  **Compactness**: Instructions like `JUMP_IF`, `REQUIRE`, and the nibble operations (0xD0-0xDF) are designed to minimize binary size, which translates to lower deployment costs on Solana.
2.  **Solana Alignment**: Account operations map directly to Solana's account model, efficiently using CPIs and local state management.
3.  **Safety**: Checked arithmetic and explicit constraint opcodes (`CHECK_SIGNER`, `CHECK_WRITABLE`) allow for robust security assertions.
4.  **Performance**: Zero-copy operations (`LOAD_FIELD`) and register operations reduce the overhead of memory copying and stack manipulation.

## Backwards Compatibility

This specification represents version 1 of the ISA. Future additions (e.g., in the 0xE0-0xEF range) must maintain backward compatibility with existing binaries. Breaking changes will require a new version of the VM.

## Security Considerations

-   **Resource Limits**: All operations are assigned a compute unit cost to prevent denial-of-service attacks.
-   **Validation**: Loaders and constraints (like `CHECK_SIGNER`) must be strictly enforced.
-   **Isolation**: The VM ensures that programs cannot access memory or accounts outside their authorized scope.

## Copyright

Copyright (c) 2024 Five Team.
