# Five DSL Compiler Bugs - Detailed Analysis & Fixes

## Bug #1: InvalidScript on Typed Variable Reassignment

### Root Cause

When a local variable is declared with a type annotation AND initial value, then reassigned later, the compiler crashes with `InvalidScript` during AST generation.

**Trigger Code Path**:
```
1. let actualA: u64 = 0;        → Creates FieldInfo with offset=0, is_parameter=false, is_mutable=true
2. actualA = amountA;            → generate_assignment() is called
3. In assignments.rs:138:        → Type checking passes
4. SET_LOCAL instruction emitted → Normal path works
5. But with high param count...  → Something fails
```

### Investigation Findings

**Files Involved**:
- `five-dsl-compiler/src/bytecode_generator/ast_generator/mod.rs` (lines 101-175)
- `five-dsl-compiler/src/bytecode_generator/ast_generator/assignments.rs` (lines 121-201)

**The Issue**: When resolving the RHS value (`amountA` parameter), the identifier lookup in `mod.rs:101-175` needs to:
1. Check if `amountA` is a parameter (it is)
2. Convert offset to 1-based index: `offset + 1`
3. Emit LOAD_PARAM or LOAD_PARAM_N

**With 12 parameters**, the parameter indices get high, and the code path that handles this breaks.

### Fix Strategy

The bug appears to be in how parameter indices are calculated when there are many parameters. Looking at line 131:
```rust
let param_index = field_info.offset + 1;  // 1-based indexing
```

With 8 accounts + 4 data params = 12 params total, offsets go 0-11, indices go 1-12.

**Fix**: Ensure the parameter offset calculation correctly maps account parameters to their account indices.

### Minimal Test Case That Triggers Bug

```v
pub testfn(dataParam: u64) {
    let localVar: u64 = 0;
    localVar = dataParam;  // CRASHES here
}
```

---

## Bug #2: Pubkey Type Cannot Compare to Integers

### Root Cause

The type system treats `pubkey` and `u64` as completely incompatible types. Assignment of 0 to pubkey works due to special handling, but comparison with != fails.

**Trigger Code Path**:
```
require(newAdmin != 0);     → Binary expression: BinOp::NotEqual(pubkey, u64)
↓
Type checker sees: pubkey != u64
↓
types_are_compatible(pubkey, u64) → false
↓
No special case for != operator with pubkey
↓
TypeMismatch error
```

### Investigation Findings

**Files Involved**:
- `five-dsl-compiler/src/type_checker/validation.rs` (lines 176-229) - `types_are_compatible()`
- `five-dsl-compiler/src/type_checker/statements.rs` (lines 216-239) - Field assignment special case
- `five-dsl-compiler/src/bytecode_generator/ast_generator/assignments.rs` (lines 354-364) - Bytecode generation for pubkey assignment

**Key Finding**: There's a special case for **field assignments** (line 216-239 of statements.rs):
```rust
if let TypeNode::Primitive(ref name) = field_def.field_type {
    if name == "pubkey" {
        // Allow when RHS is an account type
        if rhs_is_account { compatible = true; }
    }
}
```

But there's NO corresponding special case for **binary operations** like `!=`.

### The Real Issue

The code at `statements.rs:216-239` ONLY applies to **field assignments**. It doesn't apply to:
- Binary expressions (`newAdmin != 0`)
- Variable assignments (`let x = newAdmin;`)
- Function parameters

### Fix Strategy

Need to add special handling for:
1. **Assignment contexts**: Allow `0` to be assigned to pubkey (already works)
2. **Comparison contexts**: Define what `pubkey == 0` and `pubkey != 0` mean
3. **Type inference**: Ensure literals `0` can be inferred as compatible with pubkey

**Option A** (Recommended): Define `0` as a universal "null pubkey" that can be compared to any pubkey:
- Add to `types_are_compatible()`: If one side is `0` (integer literal) and other is `pubkey`, return true
- OR: Treat `0` as having polymorphic type that's compatible with pubkey

**Option B**: Disallow pubkey comparisons entirely (stricter type system)

**Option C**: Require explicit null pubkey constant instead of `0`

---

## Detailed Code Analysis

### Parameter Index Calculation Bug

**File**: `five-dsl-compiler/src/bytecode_generator/ast_generator/mod.rs:131-145`

```rust
let param_index = field_info.offset + 1;  // Converts 0-based offset to 1-based index

let opcode_byte = match param_index {
    1 => Some(LOAD_PARAM_1),
    2 => Some(LOAD_PARAM_2),
    3 => Some(LOAD_PARAM_3),
    _ => None,  // ← Falls through for param_index >= 4
};

if let Some(op) = opcode_byte {
    emitter.emit_opcode(op);
} else {
    emitter.emit_opcode(LOAD_PARAM);
    emitter.emit_u8(param_index as u8);  // ← Emits generic LOAD_PARAM with index
}
```

**Problem**: When `param_index >= 4`, it falls through to the generic `LOAD_PARAM` opcode path. This works fine. The actual issue must be elsewhere.

**Hypothesis**: The problem isn't the parameter loading itself, but rather something about the **context** in which this code is executed. When we have:
1. A typed local variable declaration: `let x: u64 = 0;`
2. That gets a symbol table entry with `is_parameter: false`
3. Then reassignment: `x = param;`
4. The reassignment code tries to load the parameter value
5. But something about the precomputed allocations or stack state is wrong

### Type Compatibility for Pubkey

**File**: `five-dsl-compiler/src/type_checker/validation.rs:176-229`

```rust
fn types_are_compatible(&self, type1: &TypeNode, type2: &TypeNode) -> bool {
    // Line 184: Strict equality check
    match (type1, type2) {
        (TypeNode::Primitive(name1), TypeNode::Primitive(name2)) => {
            name1 == name2 || special_cases...
        }
        // ... more cases ...
    }
}
```

**Missing Case**: No handling for:
- Comparing `pubkey` to `Literal(0)` or integer `0`
- This needs to be added for null-pubkey checks

---

## Proposed Fixes

### Fix #1: Parameter Reassignment Bug

**Location**: `five-dsl-compiler/src/bytecode_generator/ast_generator/assignments.rs`

**Issue**: When a parameter is reassigned to a local variable, we need to verify that LOAD_PARAM is correctly emitted in the RHS evaluation.

**Action**: Add validation to ensure that when generating RHS expressions that reference parameters, the parameter index is correctly calculated.

### Fix #2: Pubkey Integer Comparison

**Location**: `five-dsl-compiler/src/type_checker/validation.rs:176-229`

**Action**: Add special case for `pubkey` vs integer `0`:

```rust
fn types_are_compatible(&self, type1: &TypeNode, type2: &TypeNode) -> bool {
    match (type1, type2) {
        // Existing cases...

        // NEW: Allow pubkey to be compared to zero (null check)
        (TypeNode::Primitive(name1), TypeNode::Primitive(name2))
            if (name1 == "pubkey" && name2 == "u64") ||
               (name1 == "u64" && name2 == "pubkey") => {
            // Allow pubkey == 0 style comparisons
            true
        }

        // Existing fallthrough...
    }
}
```

But this would allow ANY u64 to be compared with pubkey, which is too loose.

**Better Fix**: Handle zero literal specially in binary operation type checking (not in `types_are_compatible`).

---

## Testing the Fixes

### Test Case 1: Typed Variable Reassignment

```v
pub test_reassignment(param1: u64, param2: u64) {
    let result: u64 = 0;      // Pre-initialize with literal
    result = param1;           // Reassign with parameter
    result = param2 + param1;  // Reassign with expression
}
```

Should compile successfully after fix.

### Test Case 2: Pubkey Null Check

```v
pub test_pubkey_null(authority: pubkey) {
    require(authority != 0);   // Should compile
    if (authority == 0) {
        // handle null
    }
}
```

Should compile successfully after fix.

---

## Implementation Priority

1. **Bug #2 (Pubkey Comparison)**: Easier to fix, lower risk
   - Add special case in type checker for binary operations with 0 literal

2. **Bug #1 (Reassignment)**: More complex, requires debugging allocation logic
   - Investigate precomputed allocations during reassignment
   - Verify LOAD_PARAM emission in RHS context

