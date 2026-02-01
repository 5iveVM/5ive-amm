# Bytecode Patching Bug Discovery

**Date:** 2026-01-31
**Status:** CRITICAL - Blocks all token.v testing
**Scope:** Affects both baseline (stack-only) and register-optimized bytecode

## Summary

Discovered a fundamental bytecode patching bug where JUMP instructions are emitted during code generation but not properly recorded for patching. This causes many JUMPs to remain at placeholder/uninitialized values, preventing bytecode execution on-chain (error 8122).

## Evidence

Analyzed baseline (stack-only) bytecode compiled without register optimization:

```
Bytecode size: 766 bytes
Total JUMP instructions found: 122
Out-of-bounds JUMPs: ~85 (70%)
```

### Examples of Unpatched JUMPs

| Offset | Target | Valid? | Issue |
|--------|--------|--------|-------|
| 0x0b   | 0x090e | ❌     | 2318 >> 766 bytes |
| 0xc3   | 0x19dc | ❌     | 6620 >> 766 bytes (appears 12+ times) |
| 0x11e  | 0x9500 | ❌     | 38144 >> 766 bytes (appears 8+ times) |
| 0x1c7  | 0x7fff | ❌     | 32767 >> 766 bytes (uninitialized placeholder) |
| 0x23a  | 0xc148 | ❌     | 49480 >> 766 bytes |
| **0xc6** | **0x227** | **✅** | Valid dispatcher jump |
| **0x270** | **0x270** | **✅** | Valid function jump |

Valid JUMPs are to addresses like 0x200, 0x220, 0x227, 0x270, etc. These are likely function entry points.

Unpatched JUMPs appear to be using placeholder values or corrupted patch records.

## Root Cause Analysis

### Why This Happens

1. **JUMP emission during code generation** - When generating function bodies, the compiler emits JUMP instructions for control flow (loops, conditionals, etc.)

2. **Incomplete patch recording** - Not all emitted JUMPs are recorded in `jump_patches` for later patching
   - File: `five-dsl-compiler/src/bytecode_generator/ast_generator/jumps.rs`
   - Function: `emit_jump()` at line 113

3. **Patching happens once, late** - After AST generation completes
   - File: `five-dsl-compiler/src/bytecode_generator/mod.rs`
   - Line: 544 `ast_generator.patch(self)?;`

4. **Missing JUMPs never get patched** - If a JUMP is emitted but not recorded in jump_patches, it retains its placeholder value (usually 0x19dc or 0x7fff)

### Why Register Optimization Doesn't Help

The temporary workaround I applied (disabling register optimization) doesn't fix this bug because:
- The bug exists in the baseline (stack-only) bytecode
- Register optimization is not the root cause
- The root cause is incomplete patch recording during code generation

## Impact

- **Baseline bytecode:** FAILS bytecode verification (error 8122)
- **Register-optimized bytecode:** FAILS bytecode verification (error 8122)
- **All token.v execution:** BLOCKED until this is fixed
- **E2E test plan:** Cannot proceed

## Next Steps (Priority Order)

### 1. Identify Missing JUMP Patch Recording ⚠️ CRITICAL

Find all code paths in AST generator that emit JUMP instructions but don't record them for patching:

```bash
# Search for emit_jump calls
grep -r "emit_jump" five-dsl-compiler/src/bytecode_generator/

# Search for jump instruction emission (opcode 0x01)
grep -r "emit_opcode.*JUMP" five-dsl-compiler/src/bytecode_generator/

# Look for direct JUMP emission without recording
grep -r "emitter.emit_u8(0x01)" five-dsl-compiler/src/
```

**Likely locations:**
- `ast_generator/control_flow.rs` - For loop and conditional JUMPs
- `ast_generator/statements.rs` - For statement control flow
- `ast_generator/expressions.rs` - For short-circuit evaluation
- `ast_generator/fused_opcodes.rs` - For optimized instruction patterns

### 2. Audit Patch Recording

For each JUMP-emitting code path, verify:
- ✅ JUMP is emitted via `emit_jump()` (records in jump_patches)
- ✅ Label is created via `new_label()` (creates unique label)
- ✅ Label is placed via `place_label()` (records position)
- ✅ Jump target matches placed label

If any JUMP is emitted directly without using `emit_jump()`, that's the bug.

### 3. Verify Label Placement

Ensure every label that's referenced in a JUMP is actually placed:

```rust
// BAD: JUMP emitted but label never placed
self.emit_jump(emitter, JUMP, "loop_start");
// ... code that never calls place_label("loop_start") ...

// GOOD: Label placement matches JUMP
self.emit_jump(emitter, JUMP, "loop_start");
// ... code ...
self.place_label(emitter, "loop_start");
```

### 4. Fix Missing Patch Recording

Once identified, fix all JUMP emission sites to:

```rust
// Create unique label
let loop_start = self.new_label();

// ... emit loop body ...

// Place label at current position
self.place_label(emitter, loop_start.clone());

// Emit JUMP instruction with proper recording
self.emit_jump(emitter, JUMP_OPCODE, loop_start);
```

### 5. Add Verification

Add compile-time checks:

```rust
// After patching, verify all recorded JUMPs were valid
pub fn verify_all_jumps_patched(&self) -> Result<(), VMError> {
    for patch in &self.jump_patches {
        if !self.label_positions.contains_key(&patch.target_label) {
            return Err(VMError::InvalidScript);
        }
    }
    Ok(())
}
```

### 6. Test & Validate

```bash
# Compile baseline
cargo run --bin five -- compile token.v --output token.fbin

# Analyze for unpatched JUMPs
node analyze-jumps.js token.fbin

# Should show: 0 unpatched JUMPs
```

## Files to Investigate

### Primary (Likely culprits)
- `five-dsl-compiler/src/bytecode_generator/ast_generator/control_flow.rs` - Loops, conditionals
- `five-dsl-compiler/src/bytecode_generator/ast_generator/statements.rs` - Statement dispatch
- `five-dsl-compiler/src/bytecode_generator/ast_generator/expressions.rs` - Expression evaluation

### Secondary (May contribute)
- `five-dsl-compiler/src/bytecode_generator/ast_generator/fused_opcodes.rs` - Optimized patterns
- `five-dsl-compiler/src/bytecode_generator/ast_generator/functions.rs` - Function prologue/epilogue
- `five-dsl-compiler/src/bytecode_generator/performance.rs` - Optimization patterns

### Core
- `five-dsl-compiler/src/bytecode_generator/ast_generator/jumps.rs` - Patch infrastructure (lines 113-196)
- `five-dsl-compiler/src/bytecode_generator/mod.rs` - Compilation pipeline (line 544 patch call)

## Current Workaround Status

Applied temporary workaround to disable register optimization:
- ✅ Both ast_generator creation paths disable register features
- ✅ Warning message printed when registers requested
- ❌ **Does NOT fix the underlying patching bug**
- ❌ Baseline bytecode still has unpatched JUMPs

## Conclusion

The bytecode patching bug is more serious than initially understood. It's not caused by register optimization but by incomplete patch recording in the core code generation. The register optimization simply exposed/worsened the bug.

**Priority:** Fix the patch recording before attempting register optimization.

---

## Debug Commands

```bash
# Compile and analyze baseline
cd five-dsl-compiler
cargo run -q --bin five -- compile ../five-templates/token/src/token.v \
  --output ../five-templates/token/build/token-debug.fbin 2>&1 | grep WARNING

# Check for unpatched JUMPs
node analyze-jumps.js ../five-templates/token/build/token-debug.fbin

# Disassemble to see structure
cargo run -q --bin five -- disasm ../five-templates/token/build/token-debug.fbin
```

---

**Next Action:** Investigate control_flow.rs and statements.rs for JUMP emission without proper patch recording.
