# Known Limitations and TODOs

This document tracks known limitations and correctness issues in the AST generator that require future attention.

## Medium Priority Issues

### 1. BR_EQ_U8 Offset Range Limitation (jumps.rs)

**Location**: `jumps.rs:68` - `patch_br_eq_u8_offset()`

**Issue**: BR_EQ_U8 uses a fixed-width `u16` offset field; patching currently enforces i16-range relative offsets.

**Severity**: MEDIUM - Very large relative jumps can overflow the enforced range and fail patching.

**Current Workaround**: Uses `patch_u16()` with explicit bounds checks before patching.

**Proper Solution**:
Define and enforce canonical BR_EQ_U8 branch semantics end-to-end (signed vs unsigned offset),
then update patching/validation to match VM execution exactly.

**Test Coverage Needed**:
- Test with large bytecode files (> 16KB)
- Test BR_EQ_U8 jumps across large distances
- Verify fixed-width branch offset handling remains consistent

---

## Future Enhancements

### 2. V3 Pattern Detection (mod.rs:666)

**Location**: `mod.rs:666` - `FieldAccess` handling

**Issue**: Bulk field loading optimization not implemented

**Severity**: LOW - Performance enhancement, not correctness issue

**Enhancement**:
Detect consecutive field accesses from the same account and emit BULK_LOAD instructions:
```rust
// Instead of:
LOAD_FIELD account_idx, field1
LOAD_FIELD account_idx, field2
LOAD_FIELD account_idx, field3

// Emit:
BULK_LOAD account_idx, [field1, field2, field3]
```

**Benefits**:
- Reduced bytecode size
- Fewer opcode dispatches
- Better CPU cache utilization

---

## Tracking

All issues are tracked with inline `TODO(correctness)` or `TODO(enhancement)` comments in the source code.

**Priority Order**:
1. **Medium**: Issue #1 - BR_EQ_U8 offset range/semantics
2. **Low**: Issue #2 - Pattern optimization

**Recommended Next Steps**:
1. Create GitHub issues for each item
2. Add comprehensive integration tests
