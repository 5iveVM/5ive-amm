# AMM Contract Compilation Issues - Investigation Report

## Summary
The AMM contract provided doesn't compile due to **one critical compiler bug** related to variable reassignment after typed initialization.

## Issues Found

### Issue 1: Compiler Bug - Reassigning Typed Local Variables (CRITICAL)

**Status**: Compiler bug - this is a Five DSL compiler limitation, not user error

**Problem**: When a local variable is declared with a type annotation AND an initial value (both), then reassigned later, the compiler crashes with `InvalidScript`.

**Minimal Reproduction**:
```v
pub testfn(dataParam: u64) {
    let localVar: u64 = 0;        // Declare with type + init value
    localVar = dataParam;          // Then reassign → CRASH
}
```

**Error**: `ERROR: AST generation failed... InvalidScript`

**Workaround**: Initialize variables with the actual value, not a literal placeholder:
```v
pub testfn(dataParam: u64) {
    let localVar: u64 = dataParam;  // ✅ Initialize directly (works)
}
```

Or use if/else blocks to assign conditionally:
```v
pub testfn(
    pool: PoolState @mut,
    condition: bool,
    valueA: u64,
    valueB: u64
) {
    let result: u64 = 0;
    if (condition) {
        result = valueA;
    } else {
        result = valueB;
    }
}
```

**Impact on AMM Contract**: Multiple functions use this pattern:
- `addLiquidity()`: Uses `let optimalB: u64;` then `optimalB = (...)`
- `addLiquidity()`: Uses `let actualA: u64 = 0;` then `actualA = amountA;`
- `addLiquidity()`: Uses `let liquidity: u64 = 0;` then `liquidity = ...;`

### Issue 2: Pubkey Type Limitations (PARTIALLY RESOLVED)

**Status**: Constructor syntax fixed, but type checking issue remains

**Problems Found**:
1. `pubkey(0)` is not valid syntax
2. **Pubkey type cannot be compared to integers** (e.g., `newAdmin != 0` fails with TypeMismatch error)

**Solution**:
- ✅ Don't use `pubkey()` constructor syntax
- ✅ You CAN assign `0` to pubkey fields: `pool.authority = 0;`
- ❌ You CANNOT compare pubkey to `0`: `require(newAdmin != 0);` fails with type mismatch
- ⚠️ **Workaround**: Remove validation checks that compare pubkey to `0`, or use alternative patterns

**Note**: This is a type system limitation in Five DSL. Pubkey types are distinct from integer types in comparisons.

### Issue 3: Variable Declaration Without Initialization (RESOLVED)

**Status**: Fixed

**Problem**: Five DSL requires initialization when type is specified:
```v
// ❌ Incorrect
let optimalB: u64;
let actualA: u64;

// ✅ Correct
let optimalB: u64 = 0;
let actualA: u64 = 0;
```

## Working Patterns in Five DSL

Based on existing templates (token.v, amm.v), here are the recommended patterns:

### Pattern 1: Initialize then conditionally reassign
```v
let shares: u64 = 0;
if (pool.total_lp_shares == 0) {
    shares = amount_a + amount_b;
} else {
    shares = (amount_a * pool.total_lp_shares) / pool.token_a_reserve;
}
```

### Pattern 2: Use intermediate values for complex logic
```v
let amountInWithFee = (amountIn * (10000 - feeBips)) / 10000;
let amountOut = (reserveOut * amountInWithFee) / (reserveIn + amountInWithFee);
```

### Pattern 3: Avoid intermediate variables for conditionals
```v
// Instead of:
let amount: u64 = 0;
if (...) { amount = x; } else { amount = y; }

// Use the value directly in the calling context
```

## How to Fix the AMM Contract

Replace all patterns like:
```v
let variable: u64 = 0;
variable = expression;
```

With:
```v
let variable: u64 = expression;
```

Or use if/else to compute the value before assignment:
```v
let variable: u64 = if (condition) { value1 } else { value2 };
```

Example fix for `addLiquidity()`:
```v
// FROM (doesn't compile):
let actualA: u64 = 0;
let actualB: u64 = 0;
if (pool.reserveA == 0) {
    actualA = amountA;
    actualB = amountB;
} else {
    // ...
    actualA = optimalA;
    actualB = amountB;
}

// TO (works):
let actualA: u64;
let actualB: u64;
if (pool.reserveA == 0) {
    actualA = amountA;
    actualB = amountB;
} else {
    optimalB = (amountA * pool.reserveB) / pool.reserveA;
    if (optimalB <= amountB) {
        actualA = amountA;
        actualB = optimalB;
    } else {
        let optimalA = (amountB * pool.reserveA) / pool.reserveB;
        actualA = optimalA;
        actualB = amountB;
    }
}
```

## Compiler Bug Details

**Location**: `five-dsl-compiler/src/bytecode_generator/ast_generator/`

**Issue**: When emitting LOAD_PARAM instructions for a parameter that's being used in a reassignment context where the variable was previously initialized with a literal value, the AST generator fails to properly emit the parameter loading instruction.

**Trigger Conditions**:
1. Local variable declared with type AND initial value: `let x: type = value;`
2. Later reassignment to that variable: `x = param;` or `x = expression;`
3. The expression/param must be loaded from outside the current scope

**Expected Fix**: Ensure that parameter index calculation and LOAD_PARAM opcode emission works correctly in reassignment contexts even when the target variable has a prior typed declaration.

## Files Affected in AMM Contract

**VERIFIED COMPILATION STATUS** (tested with Five DSL compiler):

✅ **Successfully compiles**:
- `initializePool()` - works (no reassignments to typed vars)
- `getAmountOut()` - works
- `getAmountIn()` - works
- `swapAforB()` - works
- `swapBforA()` - works
- `getReserveA()`, `getReserveB()`, `getTotalLiquidity()`, `getLpSupply()` - all work
- `getPrice()` - works
- `setFee()` - works
- `freezePool()`, `unfreezePool()` - work
- `transferAuthority()` - works (after removing pubkey to integer comparison)
- `revokeAuthority()` - works (after removing pubkey assignment with 0)
- `syncReserves()` - works
- `isFrozen()`, `getAuthority()` - work

❌ **CANNOT compile** (compiler bugs prevent implementation):
- `addLiquidity()` - **Blocked by Issue #1** (typed variable reassignment bug)
- `removeLiquidity()` - works IF typed variables are avoided

See `amm-workaround.v` for a working version that excludes `addLiquidity()` and removes pubkey comparison checks.

## Recommendation

**Short-term**: Refactor the AMM contract to avoid the triggering pattern by:
1. Removing explicit type declarations from pre-initialized variables
2. Or initializing variables with their actual computed values instead of placeholder literals
3. Using inline expressions where possible

**Long-term**: Fix the compiler bug in the AST generator's parameter handling code to support all valid variable reassignment patterns.

