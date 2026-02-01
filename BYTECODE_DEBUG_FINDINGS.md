# Bytecode Patching Investigation - Findings

**Date:** 2026-01-31
**Status:** Investigation Complete - Findings Documented

## Executive Summary

Investigated why baseline (stack-only) token.v bytecode fails verification with error 8122. Found that the issue is more nuanced than originally thought:

1. ✅ Temporary register optimization workaround is in place (both ast_generator paths)
2. ✅ Baseline bytecode compiles successfully
3. ⚠️ Baseline bytecode still fails with error 8122 on-chain
4. 🔍 Investigation revealed unexpected bytecode structure

## Key Findings

### Finding 1: Very Few JUMP Patches Recorded

```
patch() called with:
  - 2 jump_patches total (for entire 14-function token.v)
  - 4 label_positions (L0, L1, L2, L3)
  - Only 2 out of 4 labels actually used in patches (L0, L2)
  - 0 loop-based break/continue patches (token.v has no loops)
```

### Finding 2: Token.v Has No Control Flow Loops

```bash
$ grep -n "while\|loop\|for\|break\|continue" token.v
# (no results)
```

This explains why there are no loop-based JUMP patches.

### Finding 3: Bytecode Contains Many 0x01 Bytes

Earlier bytecode analysis found 122 instances of opcode 0x01 throughout the 766-byte bytecode. However:

- Opcode 0x01 = JUMP instruction
- But these 0x01 bytes might **not all be JUMP opcodes**
- They could be:
  - VLE-encoded parameter values
  - Data bytes (coincidentally 0x01)
  - Parts of multi-byte instructions

Only 2 JUMP instructions are explicitly recorded for patching, suggesting most 0x01 bytes are NOT JUMP opcodes.

### Finding 4: Bytecode Verification Error (8122)

Error 8122 = `CallTargetOutOfBounds` from five-solana/src/instructions/verify.rs

Occurs when:
- Instruction pointer >= bytecode.len()
- CALL/CALL_REG target >= bytecode.len()

The error happens during bytecode verification **before execution**, preventing deployment.

## Root Cause Hypothesis

The baseline bytecode fails verification not because of JUMP patching issues (since only 2 are recorded), but potentially:

1. **Other instruction types failing bounds checks** - Not just JUMPs, but CALL or CALL_REG instructions
2. **Invalid function addresses** - Function pointers in dispatch table might be out-of-bounds
3. **Bytecode structure mismatch** - Expected vs actual bytecode layout differences

## Current Status

### ✅ Completed
- Disabled register optimization in both code paths
- Added compiler warning when registers requested
- Baseline bytecode compiles successfully (766 bytes)
- Bytecode artifact created for testing
- Comprehensive debug logging added

### ⚠️ Blocked
- Cannot deploy baseline bytecode (fails error 8122)
- Cannot test token.v functions without working bytecode
- Cannot measure actual CU improvements

### 📋 Next Steps

**Option A: Investigate Function Pointers** (Recommended)
- Check if dispatcher function addresses are within bytecode bounds
- Verify function entry points are correct
- May be issue with five-solana verify.rs hardcoded limits

**Option B: Simplify Test Case**
- Create minimal Five DSL contract with no loops, no complexity
- Compile and verify it deploys successfully
- Incrementally add features until error appears
- Identifies which language feature causes 8122

**Option C: Compare with Known Working Bytecode**
- Ensure deployment infrastructure works with different bytecode
- Test if counter.v or other templates deploy successfully
- Validates that error isn't environmental

## Technical Details

### Bytecode Structure (verified)

```
[HEADER - ~189 bytes]
├─ Magic: "5IVE"
├─ Flags: 0xf
├─ Function table

[DISPATCHER - ~51 bytes]
├─ 14 LOAD_PARAM_0 + PUSH_U16 + JUMP_IF sequences
├─ Target addresses: 0x0220-0x0254 (within 766-byte file)

[CODE SECTION - ~531 bytes]
├─ Function implementations
└─ Correct size (no overflow)
```

### Patch Recording Analysis

```rust
// Only paths that record patches:
1. emit_jump() in expressions.rs (used in conditionals)
2. loop break/continue patching (not used in token.v)
3. Dispatcher patching (function_patches, not jump_patches)

// Results for token.v:
- 2 regular jump_patches
- 0 break/continue patches
- Dispatcher correctly patched separately
```

## Files Modified

### Workarounds Applied
- `five-dsl-compiler/src/bytecode_generator/mod.rs` (2 locations)
  - Lines 471-488: Disable registers in first ast_generator path
  - Lines 507-523: Disable registers in second ast_generator path

### Debug Code Added (cleanup pending)
- `five-dsl-compiler/src/bytecode_generator/ast_generator/jumps.rs` - Added debug logging to patch()
- `five-dsl-compiler/src/bytecode_generator/ast_generator/control_flow.rs` - Added debug logging to loop patching

## Recommendations

### Short Term (Fix Immediate Issue)
1. **Run Option B** - Create minimal test contract
   ```bash
   # Create test.v with single function, no loops
   # Compile and verify it works
   # Then incrementally add features
   ```

2. **Investigate dispatcher addresses**
   - Check five-solana verify.rs to understand address validation
   - Verify dispatcher jump targets are definitely in-bounds

3. **Test deployment infrastructure**
   - Confirm counter.v or other known-working contracts deploy
   - Rules out environment/configuration issues

### Medium Term (Proper Fix)
1. Fix root cause (once identified)
2. Remove temporary register optimization workaround
3. Implement proper Option A: label position recalculation
4. Add bytecode verification tests

### Long Term (Robustness)
1. Add compile-time bytecode validation
2. Check all offsets before deployment attempt
3. Improve error messages for bounds violations
4. Document bytecode layout guarantees

## Next Actions

**User should choose which investigation path to follow:**

- **Path A**: Fix register optimization properly (Option A implementation)
  - Time: ~2-3 hours
  - Risk: Low (well-understood approach)
  - Benefit: Enables full feature set

- **Path B**: Debug deployment blockers
  - Time: ~1-2 hours
  - Risk: Medium (unknown root cause)
  - Benefit: Unblocks immediate testing

- **Path C**: Defer and test with known-working contract
  - Time: ~30 minutes
  - Risk: Very Low
  - Benefit: Validates infrastructure works

---

**Status:** Awaiting user guidance on next investigation direction.
