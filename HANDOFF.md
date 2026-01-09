# Handoff: Token Template `init_mint` Debugging

## ✅ FIXED: Debug Panic Statements Removed

### Root Cause (Resolved)
Found **4 debug `panic!` statements** that were blocking VM execution:

1. `five-vm-mito/src/handlers/accounts.rs` (line 24) - `CREATE_ACCOUNT` handler
2. `five-vm-mito/src/handlers/system/init.rs` (line 65) - `INIT_ACCOUNT` handler  
3. `five-vm-mito/src/handlers/system/init.rs` (line 186) - `INIT_PDA_ACCOUNT` handler
4. `five-vm-mito/src/context.rs` (line 1405) - `create_pda_account` method

These were trace statements left from debugging that caused immediate crash when account initialization opcodes were reached.

### Fix Applied
All panic statements have been removed and `five-solana` has been rebuilt and redeployed.

---

## ✅ FIXED: Script Account Owner Mismatch

### Issue
The token script was deployed with an older Five program instance, causing "Provided owner is not allowed" error (only 111 CU consumed).

### Solution
Redeployed token script using `deploy-to-five-vm.mjs` with current Five program ID.

**New Addresses:**
- Script: `tYpGyjcpjGbfm4tD327d6iXmuU8uPgy1zYvgCnZDmwe`
- VM State: `5cwbJMkXLg44ga71ATypxY8SRkvqzrsP9R2g5uihbJGy`
- Five Program: `DmBJLjdfSidk5SYMscpRZJeiyMqeBZvir1nHAVZZvAX8`

---

## ✅ FIXED: Account Index Mismatch (ConstraintViolation 0x232b)

### Root Cause (SOLVED)
The token test was passing an **extra payer account** in the accounts array that the bytecode didn't expect, causing account index misalignment and constraint validation failures.

**The Problem:**
```
DSL: pub init_mint(mint_account @init(payer=authority), authority, ...)
Test: Passes [mintAccount, authority, payer, SystemProgram, Rent]
       (3 account-type parameters instead of 2)
```

The `@init(payer=authority)` constraint tells the compiler that `authority` (parameter 1) will pay for account creation. But the test was passing a separate payer account at a different position, causing:
- Bytecode expected account at VM index 2 to be authority (the payer)
- But after the extra payer was inserted, indices shifted
- CHECK_SIGNER failed because it was checking the wrong account

**Pattern Mismatch with Counter:**
Counter template (WORKING): Passes owner once as parameter, SDK adds it again for fees
Token template (BROKEN): Was passing authority AND a separate payer, causing duplication

### Solution Applied
**File:** `five-templates/token/e2e-token-test.mjs`

Removed the separate payer account from ALL function calls' accounts arrays. Now:
- `@init(payer=X)` functions only receive X as the account parameter
- The SDK/helper automatically adds the transaction fee payer as an auxiliary account
- Account indices align: param 0 → VM index 1, param 1 → VM index 2, etc.

**Functions Fixed:**
1. `init_mint` (line 345-349) - Removed payer, kept mintAccount + authority
2. `init_token_account` (line 387-391) - Removed payer, kept account + owner
3. `mint_to` (line 424-428) - Removed payer
4. `transfer` (line 454-458) - Removed payer
5. `approve` (line 484-487) - Removed payer
6. `transfer_from` (line 508-512) - Removed payer
7. `revoke` (line 537-540) - Removed payer
8. `burn` (line 565-569) - Removed payer
9. `freeze_account` (line 594-598) - Removed payer
10. `thaw_account` (line 619-623) - Removed payer
11. `disable_mint` (line 648-651) - Removed payer

### Expected Account Layout (Corrected)
```
[0]: Script account
[1]: VM State PDA
[2]: First user account param (e.g., mint_account)
[3]: Second user account param (e.g., authority) <-- Payer for @init
[4]: Payer (added by executeTokenFunction helper)
[5]: System Program (added by executeTokenFunction helper)
[6]: Rent Sysvar (added by executeTokenFunction helper)

VM sees (after Script stripped):
[0]: VM State
[1]: First param → VM account_idx 1 (param_index 0 + ACCOUNT_INDEX_OFFSET)
[2]: Second param → VM account_idx 2 (param_index 1 + ACCOUNT_INDEX_OFFSET)
[3]: Payer (not referenced by bytecode, just for tx fees)
[4]: System Program
[5]: Rent Sysvar
```

---

## ⏳ IN PROGRESS: InvalidInstructionData During Bytecode Execution

### Current Status
Both `init_mint` and `init_token_account` fail with `InvalidInstructionData` after successfully:
- Creating accounts via CPI (System Program invocations succeed)
- Parsing instruction data
- Beginning bytecode execution (9-10K+ compute units consumed)

The error occurs **within VM bytecode execution**, not at parameter boundaries or account creation.

### Session Work (Latest)

**Fixes Applied:**
1. **@mut Attribute Added** - `init_token_account` owner parameter now marked as writable
2. **ABI Updated** - Created proper `.five` JSON with embedded bytecode and full metadata
3. **Script Created** - `create-five-file.mjs` automates .five file generation with correct ABI

**Files Modified:**
- `five-templates/token/src/token.v` - Added @mut to owner
- `five-templates/token/src/token.bin` - Recompiled with fix
- `five-templates/token/create-five-file.mjs` - New script for ABI generation
- `five-templates/token/e2e-token-test.mjs` - Comments clarifying payer roles

### Investigation Results

**Verified Correct:**
- Instruction VLE encoding is correct: [discriminator(9)][func_idx(VLE)][sentinel(128)][param_count(7)][params...]
- Account index mapping follows ACCOUNT_INDEX_OFFSET = 1
- System Program CPI for account creation is working
- Transaction structure matches counter template pattern

**Issue Location:**
The error is not in parameter encoding or account setup, but rather in the Five VM's bytecode execution path when handling:
- Multiple account parameters (2 accounts + 5 data params)
- Typed parameter decoding with account and non-account mix
- Constraint validation during execution

### Next Steps
Investigate Five VM's `parse_vle_parameters_unified` and typed parameter handling for functions with mixed account/data parameters. The counter template (2 accounts, 0 data params) works fine; the token template (2+ accounts + data params) fails.
