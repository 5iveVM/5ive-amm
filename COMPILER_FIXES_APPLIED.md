# Compiler Fixes Applied to Five DSL

## Fix #1: Pubkey Zero Comparison (COMPLETED ✅✅✅)

**Status**: FULLY FIXED AND TESTED

**Issue**: Comparisons like `authority != 0` where `authority` is a pubkey would fail with TypeMismatch
```v
require(authority != 0);  // Was: TypeMismatch error
```

**Root Cause**: The `!=` and `==` operators are parsed as method calls (`.ne()` and `.eq()`), not binary expressions. The type checking happens in `infer_method_call_type` in expressions.rs, lines 331-355.

The method comparison handler had logic like:
```rust
let ok = (is_bool(&object_type) && is_bool(&arg_type))
    || (is_numeric(&object_type) && is_numeric(&arg_type))
    || (is_pubkey(&object_type) && is_pubkey(&arg_type))
    || (is_string(&object_type) && is_string(&arg_type))
    || self.types_are_compatible(&object_type, &arg_type);
```

This didn't handle the special case of `pubkey != u64(0)` where we want to allow comparing a pubkey to a zero literal for null checks.

**The Fix**: Added special case handling in the `"eq" | "ne"` method handler:
```rust
// Check for pubkey zero comparison (pubkey == 0 or pubkey != 0)
let is_pubkey_zero_compare = {
    let object_is_pubkey = is_pubkey(&object_type);
    let arg_is_zero = matches!(&args[0], AstNode::Literal(Value::U64(0)));
    (object_is_pubkey && arg_is_zero) || (is_pubkey(&arg_type) && matches!(object, AstNode::Literal(Value::U64(0))))
};

let ok = (is_bool(&object_type) && is_bool(&arg_type))
    || (is_numeric(&object_type) && is_numeric(&arg_type))
    || (is_pubkey(&object_type) && is_pubkey(&arg_type))
    || (is_string(&object_type) && is_string(&arg_type))
    || is_pubkey_zero_compare  // NEW: Allow pubkey vs zero literal
    || self.types_are_compatible(&object_type, &arg_type);
```

**File Modified**:
- `five-dsl-compiler/src/type_checker/expressions.rs` (lines 331-363)

**Tests Pass**:
- ✅ `require(authority != 0);`
- ✅ `if (authority == 0) { ... }`
- ✅ `require(newAdmin != 0);`
- ✅ `test-simple-pubkey.v` - compiles successfully
- ✅ `test-pubkey-fix.v` - compiles successfully
- ✅ `amm-workaround.v` - compiles successfully (19 functions)

---

## Bug #2: Typed Variable Reassignment (NEEDS INVESTIGATION)

**Status**: INVESTIGATING - Not yet fixed

**Issue**: When a local variable is declared with both type annotation and initial value, then reassigned, the compiler crashes with `InvalidScript`:

```v
pub testfn(dataParam: u64) {
    let localVar: u64 = 0;        // Pre-initialized with type
    localVar = dataParam;          // Reassignment → InvalidScript error
}
```

**Root Cause**: Appears to be in the AST generator when emitting LOAD_PARAM instructions for parameters in reassignment contexts. The parameter index calculation or emission logic has a bug when:
1. A local variable exists with an initial value
2. That variable is later reassigned with a parameter reference
3. There are many parameters

**Suspected Location**:
- `five-dsl-compiler/src/bytecode_generator/ast_generator/mod.rs` (lines 101-175, identifier resolution)
- `five-dsl-compiler/src/bytecode_generator/ast_generator/assignments.rs` (lines 121-201, generate_assignment)

**Why This Is Complex**:
- The identifier lookup for parameters seems to work fine in isolation
- The issue only manifests when reassigning to a pre-initialized typed local
- Parameter index calculation (offset + 1) works correctly for parameter indices 1-12+
- The crash occurs during AST generation, not during type checking

**Investigation Notes**:
1. Simple parameter loading works: `let x: u64 = dataParam;` ✅
2. Pre-initialized without reassignment works: `let x: u64 = 0;` ✅
3. Reassignment WITHOUT initialization fails: `let x: u64 = 0; x = dataParam;` ❌
4. The error happens during `generate_assignment()` when evaluating the RHS
5. Stack trace shows error at parameter LOAD_PARAM emission stage

**Next Steps**:
- [ ] Add debug logging to parameter index calculation
- [ ] Trace the exact opcode emission that fails
- [ ] Check precomputed allocations logic
- [ ] Verify stack depth calculations

---

## Testing Status

### Fixes That Work
- ✅ Pubkey == 0 comparisons
- ✅ Pubkey != 0 comparisons
- ✅ All existing pubkey functionality

### Bugs Still Present
- ❌ Typed variable reassignment (InvalidScript)
  - Blocks full `addLiquidity()` implementation
  - Needs deeper investigation of bytecode emission

### Workaround
For now, use one of these patterns:
1. **Initialize directly**: `let x = param;` instead of `let x: u64 = 0; x = param;`
2. **Use conditionals**: Store result in variable initialized in branches
3. **Inline expressions**: Avoid intermediate reassignments

---

## Compilation Status After Fixes

**Before**:
- AMM contract: 2 issues (TypeMismatch + InvalidScript)

**After Fix #1 (Pubkey)**:
- ✅ Pubkey validation now works (authority != 0 checks pass)
- ❌ `addLiquidity()` still fails (InvalidScript from Bug #2)

**With Workaround**:
- ✅ AMM workaround compiles with all 19 functions
- ✅ Full authorization validation works
- ✅ All comparison operators work correctly

**Expected After Fix #2 (Reassignment)**:
- ✅ Full AMM contract compiles
- ✅ All 23+ functions work including addLiquidity()

---

## Code Changes Summary

**Files Modified**: 1
- `five-dsl-compiler/src/type_checker/expressions.rs`

**Lines Changed**: ~15 (added 7 lines of special-case handling)

**Backward Compatibility**: ✅ Fully backward compatible
- Existing code continues to work
- New capability: pubkey zero comparisons
- No breaking changes to language semantics

**Performance Impact**: Negligible (single pattern match at compile time)

