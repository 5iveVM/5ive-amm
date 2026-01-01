# Known Limitations and TODOs

This document tracks known limitations and correctness issues in the AST generator that require future attention.

## Critical Correctness Issues

### 1. ABI Field Offset Calculation (utilities.rs:165)

**Location**: `utilities.rs:165` - `resolve_field_offset_from_abi()`

**Issue**: Field offsets are calculated using heuristic pattern matching instead of actual memory layout computation.

**Severity**: HIGH - Incorrect offsets can cause memory corruption or wrong data access

**Current Workaround**:
```rust
// Uses temp_resolve_field_offset() which has hardcoded patterns:
"balance" => 0
"owner" => 8
"total_supply" => 16
// etc.
```

**Proper Solution**:
Implement a field layout algorithm that:
1. Respects field ordering in the struct definition
2. Accounts for field type sizes (u64=8, u128=16, pubkey=32, etc.)
3. Handles proper alignment requirements
4. Supports nested structs and arrays
5. Reads actual layout from compiled .five file ABI

**Test Coverage Needed**:
- Test with various struct layouts
- Test with different field orderings
- Test with nested structures
- Test with arrays of different sizes

---

### 2. ABI Function Offset Calculation (utilities.rs:203)

**Location**: `utilities.rs:203` - `resolve_function_offset_from_abi()`

**Issue**: Function bytecode offsets are calculated using heuristic pattern matching instead of actual compiled positions.

**Severity**: HIGH - Incorrect offsets will cause calls to wrong functions or invalid instruction pointers

**Current Workaround**:
```rust
// Uses temp_resolve_function_offset() which has hardcoded patterns:
"transfer" => 0
"approve" => 64
"mint" => 128
// etc.
```

**Proper Solution**:
Store actual bytecode offsets in the .five file ABI during compilation:
1. During compilation, record the bytecode position of each public function
2. Write these offsets to the .five file ABI metadata
3. Read them here for accurate cross-contract calls
4. Validate offset ranges to prevent invalid jumps

**Test Coverage Needed**:
- Test cross-contract function calls
- Test with different function orderings
- Test with varying function sizes
- Verify offset bounds checking

---

### 3. Temporary Field Offset Heuristics (utilities.rs:244, mod.rs:938)

**Location**: Multiple files use `temp_resolve_field_offset()`

**Issue**: Hardcoded field offset mappings that don't reflect actual memory layout.

**Severity**: HIGH - Same as issue #1

**Impact**:
- Duplicated logic in mod.rs (before duplication removal)
- Inconsistent offset calculations
- Will break with custom account types

**Solution**: Same as issue #1 - proper field layout calculation

---

### 4. Temporary Function Offset Heuristics (utilities.rs:267, mod.rs:959)

**Location**: Multiple files use `temp_resolve_function_offset()`

**Issue**: Hardcoded function offset mappings that don't reflect actual bytecode positions.

**Severity**: HIGH - Same as issue #2

**Impact**:
- Duplicated logic in mod.rs (before duplication removal)
- Inconsistent offset calculations
- Will break with custom function sets

**Solution**: Same as issue #2 - store offsets in ABI during compilation

---

## Medium Priority Issues

### 5. VLE Patching Limitation (jumps.rs:68)

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

### 6. V3 Pattern Detection (mod.rs:666)

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
1. **Critical**: Issues #1, #2, #3, #4 - ABI offset calculations
2. **Medium**: Issue #5 - VLE patching
3. **Low**: Issue #6 - Pattern optimization

**Recommended Next Steps**:
1. Create GitHub issues for each critical item
2. Design ABI metadata format to include offsets
3. Implement field layout algorithm
4. Add comprehensive integration tests
5. Consider fuzzing with random struct layouts
