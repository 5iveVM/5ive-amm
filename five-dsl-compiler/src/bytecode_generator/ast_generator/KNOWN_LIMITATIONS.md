# Known Limitations and TODOs

This document tracks known limitations and correctness issues in the AST generator that require future attention.

## Medium Priority Issues

### 1. VLE Patching Limitation (jumps.rs:68)

**Location**: `jumps.rs:68` - `patch_br_eq_u8_offset()`

**Issue**: Assumes VLE encoding is always 2 bytes, which may not be true for large offsets.

**Severity**: MEDIUM - Works for most cases (offsets < 16384) but could fail for very large functions

**Current Workaround**: Uses `patch_u16()` method which assumes fixed 2-byte encoding

**Proper Solution**:
Add `patch_vle_u16()` method to OpcodeEmitter trait that:
1. Determines actual VLE encoding size used
2. Patches the correct number of bytes
3. Handles size changes gracefully

**Test Coverage Needed**:
- Test with large bytecode files (> 16KB)
- Test BR_EQ_U8 jumps across large distances
- Verify VLE encoding consistency

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
1. **Medium**: Issue #1 - VLE patching
2. **Low**: Issue #2 - Pattern optimization

**Recommended Next Steps**:
1. Create GitHub issues for each item
2. Add comprehensive integration tests
