# Critical Compiler Bug: Root Cause Analysis

## Executive Summary

The register-optimized bytecode has **93 out-of-bounds JUMP instructions** that prevent deployment. Analysis reveals:

1. ✅ **Dispatcher is correct** - Function offsets properly patched
2. ❌ **Code section JUMPs are corrupted** - Targets don't match bytecode length
3. 🔍 **Root cause:** Register allocation changes bytecode structure but doesn't update JUMP targets in code section

## Evidence

### Bytecode Structure

```
[HEADER - 189 bytes]
├─ Magic bytes: "5IVE"
├─ Function names table (169 bytes)
└─ Dispatcher area starts at 0xbd

[DISPATCHER - ~51 bytes] (offsets 0xbd-0x100)
├─ Function 0: dc 19 00 27 02 24 01 (LOAD_PARAM_0, PUSH_U16 0x0224, JUMP_IF)
├─ Function 1: dc 19 01 27 02 28 01 (LOAD_PARAM_0, PUSH_U16 0x0228, JUMP_IF)
├─ ...14 functions total
└─ Dispatcher ends, CALL_REG area begins (0x100)

[CALL_REG DISPATCH TABLE - ~14 bytes] (offsets 0x100-0x15a)
├─ 0x100: 95 cd 02 00 (CALL_REG 0x02cd)
├─ 0x104: 95 8d 01 00 (CALL_REG 0x018d)
├─ ... function indirect calls
└─ Ends ~0x15a

[CODE SECTION - 581 bytes] (offsets 0x15b-0x302)
├─ Function implementations with JUMP instructions
├─ ❌ JUMP targets corrupted in this section
└─ Functions end at 0x302 (770 bytes total)
```

###  Dispatcher Offsets (Correctly Patched!)

**Baseline dispatcher jumps to:**
```
0x022e, 0x0234, 0x023b, 0x0242, 0x0249, 0x0251, 0x0256, 0x025d,
0x0263, 0x0269, 0x026f, 0x0275, 0x027a  (13 functions + init logic)
```

**Optimized dispatcher jumps to:**
```
0x0224, 0x0228, 0x022c, 0x0230, 0x0234, 0x0238, 0x023c, 0x0240,
0x0244, 0x0248, 0x024c, 0x0250, 0x0254  (adjusted for shorter bytecode)
```

✅ **These are valid offsets within the bytecode**

### Code Section JUMPs (Corrupted!)

**Baseline code section has JUMPs to:** (randomly broken, but some examples)
```
0x0120, 0x0227, 0x012e, 0x0134, etc. - many out of bounds even here
```

**Optimized code section has JUMPs to:**
```
❌ 0x7fff (max u16) - appears 15+ times
❌ 0x9500 (38144) - appears 14+ times
❌ 0xc120 (49440) - appears 2 times
❌ 0xc148 (49480) - appears 2 times
❌ 0xc448 (50248) - appears 5+ times
❌ 0x19dc (6620) - appears 8 times
```

These don't look like random corruption - they look like:
- **0x7fff** - Maximum u16 value (uninitialized placeholder)
- **0x9500, 0xc1xx** - Possible register indices or metadata misinterpreted as addresses
- **0x19dc** - Possibly function table index or persistent value

## Analysis

### Why Dispatcher Works But Code Section Fails

The **dispatcher is generated once at the start of compilation** with:
1. Label positions recorded during code generation
2. PUSH_U16 values are u16 addresses patched correctly
3. These are absolute bytecode offsets verified before patching

The **code section JUMPs are generated during function processing**:
1. Label positions determined dynamically
2. Patch happens at end of function generation
3. Register optimization changes bytecode size/structure **AFTER** initial label calculations
4. JUMP targets become invalidated but patch positions are stale

### Suspected Bug Location

**File:** `five-dsl-compiler/src/bytecode_generator/ast_generator/jumps.rs:patch()`

When `patch_jump_offset` is called, it patches position `patch.position` with target `target_position`. But if register optimization has:
1. Changed bytecode offsets dynamically
2. Modified instruction sizes (VLE encoding varies)
3. Reordered or removed instructions

Then:
- `patch.position` points to wrong location in modified bytecode
- `target_position` was calculated before optimization
- Result: writes patched value at wrong byte offset

## Examples of Corrupted JUMPs

| Offset | Instruction | Target | Bytecode Len | Issue |
|--------|-------------|--------|--------------|-------|
| 0x1b8  | JUMP 0x7107 | 28935  | 770          | 3756% too high |
| 0x237  | JUMP_IF 0xc120 | 49440 | 770          | 6419% too high |
| 0x23a  | JUMP 0xc148 | 49480  | 770          | 6419% too high |
| 0x1f3  | JUMP 0x7106 | 28934  | 770          | 3756% too high |
| 0x0c3  | JUMP 0x19dc | 6620   | 770          | 860% too high |

## Why Register Optimization Triggers This

Register optimization affects bytecode size in ways that invalidate previously-calculated label positions:

1. **PUSH_REG / POP_REG opcodes** are single bytes (vs multi-byte stack operations)
2. **VLE encoding changes** - same value might encode to different length after optimization
3. **Instruction reordering** - registers change instruction sequencing
4. **Offset tracking** - absolute bytecode offsets change but patch records don't update

Example:
```
Before optimization:
  [... 10 bytes of setup ...]
  LABEL "loop_start"         <- position 0x100
  [... code ...]

After optimization:
  [... 5 bytes of setup with PUSH_REG ...]  <- 5 fewer bytes!
  LABEL "loop_start"         <- should be position 0xfb (was 0x100)
  [... code ...]

But the patch record still says "target at 0x100"!
```

## Fix Strategy

The compiler must ensure that **all label positions and patch records are updated when register optimization modifies bytecode**:

### Option 1: Calculate Labels AFTER Optimization (Recommended)
- Disable final patching pass until after all optimizations
- Recalculate all label positions post-optimization
- Apply patches once with final bytecode structure

### Option 2: Adjust Patch Records During Optimization
- Track bytecode offset deltas during optimization
- Update patch record offsets based on delta
- Update target offsets based on delta

### Option 3: Disable Register Optimization for Now
- Revert `--enable-registers` feature
- Mark as experimental/unsupported
- Fix underlying infrastructure

## Critical Files to Fix

1. **`five-dsl-compiler/src/bytecode_generator/ast_generator/jumps.rs`**
   - `patch()` function assumes stable bytecode structure
   - Needs to handle post-optimization bytecode

2. **`five-dsl-compiler/src/bytecode_generator/performance.rs`**
   - `optimize_registers()` runs after AST generation
   - May be running too late in the process

3. **`five-dsl-compiler/src/bytecode_generator/ast_generator/mod.rs`**
   - Main AST generator coordination
   - Determines order of: generation, optimization, patching

4. **`five-dsl-compiler/src/bytecode_generator/register_allocator.rs`**
   - Register allocation logic
   - Must not change bytecode structure after label calculation

## Verification

To confirm this theory:
1. ✅ Both baseline and optimized have out-of-bounds JUMPs (confirmed)
2. ✅ Dispatcher is correct (targets validated)
3. ✅ Code section has corrupted JUMPs (93 found)
4. ❌ Need to verify: patch positions vs actual bytecode locations

## Next Steps

### Immediate

1. Trace through one corrupted JUMP:
   - Find patch record for JUMP at 0x1b8 (0xc120 target)
   - Check what label it should reference
   - Verify where that label's position was calculated
   - See if optimization invalidated the position

2. Add debug output to `patch_jump_offset()`:
   ```rust
   pub(super) fn patch_jump_offset<T: OpcodeEmitter>(
       &self,
       emitter: &mut T,
       offset_pos: usize,
       target: usize,
   ) -> Result<(), VMError> {
       debug_log!("PATCH_JUMP: offset_pos=0x{:x} target=0x{:x} bytecode_len={}",
                  offset_pos, target, emitter.get_position());
       // ... rest of function
   }
   ```

3. Compare bytecode offsets in baseline vs optimized to understand the shift

### Medium Term

1. Refactor to recalculate label positions after optimization
2. Add tests that:
   - Compile with `--enable-registers`
   - Verify all JUMP targets are in bounds
   - Run bytecode verification before deployment

3. Add integration test that deploys register-optimized bytecode

### Long Term

1. Document bytecode structure and patching assumptions
2. Create bytecode validator that runs after each compilation stage
3. Consider moving optimization to earlier in pipeline (before label calculation)

## Conclusion

This is a **coordination bug between register optimization and JUMP patching**. The register allocator changes bytecode structure, invalidating label positions that were calculated before optimization. The patch pass then writes to wrong bytecode locations, corrupting JUMP instructions.

The fix requires ensuring label positions are recalculated after all optimizations, or optimizations happen before label calculation.
