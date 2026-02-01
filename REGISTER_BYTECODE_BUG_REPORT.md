# Critical Bug Report: Register-Optimized Bytecode Corruption

**Date:** 2026-01-30
**Status:** BLOCKING
**Severity:** CRITICAL

## Summary

The register-optimized token bytecode contains invalid JUMP instructions that target out-of-bounds addresses, causing deployment failures with error code 8122 (CallTargetOutOfBounds).

## Symptoms

- **Error Code:** 8122 (CallTargetOutOfBounds)
- **When:** During final bytecode verification on-chain
- **Impact:** Deployment fails for all register-optimized bytecode
- **Workaround:** None - bytecode is invalid

## Root Cause: Compiler Bug

The register allocation or bytecode generation is corrupting JUMP instruction arguments.

### Invalid JUMP Targets Found

**Bytecode size:** 770 bytes (0x302)

**Invalid JUMP instructions:**
| Offset | Instruction | Target | Issue |
|--------|-------------|--------|-------|
| 0x1b8 | JUMP | 28935 | Out of bounds (3769% of bytecode) |
| 0x1f3 | JUMP | 28934 | Out of bounds |
| 0x202 | JUMP | 28934 | Out of bounds |
| 0x237 | JUMP_IF | 49440 | Out of bounds (6419% of bytecode) |
| 0x23a | JUMP | 49480 | Out of bounds |
| 0x23d | JUMP_IF | 50248 | Out of bounds |
| 0x02b2 | JUMP | 32767 | Out of bounds (4256% of bytecode) |

### Hex Evidence

```
000001b0  01 51 1d 01 42 01 71 57  01 07 71 02 70 03 c5 01  |.Q..B.qW..q.p...|
000001b8  ^ JUMP instruction here (0x01), but target decoding corrupted
```

The JUMP opcode is correct (0x01), but the target address encoding (VLE) is producing impossible values.

## Bytecode Comparison

**Baseline (working):**
- Size: 805 bytes
- Compilation: Without --enable-registers
- Status: Deploys successfully (though hitting infrastructure error later)
- JUMP targets: All valid

**Register-Optimized (broken):**
- Size: 770 bytes (35 bytes smaller)
- Compilation: With --enable-registers --use-linear-scan
- Status: Fails bytecode verification
- JUMP targets: CORRUPTED

**Difference:**
- First difference at offset 0xc9
- Register allocator modifies instruction sequences
- Something in register allocation corrupts JUMP argument encoding

## Compiler Code Paths Affected

**Likely locations:**
1. `five-dsl-compiler/src/bytecode_generator/register_allocator.rs` - Register allocation logic
2. `five-dsl-compiler/src/bytecode_generator/linear_scan_allocator.rs` - Linear scan implementation
3. `five-dsl-compiler/src/bytecode_generator/ast_generator/*.rs` - Bytecode emission for JUMP instructions with register optimization

**Suspected issue:**
- JUMP instruction argument is being modified by register allocator
- VLE encoding of new value is corrupted
- Or instruction offset tracking is wrong after register optimization

## On-Chain Verification Flow

The Five VM program correctly rejects the bytecode:

**File:** `five-solana/src/instructions/deploy.rs` (line 361-378)
```rust
if new_len == expected_size {
    // Bytecode complete - verify before finalization
    if let Err(e) = verify_bytecode_content(bytecode) {
        // Error 8122: CallTargetOutOfBounds
        return Err(e);
    }
}
```

**Verification logic:** `five-solana/src/instructions/verify.rs` (line 81-87)
```rust
if inst.opcode == opcodes::CALL || inst.opcode == opcodes::CALL_REG {
    let func_addr = inst.arg1 as usize;
    if func_addr >= bytecode.len() {
        return Err(ProgramError::Custom(8122)); // ← Error returned here
    }
}
```

The bytecode is genuinely invalid, not a verification issue.

## Next Steps

### Investigation Required

1. **Check JUMP instruction generation:**
   - Find where JUMP instructions are emitted in AST generator
   - Add debug logging for JUMP target calculations

2. **Check register allocator modifications:**
   - Review how register allocator modifies instruction arguments
   - Verify VLE encoding is correct after modification

3. **Verify offset tracking:**
   - Ensure instruction offset calculations are correct
   - Check if bytecode offsets change during register optimization

4. **Test individual functions:**
   - Compile individual functions with registers
   - Identify which function has corrupted JUMPs
   - Narrow down the register allocation causing the issue

### Debugging Commands

```bash
# Compile with debug output
cargo run --bin five -- compile \
  ../five-templates/token/src/token.v \
  --enable-registers \
  --use-linear-scan \
  --output ../five-templates/token/build/token-registers.fbin \
  --verbose

# Compare bytecodes
hexdump -C five-templates/token/build/token-registers.fbin > registers.hex
hexdump -C <baseline.fbin> > baseline.hex
diff -u baseline.hex registers.hex

# Test simpler contract
cargo run --bin five -- compile \
  five-templates/counter/src/counter.v \
  --enable-registers \
  --use-linear-scan

# Inspect specific function
cargo run --bin five -- inspect <bytecode> --disasm | grep -A5 "JUMP"
```

## Impact Assessment

**Current Status:**
- ✅ Register compilation works syntactically
- ✅ Register allocation produces bytecode (35 bytes smaller)
- ❌ Bytecode is semantically invalid
- ❌ Cannot deploy or execute on-chain

**Blocking:**
- All Phase 2 deployments
- All Phase 3 E2E tests
- Cannot measure real on-chain CU savings

**Alternative:**
- Revert to baseline (stack-only) for testing
- Or fix this compiler bug first

## Related Code

- Compiler entry: `five-dsl-compiler/src/bin/five.rs`
- Register allocator: `five-dsl-compiler/src/bytecode_generator/register_allocator.rs`
- Linear scan: `five-dsl-compiler/src/bytecode_generator/linear_scan_allocator.rs`
- JUMP generation: `five-dsl-compiler/src/bytecode_generator/ast_generator/expressions.rs`
- Verification: `five-solana/src/instructions/verify.rs` (lines 81-102)

##Conclusion

**The register-optimized bytecode generation has a critical bug that corrupts JUMP instruction arguments.** This must be fixed in the compiler before register-optimized bytecode can be deployed or tested on-chain.

The bug appears to be in how the register allocator modifies or tracks JUMP instruction offsets after register allocation changes bytecode structure.
