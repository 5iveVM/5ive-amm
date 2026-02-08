# Changes Made - SDK Parameter Encoding Investigation

**Date**: February 8, 2026
**Branch**: `registers_broken`
**Objective**: Investigate and fix parameter encoding stub preventing E2E tests from passing

---

## Summary of Changes

**Total Files Modified**: 9
**Lines Added**: 207
**Lines Removed**: 60
**Net Change**: +147 lines

---

## Detailed Changes

### 1. ✅ FIXED: VM Compilation Config Gate
**File**: `five-vm-mito/src/systems/accounts.rs`
**Lines**: 10-13
**Issue**: Test configuration was removed from import guards, breaking account creation on localnet
**Fix**: Restored `test` cfg gate for Instruction and invoke_signed imports

```rust
// Before:
#[cfg(target_os = "solana")]
use pinocchio::instruction::{AccountMeta, Seed};

// After:
#[cfg(any(target_os = "solana", test))]
use pinocchio::instruction::{AccountMeta, Instruction, Seed};
#[cfg(any(target_os = "solana", test))]
use pinocchio::program::invoke_signed;
```

---

### 2. ✅ FIXED: SDK Account Metadata Passing
**File**: `five-sdk/src/program/FunctionBuilder.ts`
**Lines**: 415-420
**Issue**: Account metadata (isWritable flags) was built but not passed to bytecode encoder
**Fix**: Pass accountMetadata to FiveSDK.generateExecuteInstruction()

```typescript
// Added:
{
  debug: this.options.debug,
  abi: this.abi,
  fiveVMProgramId: this.options.fiveVMProgramId,
  vmStateAccount: this.options.vmStateAccount,
  adminAccount: this.options.feeReceiverAccount,
  accountMetadata: accountMetadata,  // ← NEW
}
```

---

### 3. ✅ FIXED: FiveSDK Type Definition
**File**: `five-sdk/src/FiveSDK.ts`
**Lines**: 336
**Issue**: Account metadata type not defined in options
**Fix**: Added `accountMetadata` parameter type

```typescript
options: {
  debug?: boolean;
  computeUnitLimit?: number;
  vmStateAccount?: string;
  fiveVMProgramId?: string;
  abi?: any;
  adminAccount?: string;
  estimateFees?: boolean;
  accountMetadata?: Map<string, { isSigner: boolean; isWritable: boolean; isSystemAccount?: boolean }>;
}
```

---

### 4. ✅ FIXED: Execute Module Metadata Usage
**File**: `five-sdk/src/modules/execute.ts`
**Lines**: 240, 437-443
**Issue**: Account metadata wasn't being used when building instruction accounts
**Fix**: Check both ABI metadata and passed-in metadata when determining account flags

```typescript
// Changed from:
const metadata = abiAccountMetadata.get(acc);

// To:
const abiMetadata = abiAccountMetadata.get(acc);
const passedMetadata = options.accountMetadata?.get(acc);
const metadata = abiMetadata || passedMetadata;
```

---

### 5. ✅ PARTIAL: Parameter Encoder Implementation
**File**: `five-sdk/src/assets/vm/five_vm_wasm.js`
**Lines**: 1-189 (complete rewrite)
**Issue**: Stub encoder ignores all parameters, returns fixed 9-byte dummy
**Status**: ⚠️ Implementation exists but module loading issue prevents usage
**Details**:

**Implemented Functions**:
- `encodeU8(value)` - Single byte encoding
- `encodeU16LE(value)` - 16-bit little-endian
- `encodeU32LE(value)` - 32-bit little-endian
- `encodeU64LE(value)` - 64-bit little-endian with BigInt support
- `stringToBase58OrBytes(str)` - String encoding (UTF-8, needs proper Base58)
- `encodePubkey(pubkeyStr)` - Pubkey encoding (placeholder, needs Base58 decoding)
- `encodeSingleParameter(param, value, accountIndexOffset)` - Type-aware parameter encoding
- `ParameterEncoder.encode_execute(funcIdx, params)` - Main entry point with debug logging

**Type Support**:
- Accounts: PUSH_U8 with index (offset by +2)
- u8: PUSH_U8 with optimizations for 0-3 (PUSH_0/PUSH_1/PUSH_2/PUSH_3)
- u16: PUSH_U16 with LE encoding
- u32: PUSH_U32 with LE encoding
- u64/i64: PUSH_U64 with LE encoding
- bool: PUSH_0 or PUSH_1
- pubkey: PUSH_PUBKEY with 32 bytes (needs work)
- string: Length prefix + UTF-8 bytes (needs verification)

---

### 6. 📊 DEBUG: Parameter Encoding Logging
**File**: `five-sdk/src/lib/bytecode-encoder.ts`
**Lines**: 165-179
**Purpose**: Understand why new encoder isn't being called
**Status**: Logs show it's still returning 9-byte stub

```typescript
console.log(`[BytecodeEncoder] About to encode with paramArray:`, ...);
console.log(`[BytecodeEncoder] WASM encoded ${paramArray.length} parameters: ${buf.length} bytes`);
console.log(`[BytecodeEncoder] Encoded result: ${Buffer.from(encoded).toString('hex')}`);
```

---

### 7. 📦 REMOVED: Test Artifacts
**Files Deleted**:
- `five-templates/token/.five/build.json` (18 lines)
- `five-templates/token/test-state-fiveprogram.json` (26 lines)

**Reason**: Previous test run artifacts from failed executions

---

### 8. ❓ UNKNOWN: Config File Change
**File**: `five-dsl-compiler/src/config/project_config.rs`
**Lines**: -3
**Reason**: Unknown (possibly unrelated, should investigate)

---

## What's Still Broken

### Module Loading Issue (🔴 CRITICAL)
**Symptom**:
- Logs show `[BytecodeEncoder] WASM encoded 7 parameters: 9 bytes`
- Returned hex: `090000000000000000`
- Expected: 100+ bytes with PUSH opcodes

**Root Cause**:
- `ParameterEncoder.encode_execute()` implementation in source is not being called
- Likely module caching or import resolution issue
- New encode_execute() function exists in source but still returns stub in output

**Next Steps**:
1. Verify `dist/assets/vm/five_vm_wasm.js` contains new implementation
2. Check if Node.js module cache needs clearing
3. Add logging to confirm function entry point
4. Verify WASM loader is importing from correct path

---

## Files Ready for Investigation

### Critical Path Files
1. `five-sdk/src/wasm/loader.ts` - Module loading logic
2. `five-sdk/src/assets/vm/five_vm_wasm.js` - Encoder implementation
3. `five-sdk/src/lib/bytecode-encoder.ts` - Encoder caller
4. `five-sdk/dist/assets/vm/five_vm_wasm.js` - Check if build updated this

### Reference Implementation
1. `five-dsl-compiler/src/bytecode_generator/opcodes.rs` - How compiler emits PUSH opcodes
2. `five-protocol/OPCODE_SPEC.md` - Opcode specifications with fixed-size encoding

### Test Cases
1. `five-templates/token/e2e-token-test.mjs` - Complex test (7 params)
2. `five-templates/counter/e2e-counter-test.mjs` - Simple test (2 params)

---

## Testing Results

### Current Test Status: ❌ FAILING

```bash
$ node e2e-token-test.mjs

[BytecodeEncoder] WASM encoded 7 parameters: 9 bytes      ← WRONG! Should be 100+
[BytecodeEncoder] Encoded result: 090000000000000000      ← Stub output

❌ init_mint FAILED
   Error: Transaction simulation failed: Provided owner is not allowed
   Program consumed 98 CU (fails immediately)
```

### Expected After Fix: ✅ PASSING

```bash
$ node e2e-token-test.mjs

[BytecodeEncoder] WASM encoded 7 parameters: 156 bytes    ← CORRECT
[ParameterEncoder] Encoding 7 parameters               ← New logs should appear
[ParameterEncoder] Param 0: name=mint_account, type=account, ...
...
[ParameterEncoder] Returning 156 bytes: 0x18 0x02 0x1e ...

✅ init_mint succeeded
   Signature: <transaction_signature>
   CU: <actual_usage>
```

---

## Verification Checklist

Before considering this fixed, verify:

- [ ] `ParameterEncoder.encode_execute()` logs appear when running tests
- [ ] Instruction data is **100+ bytes** (not 9)
- [ ] First bytes include PUSH opcodes (0x18, 0x19, 0x1A, 0x1B, 0x1E)
- [ ] Token E2E test reaches Step 2 (init_token_account)
- [ ] Counter E2E test successfully initializes
- [ ] Transaction CU usage is reasonable (not 98)

---

## Branch State

**Current Branch**: `registers_broken`
**Committed Changes**: None (all changes are staged/unstaged)
**Ready to Commit**: Yes, these changes are improvements regardless of parameter encoding fix

**Suggested Commit Message**:
```
fix: restore VM compilation cfg gate and pass account metadata to SDK

- Fix five-vm-mito compilation on non-Solana targets by restoring test cfg gate
- Pass account metadata from FunctionBuilder to SDK encoder for correct isWritable flags
- Add parameter encoder framework for fixed-size bytecode generation (post-VLE removal)
- Add debug logging to trace parameter encoding issues

This addresses the foundation for parameter encoding but module loading issue
prevents new encoder from being used. See HANDOFF_SDK_PARAMETER_ENCODING.md for
complete analysis and next steps.
```

