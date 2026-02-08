# Handoff: Five SDK Parameter Encoding - VLE Removal Migration

**Date**: February 8, 2026
**Branch**: `registers_broken`
**Status**: 🔴 BLOCKED - E2E tests failing due to parameter encoding stub
**Priority**: Critical - Blocks all SDK-based smart contract execution

---

## Executive Summary

The Five VM ecosystem has **completely removed VLE (Variable Length Encoding) and Register-based opcodes**, but the **SDK's parameter encoder is still using a stub that returns dummy data**. This causes all function parameters to be lost before execution, preventing the Five VM from receiving the actual contract function arguments.

**Impact**: E2E token and counter tests fail with "Provided owner is not allowed" error (secondary symptom of missing account creation parameters).

---

## Problem Statement

### Current Behavior
When calling a smart contract function via the SDK:
1. Parameters are correctly prepared and structured
2. `FiveSDK.generateExecuteInstruction()` is called
3. Parameter encoder returns only **9 bytes** (magic byte + counts): `09 00 00 00 00 00 00 00 00`
4. No actual parameter data is included in the bytecode
5. Five VM receives instruction but lacks function parameters → execution fails

### Expected Behavior
For the `init_mint` function with 7 parameters (2 accounts + 5 data params):
- Should generate **100+ bytes** with PUSH opcodes and parameter values
- Each parameter encoded as: `PUSH_<TYPE> <value_in_fixed_size_LE>`
- Five VM receives complete instruction with all parameters on stack

### Test Case
**File**: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-templates/token/e2e-token-test.mjs`
**Error**:
```
Program 6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k failed: Provided owner is not allowed
CU consumed: 98 (fails immediately - not reaching account creation logic)
```

---

## Root Cause Analysis

### Stub Parameter Encoder
**File**: `five-sdk/src/assets/vm/five_vm_wasm.js`

```javascript
// CURRENT (BROKEN) - Returns dummy data
export const ParameterEncoder = {
    encode_execute: (funcIdx, params) => {
        return new Uint8Array([9, 0,0,0,0, 0,0,0,0]);  // Always 9 bytes!
    }
};
```

**Problem**:
- Ignores the `params` argument entirely
- Returns fixed stub regardless of parameter count/types
- No actual bytecode generation for parameters

### VLE Removal Impact
The original encoder used VLE (Variable Length Encoding), which has been **completely removed** from:
- Five Protocol (`five-protocol/src/opcodes.rs` - no VLE references)
- Five VM (`five-vm-mito` - stack-based, no VLE decoding)
- Five DSL Compiler (`five-dsl-compiler` - emits fixed-size opcodes)

**New Protocol**: All values use **fixed-size little-endian encoding**:
- `PUSH_U8` (0x18): 1 byte opcode + 1 byte value = 2 bytes total
- `PUSH_U16` (0x19): 1 byte opcode + 2 bytes LE value = 3 bytes total
- `PUSH_U32` (0x1A): 1 byte opcode + 4 bytes LE value = 5 bytes total
- `PUSH_U64` (0x1B): 1 byte opcode + 8 bytes LE value = 9 bytes total
- `PUSH_PUBKEY` (0x1E): 1 byte opcode + 32 bytes = 33 bytes total
- Special opcodes: `PUSH_0` (0xD8), `PUSH_1` (0xD9), `PUSH_2` (0xDA), `PUSH_3` (0xDB)

---

## What Has Been Fixed

### 1. SDK Account Metadata (✅ FIXED)
**Issue**: SystemProgram was marked as `isWritable: true` causing Solana validation errors
**Fix**: Modified `FunctionBuilder.ts` to pass account metadata to `FiveSDK.generateExecuteInstruction()`
**Files Modified**:
- `five-sdk/src/program/FunctionBuilder.ts` - Line 415-420: Pass `accountMetadata` option
- `five-sdk/src/FiveSDK.ts` - Line 336: Added `accountMetadata` to options type
- `five-sdk/src/modules/execute.ts` - Line 240: Accept and use `accountMetadata` parameter

### 2. VM Compilation Cfg Gate (✅ FIXED)
**Issue**: `five-vm-mito/src/systems/accounts.rs` missing `test` cfg gate removed `Instruction` and `invoke_signed` imports
**Fix**: Restored `#[cfg(any(target_os = "solana", test))]` gate
**Files Modified**:
- `five-vm-mito/src/systems/accounts.rs` - Lines 10-13: Restore test cfg gate

### 3. Parameter Encoder Framework (⚠️ PARTIAL)
**File**: `five-sdk/src/assets/vm/five_vm_wasm.js`
**Status**: Implementation written but not being called due to module loading issue

**Code Written** (not yet functional):
- Full fixed-size parameter encoder functions:
  - `encodeU8()`, `encodeU16LE()`, `encodeU32LE()`, `encodeU64LE()`
  - `encodeSingleParameter()` - Handles account, u8, u16, u32, u64, bool, pubkey, string types
  - `ParameterEncoder.encode_execute()` - Main entry point with debug logging

---

## What Still Needs Investigation

### 1. Module Loading/Caching Issue (🔴 CRITICAL)
**Problem**: The new `ParameterEncoder.encode_execute()` implementation exists in source but isn't being called

**Investigation Steps**:
1. Verify WASM module is being imported from correct path
2. Check if Node.js module caching is preventing new version from loading
3. Verify `five-sdk/dist/assets/vm/five_vm_wasm.js` is being updated on rebuild
4. Add debug logging to confirm `wasmModule.ParameterEncoder.encode_execute()` is callable

**Key Files**:
- `five-sdk/src/wasm/loader.ts` - Module loading logic
- `five-sdk/src/lib/bytecode-encoder.ts` - Calls `wasmModule.ParameterEncoder.encode_execute()`
- Build script: Verify `copy-assets` is copying latest files

### 2. Parameter Type Handling
**Issue**: Encoder needs to handle various parameter types correctly

**Types to Support**:
- **Accounts**: PUSH_U8 with account index (offset by 2)
- **Pubkeys**: PUSH_PUBKEY with 32-byte public key
- **Strings**: Need to determine Five's string encoding format
- **Numbers**: u8, u16, u32, u64, i64, bool
- **Custom Types**: Mint, TokenAccount, etc. (alias to account)

**Questions**:
- How are strings encoded in Five? Length-prefixed? Null-terminated? Raw bytes?
- Are pubkeys expected as Base58 strings or raw 32-byte arrays?
- What's the account index offset? (Currently assuming +2 for script + vm_state accounts)

### 3. Proper Bytecode Format Verification
**Questions**:
- Is the instruction format: `[discriminator(1) | funcIdx(u32) | paramCount(u32) | parameterBytes]`?
- Should parameter values be raw bytes or PUSH opcodes + values?
- Verify against compiler's bytecode generation in `five-dsl-compiler/src/bytecode_generator/`

---

## Test Plan

### Local Testing
```bash
# Terminal 1: Start localnet
solana-test-validator

# Terminal 2: Deploy Five VM
cd five-solana && cargo build-sbf --release && solana program deploy target/deploy/five.so

# Terminal 3: Run tests
cd five-templates/token && node e2e-token-test.mjs
cd five-templates/counter && node e2e-counter-test.mjs
```

### Success Criteria
- ✅ Parameters encoded as **100+ bytes** (not 9)
- ✅ Instruction data starts with correct PUSH opcodes
- ✅ E2E token test reaches Step 2 (init_token_account)
- ✅ Counter test successfully initializes and increments

### Verification Commands
```bash
# Decode base64 instruction data to hex
echo "<base64_data>" | base64 -d | xxd

# Should see PUSH opcodes (0x18, 0x19, 0x1A, 0x1B, 0x1E, etc.)
# Currently shows: 09 00 00 00 00 00 00 00 00
```

---

## Key Files and References

### Five Protocol (Opcode Definitions)
- `five-protocol/src/opcodes.rs` - All opcode constants (PUSH_U8=0x18, etc.)
- `five-protocol/OPCODE_SPEC.md` - Full opcode specification with fixed-size encoding
- `five-protocol/src/bytecode_builder.rs` - Reference implementation of bytecode building

### Five DSL Compiler (Reference)
- `five-dsl-compiler/src/bytecode_generator/opcodes.rs` - Methods like `emit_push_u64()`, `emit_push_u32()`
- `five-dsl-compiler/src/bytecode_generator/ast_generator/` - How compiler generates PUSH opcodes for literals

### Five SDK (Needs Fixing)
- `five-sdk/src/lib/bytecode-encoder.ts` - Calls ParameterEncoder, needs verification
- `five-sdk/src/assets/vm/five_vm_wasm.js` - Parameter encoder implementation
- `five-sdk/src/modules/execute.ts` - Instruction generation logic
- `five-sdk/src/program/FunctionBuilder.ts` - Parameter merging and metadata

### Test Templates (Verification)
- `five-templates/token/e2e-token-test.mjs` - 7-param function test (good for debugging)
- `five-templates/counter/e2e-counter-test.mjs` - 2-param function test (simpler)

---

## Recommendations for Next Agent

### Priority 1: Fix Module Loading
1. Add console.log at start of `ParameterEncoder.encode_execute()` to confirm it's called
2. Check `five-sdk/src/wasm/loader.ts` to verify module import paths
3. Verify `dist/` folder has latest WASM module (may need manual copy or build fix)
4. Test parameter encoding in isolation before end-to-end tests

### Priority 2: Verify Bytecode Format
1. Study `five-protocol/OPCODE_SPEC.md` for exact parameter encoding requirements
2. Review compiler's `emit_push_*()` functions as reference implementation
3. Check if parameters should be raw values or PUSH-encoded opcodes
4. Validate against Five VM's instruction format in `five-vm-mito/src/lib.rs`

### Priority 3: Handle Edge Cases
1. Implement proper Base58 decoding for pubkey parameters (currently using dummy)
2. Determine string encoding format and implement proper encoding
3. Handle custom account types (Mint, TokenAccount, ProgramAccount)
4. Add account index offset calculation (currently assumes +2)

### Debugging Tips
- Enable debug mode in test: Check console for `[BytecodeEncoder]` and `[ParameterEncoder]` logs
- Use hex dump to inspect instruction data: `echo "<data>" | base64 -d | xxd`
- Compare working bytecode (compiler output) with SDK output
- Check if WASM module is being cached by Node.js - may need process restart

---

## Historical Context

**Branch Name**: `registers_broken` suggests recent major refactoring
**Recent Changes**:
- Removed register-based opcodes (commit: `72f4ad8`)
- Removed VLE encoding (commit: `77bf138`)
- Reference stack virtualization (commit: `510bf90`)

These changes removed VLE completely, but SDK wasn't updated. The stub encoder is a temporary placeholder that was never replaced with proper fixed-size implementation.

---

## Success Metrics

Once fixed, the following should work:
```javascript
// Should generate 100+ bytes with proper parameters
const instruction = await program.function('init_mint')
  .accounts({ mint_account, authority })
  .args({ freeze_authority, decimals, name, symbol, uri })
  .instruction();

// Should see PUSH opcodes in instruction data
console.log(instruction.data); // Should be ~150 bytes in base64

// Should pass account creation stage
const result = await sendInstruction(connection, instruction, [payer, user1, mintAccount]);
// Should succeed with signature, not "Provided owner is not allowed"
```

---

## Contact Points

If you need clarification on:
- **Architecture**: See `CLAUDE.md` in root and component directories
- **Protocol Details**: See `five-protocol/OPCODE_SPEC.md`
- **Compiler Reference**: See `five-dsl-compiler/src/bytecode_generator/opcodes.rs`
- **VM Execution**: See `five-vm-mito/src/lib.rs` and handlers

