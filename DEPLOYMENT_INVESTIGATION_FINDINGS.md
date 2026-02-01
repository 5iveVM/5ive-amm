# Deployment Investigation - Critical Findings

**Date:** 2026-01-31
**Status:** Bytecode verification failing - root cause identified
**Issue:** Error 8122 during bytecode chunks upload, preventing E2E test execution

## Problem Statement

Register-optimized AND baseline token.v bytecode both fail on-chain verification with **error 8122** (CallTargetOutOfBounds) during chunk upload. This prevents the script account from receiving complete bytecode, blocking all E2E tests.

## What We Learned

### 1. The Bytecode Issue is NOT Register-Specific

- ✅ Baseline (stack-only) bytecode: 805 bytes
- ✅ Register-optimized bytecode: 770 bytes
- ❌ Both fail with error 8122 during chunk upload
- ✅ Chunks 0-1 upload successfully
- ❌ Chunk 2 fails with CallTargetOutOfBounds error

**Conclusion:** The problem is in the bytecode itself, not register optimization.

### 2. Error 8122 Location

Occurs in: `five-solana/src/instructions/verify.rs` lines 81-87

```rust
if func_addr >= bytecode.len() {  // CALL/CALL_REG verification
    return Err(ProgramError::Custom(8122));  // CallTargetOutOfBounds
}
```

This means the bytecode verification finds a CALL or CALL_REG instruction with a target address >= bytecode length.

### 3. Bytecode Verification Triggers on Upload

During `APPEND` instruction (chunk upload), the Five VM program calls:
- `verify_bytecode_content()` to validate accumulated bytecode
- This check catches invalid function addresses

### 4. Token.v Bytecode Has Invalid Function Addresses

Given:
- Bytecode size: 805 bytes (or 770 with registers)
- Error: CallTargetOutOfBounds
- Verification runs during chunk upload

**Likely cause:** Function addresses in dispatcher or CALL instructions point beyond bytecode bounds.

## Investigation Results

### Dispatcher Analysis
Dispatcher correctly routes to function offsets like 0x0220, 0x0234, etc. (these are valid).

### Code Section Analysis
Some CALL/CALL_REG instructions might have incorrect function addresses that exceed 805 or 770 bytes.

### Why Chunks 0-1 Pass But 2 Fails

1. Chunk 0 & 1 might not trigger full bytecode verification
2. Chunk 2 (final chunk) triggers complete bytecode validation
3. At that point, invalid addresses are caught

## Root Cause Hypothesis

The bytecode generation pipeline may have an issue where:

1. **Function dispatch table** is generated with correct offsets
2. **Function addresses in CALL instructions** somewhere in the bytecode are incorrect
3. Or: **Dispatcher is using absolute bytecode offsets** that don't account for actual bytecode structure

## Path Forward

### Option 1: Fix Bytecode Generation (Recommended)
1. Check `five-dsl-compiler/src/bytecode_generator/function_dispatch.rs`
2. Verify all function addresses are within bounds
3. Ensure CALL instructions use correct offsets

**Time estimate:** 1-2 hours
**Effort:** Medium (requires bytecode generation debugging)

### Option 2: Investigate Verification Logic
1. Check if verification has incorrect bounds checking
2. Look for off-by-one errors in address validation
3. May be stricter than necessary

**Time estimate:** 30 minutes
**Effort:** Low (just investigation)

### Option 3: Workaround - Simpler Contract
1. Create minimal token.v with single function
2. Deploy and verify it works
3. Gradually add features to identify which causes 8122

**Time estimate:** 30-60 minutes
**Effort:** Medium (process of elimination)

## Technical Details

### Bytecode Structure
```
Offset 0:     Header (magic, flags, function metadata)
Offset 189:   Dispatcher (function routing table)
Offset ~230:  Code section (function implementations)
```

### Function Addressing
- Dispatcher contains PUSH_U16 + JUMP_IF for function selection
- Code section contains CALL instructions to call functions
- All addresses must be < bytecode length

### Verification Point
When final chunk is appended, `verify_bytecode_content()` checks:
- All CALL/CALL_REG targets < bytecode.len()
- All instruction pointers within bounds
- Instruction validity

## Why Tests Can't Run

```
Deploy fails on chunk 2 with 8122
  ↓
Script account doesn't get complete bytecode
  ↓
E2E tests can't execute (account is broken)
  ↓
CU benchmarking blocked
```

## Recommendations

### Immediate (Unblock Testing)
1. **Priority:** Fix bytecode function address generation
2. Debug: Use disassembler to check all function addresses
3. Verify: Compare with working contract (if available)

### Short Term (Fix Register Optimization)
1. Fix inline register optimization that invalidates JUMP positions
2. Implement Option A properly (label position recalculation)
3. Re-enable registers

### Long Term (Robustness)
1. Add compile-time bytecode validation
2. Add pre-deployment verification checks
3. Document bytecode format and constraints

## Critical Files

### To Check
- `five-dsl-compiler/src/bytecode_generator/function_dispatch.rs`
- `five-dsl-compiler/src/bytecode_generator/mod.rs`
- `five-solana/src/instructions/verify.rs`

### To Debug
1. Disassemble token.v bytecode
2. List all CALL instructions and their targets
3. Verify all targets < 805 bytes
4. Compare with working contract

## Next Steps for User

Choose investigation path:

**Path A (Fastest):** Option 3 - Create minimal contract
```bash
# Create minimal token.v with one function
# Deploy and test
# Should work if bytecode generation is OK
```

**Path B (Most Likely to Fix):** Option 1 - Fix bytecode generation
```bash
# Investigate function_dispatch.rs
# Check all function address calculations
# Ensure offsets are correct
```

**Path C (Explore):** Option 2 - Check verification logic
```bash
# Review five-solana verification code
# Verify it's using correct logic
```

---

## Conclusion

The token.v bytecode has invalid function addresses that exceed bytecode bounds. This prevents deployment and blocks all testing. The issue affects both baseline and register-optimized versions equally, indicating it's a core bytecode generation problem, not an optimization issue.

**Status:** Ready to debug - need to investigate function address generation in compiler.
