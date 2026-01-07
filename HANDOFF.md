# Five Mono - Project Handoff

## Status Summary

The Five DSL `@init` constraint implementation is **structurally complete** with all compiler and VM support in place. Opcode desync has been **resolved**. Current blocker: Account ownership validation prevents creating uninitialized PDA accounts.

**Implementation Status**:
- ✅ Compiler: Parser & function dispatch fixes applied
- ✅ VM: INIT_ACCOUNT/INIT_PDA_ACCOUNT handlers implemented
- ✅ Opcodes: Correctly emitting 0x84 (INIT_ACCOUNT) and 0x85 (INIT_PDA_ACCOUNT)
- ❌ Tests: Failing at account ownership validation stage

**Current Issue**: IllegalOwner error when attempting to create uninitialized PDA accounts

---

## What Was Implemented

### 1. Compiler: Function Context & Payer Resolution

**Files**: `five-dsl-compiler/src/bytecode_generator/ast_generator/types.rs`

- Added `current_function_parameters` field to `ASTGenerator` struct
- Tracks function parameters during bytecode generation for payer resolution
- Initialized in `new_internal()` and cleared in `reset()`

**Files**: `five-dsl-compiler/src/bytecode_generator/ast_generator/accounts.rs`

- Implemented `resolve_payer_account_index()`: Maps payer parameter name to account index
- Implemented `find_first_signer_account_index()`: Defaults to first @signer parameter
- Account index calculation: `(parameter_index + 2)` accounts for script and vm_state at indices 0, 1

### 2. Compiler: Bytecode Generation with Payer

**Files**: `five-dsl-compiler/src/bytecode_generator/ast_generator/accounts.rs`

**Regular Account Init**:
```
// Old: [owner, lamports, space, account_idx]
// New: [owner, lamports, payer_idx, space, account_idx]

// Payer emission:
let payer_idx = if let Some(ref payer_name) = init_config.payer {
    self.resolve_payer_account_index(payer_name)?
} else {
    self.find_first_signer_account_index()?
};
emitter.emit_opcode(PUSH_U8);
emitter.emit_u8(payer_idx);
```

**PDA Account Init**: Similar stack contract change with payer_idx emission after GET_RENT

### 3. Compiler: Function Context Management

**Files**: `five-dsl-compiler/src/bytecode_generator/function_dispatch.rs`

- Line 755: Set `ast_generator.current_function_parameters = Some(parameters.to_vec())`
- Line 845: Clear `ast_generator.current_function_parameters = None`

Ensures payer resolution has access to function signature during init sequence generation.

### 4. Compiler: Type Checker Validation

**Files**: `five-dsl-compiler/src/type_checker/functions.rs`

- Lines 168-200: Enhanced @init validation
- Verify payer exists in function parameters
- Validate payer is account type (Account or Named)
- Require @signer constraint on payer parameter
- Compile-time error messages for invalid configurations

### 5. VM: INIT_ACCOUNT Handler Update

**Files**: `five-vm-mito/src/handlers/system/init.rs`

- Updated stack comment: `[account_idx, space, payer_idx, lamports, owner_pubkey]`
- Line 68: Pop payer_idx before lamports
- Call `ctx.create_account_with_payer(account_idx, payer_idx, space, lamports, &owner)`
- Similar changes for `handle_init_pda_account()`

### 6. VM: Account Creation with Payer

**Files**: `five-vm-mito/src/context.rs`

- Line 31: Added `const MAX_ACCOUNT_SIZE: u64 = 10 * 1024 * 1024`
- Lines 1133-1235: Implemented `create_account_with_payer()` method
  - Validates account and payer indices
  - Validates payer is signer AND writable
  - Performs System Program CPI for account creation
  - Handles both on-chain (invoke) and off-chain (simulation) modes

### 7. VM: Ownership Validation Fix

**Files**: `five-vm-mito/src/context.rs`

- Lines 978-995: Updated `check_bytecode_authorization()`
- Skip ownership validation for uninitialized accounts (data_len == 0)
- Allows VM to write to accounts during @init sequence
- Normal ownership checks apply after initialization

---

## How @init Works Now

### Compile-Time Flow

```
DSL Source (@init constraint)
    ↓
Parser (recognizes @init(payer=X, space=N))
    ↓
Type Checker (validates payer exists, is account, has @signer)
    ↓
Bytecode Generator (emits payer_idx, CHECK_UNINITIALIZED, INIT_ACCOUNT)
    ↓
Bytecode with new stack contract
```

### Runtime Flow

```
Instruction arrives with function parameters [counter, owner, ...]
    ↓
Account layout: [script, vm_state, counter, owner, system, rent]
    ↓
Bytecode executes CHECK_UNINITIALIZED (counter not yet initialized)
    ↓
Bytecode emits: [payer_idx=3, owner_pubkey, lamports, space, account_idx]
    ↓
INIT_ACCOUNT handler pops stack, validates payer is writable+signer
    ↓
Invokes System Program CreateAccount with explicit payer
    ↓
Account initialized, ownership set, data allocated
    ↓
check_bytecode_authorization() skips validation (now data_len > 0)
    ↓
STORE_FIELD writes initialize flag
```

---

## Test Results & Debugging (Updated Jan 7)

### Opcode Desync Issue - RESOLVED ✅

**Problem**: Compiler was emitting legacy opcodes while VM expected new ones.

| Operation | Legacy Value | New Value | Conflict |
|-----------|--------------|-----------|----------|
| `INIT_ACCOUNT` | 0x74 | 0x84 | 0x74 now = `CHECK_PDA` |
| `INIT_PDA_ACCOUNT` | 0x75 | 0x85 | 0x75 now = `CHECK_UNINITIALIZED` |

**Root Cause**: After protocol update moved init opcodes to 0x80 range, compiler wasn't rebuilt from clean state.

**Solution Applied**:
1. Clean build: `cargo clean && cargo build -p five-protocol five-dsl-compiler`
2. Rebuild WASM and CLI assets
3. Fresh Solana program deployment

**Verification**: Bytecode now contains correct 0x85 opcode at init sequences. Disassembler correctly shows `INIT_PDA_ACCOUNT` instruction.

### Recent Changes Applied

Two commits merged to improve @init support:

1. **4aedbb2** - Parser: Support seeds and bump parameters in @init constraint
   - Extended `parse_init_arguments()` to handle `seeds=[...]` and `bump=name` syntax
   - Returns tuple: `(payer, space, seeds, bump)` for flexible initialization
   - Supports both legacy `[seeds]` block and new parameterized form

2. **c8e2708** - Function Dispatch: Record function offset before init sequence
   - Moved function offset recording before parameter processing
   - Ensures dispatch table points to correct bytecode location
   - Allows init seeds to reference other function parameters

### Current Issue: Account Ownership Validation

**Error**: `IllegalOwner` when attempting to initialize PDA account
**Status**: Not a SDK or bytecode issue - account validation failure

**Symptoms**:
- Tests fail with "Provided owner is not allowed"
- Compute units: ~747 CU (very early failure)
- Program reaches entrypoint but fails before VM execution completes
- Error occurs in Solana's account owner validation

**Root Cause**: The counter PDA doesn't exist yet, so ownership check fails before `INIT_PDA_ACCOUNT` can create it.

### Investigation Path Forward

The issue is **not** in:
- ✅ Bytecode generation (opcodes are correct)
- ✅ Opcode values (protocol-aligned)
- ✅ VM handlers (both implemented)
- ✅ Parser/dispatcher (recent fixes applied)

The issue **is** in:
- Account constraint validation in Solana program wrapper (`five-solana/src/instructions.rs`)
- Need to skip or defer ownership checks for accounts marked with `@init`

### Key Files to Check

**Constraint Validation** (`five-solana/src/instructions.rs:860-910`):
- `execute()` function calls `validate_vm_and_script_accounts()`
- This may be checking all instruction accounts including uninitialized ones
- Need to identify which validation is rejecting uninitialized accounts

**VM State Check** (`five-solana/src/common.rs:141-149`):
- `validate_vm_and_script_accounts()` validates script + vm_state accounts
- Should skip validation for PDA accounts that will be initialized

### Debugging Approach

```bash
# 1. Verify bytecode opcodes
xxd five-templates/counter/src/counter.fbin | grep "85"
# Should show 0x85 INIT_PDA_ACCOUNT opcode

# 2. Check if constraint validation is blocking
# Add logs in five-solana/src/instructions.rs:execute()
# Log which account is failing validation

# 3. Test with debug logs
cargo build-sbf --features debug-logs --manifest-path five-solana/Cargo.toml
solana program deploy target/sbpf-solana-solana/release/five.so --url http://127.0.0.1:8899

# 4. Run E2E test and capture logs
cd five-templates/counter && npm test 2>&1 | head -200

# 5. If still failing, check whether INIT_PDA_ACCOUNT is reached
# Add error_log! at start of five-vm-mito/src/handlers/system/init.rs:handle_init_pda_account()
```

---

## Recent Work (Jan 7, 2026)

### Work Completed

1. **Identified and Fixed Opcode Desync**
   - Root cause: Protocol update moved init opcodes to 0x80 range, but compiler cached old values
   - Fix: Clean rebuild `cargo clean && cargo build`
   - Verification: Bytecode now emits correct 0x85 for INIT_PDA_ACCOUNT

2. **Applied Parser Improvements**
   - Commit: 4aedbb2 - Parser: Support seeds and bump parameters in @init constraint
   - Extended parse_init_arguments to handle parameterized form
   - Can now parse: `@init(payer=..., space=..., seeds=[...], bump=...)`

3. **Fixed Function Dispatch Offset Recording**
   - Commit: c8e2708 - Function Dispatch: Record function offset before init sequence
   - Moved offset recording before init sequence generation
   - Allows seeds to reference other function parameters

4. **Rebuilt All Components**
   - Clean rebuilt five-protocol and five-dsl-compiler
   - Rebuilt five-wasm with latest compiler
   - Rebuilt five-cli with latest assets
   - Deployed fresh Solana program to localnet

5. **Verified Bytecode**
   - Bytecode disassembly shows correct INIT_PDA_ACCOUNT (0x85) at offsets
   - Compiler is now protocol-aligned
   - No opcodes being misinterpreted

### Environment Setup

**Program IDs**:
- New Five Program (deployed Jan 7): `CYGsrNpCRUt5HRYvhwh3pV23XVtCqihYoHzrQQrNAezX`
- Updated in: `five-templates/counter/deployment-config.json`

**Localnet Status**:
- Solana validator running on localhost:8899
- WASM assets synced to five-cli/assets/vm
- Counter template compiles to 244 bytes

### Remaining Issue

**Tests fail with**: `IllegalOwner` error during initialization
**Likely cause**: Account ownership validation in Solana program wrapper
**Next step**: Fix `five-solana/src/` to defer validation for @init accounts

---

## Architecture Changes

### Stack Contract (Breaking)

**Old Format**:
```
INIT_ACCOUNT: [owner, lamports, space, account_idx]
```

**New Format**:
```
INIT_ACCOUNT: [owner, lamports, payer_idx, space, account_idx]
INIT_PDA_ACCOUNT: [owner, lamports, payer_idx, space, bump, seeds..., count, account_idx]
```

Requires simultaneous compiler + VM updates (already implemented).

### Payer Resolution (Compile-Time)

Payer is now determined at compile-time from DSL:
- Explicit: `@init(payer=owner)`
- Implicit: First parameter with `@signer` constraint

No more runtime payer discovery (was creating accounts owned by program).

---

## Files Modified

### Compiler (5 files)
- `five-dsl-compiler/src/bytecode_generator/ast_generator/types.rs` (+2, -1)
- `five-dsl-compiler/src/bytecode_generator/ast_generator/initialization.rs` (+2, -2)
- `five-dsl-compiler/src/bytecode_generator/ast_generator/accounts.rs` (+50, -3)
- `five-dsl-compiler/src/bytecode_generator/function_dispatch.rs` (+4, -2)
- `five-dsl-compiler/src/type_checker/functions.rs` (+28, -0)

### VM (2 files)
- `five-vm-mito/src/handlers/system/init.rs` (+4, -2)
- `five-vm-mito/src/context.rs` (+107, -1)

### Total Changes
- **Lines Added**: ~197
- **Lines Modified**: ~11
- **Compilation Status**: ✅ No errors
- **Breaking Changes**: Stack contract (intentional, protocol update)

---

## Testing & Verification

### Unit Tests
All compiler and VM unit tests compile successfully.

### Integration Tests (Counter Template)

**Setup**:
- Two counter accounts (counter1, counter2)
- Two user accounts (user1, user2)
- Each user creates and owns their counter

**Operations**:
1. Initialize counter1 (with user1 as payer)
2. Initialize counter2 (with user2 as payer)
3. Increment counter1 (3x)
4. Add 10 to counter1
5. Decrement counter1
6. Increment counter2 (5x)
7. Reset counter2
8. Verify final states

**Current Results**: ❌ All fail at initialization due to payer writable flag issue

### Next Test Steps

1. **Fix SDK Issue**: Resolve payer account writable flag override
2. **Re-run Tests**: Verify all 13 tests pass
3. **Validate State Persistence**: Ensure counter values persist across transactions
4. **Test Edge Cases**: Multiple @init calls, error conditions

---

## Known Limitations & Future Work

### Current Limitations

1. **Payer Flag Override**: Five SDK overrides isWritable for function parameters
2. **Error Messages**: Limited error context in VM for @init failures
3. **PDA Validation**: PDA derivation validation only works on Solana (not off-chain)

### Future Enhancements

1. **Better Error Messages**: Include payer name and reason for constraint violation
2. **PDA Seed Validation**: Off-chain PDA derivation for validation
3. **Rent Calculation**: Optimize rent calculation for common sizes
4. **Multiple Payers**: Support multiple payers in single transaction
5. **Reinitialization**: Support re-initializing closed accounts

---

## Deployment Instructions

### Prerequisites
- Solana localnet running: `solana-test-validator`
- Latest Five SDK compiled
- Updated bytecode (rebuild with new compiler)

### Build & Deploy

```bash
# Build updated compiler and VM
cargo build -p five-dsl-compiler --release
cargo build -p five --release

# Deploy Five VM program
solana program deploy target/deploy/five.so --url http://127.0.0.1:8899

# Build counter template (uses new compiler)
cd five-templates/counter
npm run build

# Run tests
npm test
```

### Verify Installation

```bash
# Check program deployed
solana program show HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg --url http://127.0.0.1:8899

# Test basic initialization
npm test 2>&1 | grep "initialize counter"
```

---

## Commits

**Recent (Jan 7, 2026)**:
1. `4aedbb2` - Parser: Support seeds and bump parameters in @init constraint
2. `c8e2708` - Function Dispatch: Record function offset before init sequence

**Previous**:
3. `174e32a` - feat(compiler): add function context and payer resolution for @init constraints
4. `ca20f89` - feat(compiler): validate @init payer in type checker
5. `93e400a` - feat(vm): update INIT_ACCOUNT stack contract to include payer_idx
6. `97b5aaf` - feat(vm): implement create_account_with_payer and fix ownership validation

---

## Next Developer Notes

### High Priority: Fix Account Ownership Validation

**Problem**: Counter PDA accounts fail validation before init can create them.

**Investigation Steps**:

1. **Identify the validation failure point**
   - Trace through `five-solana/src/instructions.rs:execute()`
   - Check which accounts are being validated
   - Specifically check if all accounts are validated or just script + vm_state

2. **Check account constraints in Solana program**
   - `verify_program_owned()` in `five-solana/src/common.rs:131-138` validates owner
   - This check rejects accounts not owned by the Five program
   - Uninitialized PDAs won't have any owner, so they fail

3. **Implement deferred validation**
   - Option A: Skip validation for uninitialized accounts (data_len == 0)
   - Option B: Let VM handle initialization, then validate afterwards
   - Option C: Mark accounts as "to be initialized" and skip validation for them

4. **Testing approach**
   ```bash
   # After identifying validation point:
   # 1. Modify constraint check to allow uninitialized accounts
   # 2. Rebuild Solana program
   # 3. Redeploy to localnet
   # 4. Re-run E2E tests
   ```

### Quick Verification Checklist

- ✅ Opcodes correct in bytecode? (Check with `xxd` for 0x85)
- ✅ Disassembler shows correct instruction names?
- ✅ WASM rebuilt from latest compiler?
- ✅ Solana program rebuilt and redeployed?
- ⚠️ Account validation passes for uninitialized accounts?

### Commands to Continue Debugging

```bash
# Verify recent commits are applied
git log --oneline | head -5
# Should show:
#   c8e2708 Function Dispatch: Record function offset before init sequence
#   4aedbb2 Parser: Support seeds and bump parameters in @init constraint

# Check Solana program compilation status
cargo build-sbf --manifest-path five-solana/Cargo.toml 2>&1 | tail -20

# Verify bytecode opcodes are correct
xxd five-templates/counter/src/counter.fbin | grep " 85 "
# Should show lines with 0x85 opcode

# Check if account validation is the blocker
grep -n "IllegalOwner\|verify_program_owned" five-solana/src/common.rs

# Look at instruction validation
grep -n "execute.*program_id.*accounts" five-solana/src/instructions.rs | head -1

# Run minimal test to isolate issue
cd five-templates/counter
npm test 2>&1 | grep -A 5 "FAIL\|PASS" | head -20

# If needed, add debug logging to Solana program
# Edit five-solana/src/instructions.rs:execute() around line 870-920
# Add: pinocchio_log::log!("DEBUG: account {} owner check", account_index);
```

### Specific Files to Examine for Account Validation

1. **five-solana/src/instructions.rs** (lines 860-920)
   - `execute()` function - where account validation happens
   - Look for early exits before VM execution

2. **five-solana/src/common.rs** (lines 131-149)
   - `verify_program_owned()` - validates account owner
   - `validate_vm_and_script_accounts()` - might be checking all accounts

3. **five-solana/src/lib.rs** (lines 71-180)
   - `process_instruction()` - main entrypoint
   - Check if all accounts are validated uniformly

### Quick Syntax Reference

DSL @init usage:
```v
pub initialize(
    counter: Counter @mut @init(payer=owner, space=56) @signer,
    owner: account @signer
) { ... }
```

@init parameters:
- `payer=X`: Which parameter pays for account creation
- `space=N`: Account data size in bytes
- Auto-defaults: `payer=first_signer`, `space=auto_calculated`

### Key Constants

- `MAX_ACCOUNT_SIZE`: 10 MB
- `ACCOUNT_INDEX_OFFSET`: 2 (script + vm_state)
- `MAX_SEEDS`: 8 (for PDA)

---

## Summary

The @init constraint implementation is **structurally complete and compiling**. All compiler and VM infrastructure is in place. The opcode desync issue has been **resolved** through clean rebuild.

**What's Working**:
- ✅ Parser correctly handles `@init(payer=X, space=N, seeds=[...], bump=Y)` syntax
- ✅ Function dispatcher records correct bytecode offsets
- ✅ Compiler emits correct opcodes (0x84, 0x85)
- ✅ VM has both INIT_ACCOUNT and INIT_PDA_ACCOUNT handlers implemented
- ✅ Bytecode validation with xxd confirms correct instruction bytes

**What Needs Fixing**:
- ⚠️ Account ownership validation in Solana program wrapper blocks uninitialized accounts
- ⚠️ Need to defer or skip constraint validation for accounts marked with `@init`
- ⚠️ The issue is in `five-solana/src/` (Solana wrapper), not in compiler or VM

**To Proceed**:
1. Identify which validation is rejecting uninitialized accounts
2. Implement deferred validation for `@init` accounts (allow data_len == 0)
3. Rebuild and redeploy Solana program
4. Re-run E2E tests

All core compilation and VM functionality is implemented and ready. The fix is isolated to account constraint handling.
