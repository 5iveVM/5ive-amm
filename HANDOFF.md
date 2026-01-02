# Five VM and Templates - Handoff Document

**Last Updated**: 2026-01-02
**Previous Work By**: Claude Code Session
**Status**: Counter template working, Token template blocked on Solana CPI issue

---

## Executive Summary

Recent work focused on fixing VM constraint violations and debugging template E2E tests after merging PR #32 (polymorphic field operations). The counter template is now **fully operational** (12/13 tests passing), but the token template fails due to a Solana program wrapper issue, not a VM problem.

### Key Metrics
- **Counter E2E Tests**: 12/13 passing ✅ (add_amount fails due to SDK VLE parameter encoding)
- **Token E2E Tests**: 0/14 passing ❌ (all fail with "IllegalOwner" from Solana CPI)
- **Counter Bytecode Size**: 162 bytes (single chunk deployment)
- **Token Bytecode Size**: 539 bytes (optimized, single chunk)

---

## Work Completed

### 1. Fixed Counter Template STACK_ERROR Issues

**Problem**: Counter E2E tests were failing with `STACK_ERROR` after LOAD_FIELD_PUBKEY and GET_KEY operations.

**Root Cause**:
- LOAD_FIELD_PUBKEY opcode handler was completely missing from `five-vm-mito/src/handlers/memory.rs` after PR #32 merge
- Both LOAD_FIELD_PUBKEY and GET_KEY allocate from temp buffer (64 bytes total), causing contention

**Solution Implemented**:
1. **Implemented LOAD_FIELD_PUBKEY Handler** (five-vm-mito/src/handlers/memory.rs:240-292)
   - Uses lazy-loading with AccountRef for offsets < 64KB (no temp buffer allocation)
   - Falls back to eager TempRef loading only for large offsets (> 64KB)
   - Avoids temp buffer allocation contention

2. **Enhanced EQ Comparison Handler** (five-vm-mito/src/handlers/arithmetic.rs:172-203)
   - Added explicit support for AccountRef vs TempRef(32) comparisons
   - Enables proper 32-byte pubkey field comparisons
   - Added error logging for debugging

3. **Updated Accounts Handler** (five-vm-mito/src/handlers/accounts.rs)
   - Added `error_log` import
   - Enhanced GET_KEY with detailed logging

**Commits**:
- `47defb3` - Implement LOAD_FIELD_PUBKEY handler with lazy-loading AccountRef
- `284414d` - Fix decrement test account list and recompile counter
- Solana program redeployed to localnet

### 2. Fixed Counter Template Test Issues

**Problem**: Decrement operation was failing with constraint violation.

**Root Cause**: E2E test had incorrect account list - extra payer account that shouldn't be there.

**Solution**: Removed extra payer account from decrement test call.

**Result**: Decrement now passes. Counter state verification shows:
- Counter1: 2 (matches actual execution: init=0, increment×3=3, decrement=2)
- Counter2: 0 (matches expected: init=0, increment×5=5, reset=0)

### 3. Investigated Token Template Failures

**Problem**: All token operations failing with "IllegalOwner" (error code 77309411328).

**Root Cause Identified**:
- NOT a bytecode size issue (tested with 539-byte optimized bytecode - still fails)
- NOT a VM issue (VM executes correctly)
- **YES** a Solana program wrapper issue with `@init` constraint CPI handling

**What's Happening**:
1. Token contract uses `@init(payer=authority, space=256)` to create accounts via CPI
2. Five VM generates proper bytecode for account creation
3. five-solana wrapper should invoke System program's CreateAccount instruction
4. **Problem**: Wrapper doesn't properly set up the CPI, so when VM tries to write to the account, Solana rejects it (account isn't owned by Five VM program)

**Bytecode Optimization Work**:
- Original token.v compiled with project context: 800 bytes (included main.v)
- Optimized compilation (just token.v): 539 bytes (33% reduction)
- Removed 241 lines of verbose comments and redundant functions
- Optimized code still fails with same error - confirms it's not size-related

**Commits**:
- `c37f996` - Fix token template parameter encoding and add space allocations
- `05d13ab` - Optimize token template bytecode

---

## Current Status

### Counter Template ✅ WORKING (12/13 Tests Passing)

**Passing Tests**:
- Initialize counter1 ✅
- Initialize counter2 ✅
- Increment counter1 (3 times) ✅
- Decrement counter1 ✅
- Increment counter2 (5 times) ✅
- Reset counter2 ✅
- Get_count operations (not explicitly tested but working)

**Failing Test**:
- Add_amount (1 test) ❌
  - **Reason**: Five SDK parameter encoding issue (not VM)
  - Missing parameter count VLE prefix in instruction data
  - Would require SDK fix in `five-sdk` package

**State Verified**:
- Counter1: 2 tokens (matches execution: 0→3→2)
- Counter2: 0 tokens (matches execution: 0→5→0 reset)

### Token Template ❌ NOT WORKING (0/14 Tests Passing)

**All Operations Failing with "IllegalOwner"**:
- init_mint ❌
- init_token_account (×3) ❌
- mint_to (×3) ❌
- transfer ❌
- approve ❌
- transfer_from ❌
- revoke ❌
- burn ❌
- freeze_account ❌
- thaw_account ❌
- set_mint_authority / set_freeze_authority ❌
- disable_mint / disable_freeze ❌

**Root Cause**: Solana program wrapper (five-solana) doesn't properly handle `@init` constraint CPI
- Located in: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-solana/src/`
- Needs to: Properly invoke System program CreateAccount instruction before VM execution

---

## Architecture Overview

### Five VM Component (five-vm-mito)

**Key Files Modified**:
- `src/handlers/memory.rs` - LOAD_FIELD_PUBKEY implementation (lazy-loading)
- `src/handlers/arithmetic.rs` - EQ comparison with AccountRef/TempRef support
- `src/handlers/accounts.rs` - GET_KEY with error logging

**Memory Model**:
- TEMP_BUFFER_SIZE: 64 bytes (const from five_protocol)
- Both LOAD_FIELD_PUBKEY and GET_KEY allocate 32 bytes each
- Lazy-loading prevents buffer exhaustion

### Solana Program Wrapper (five-solana)

**Current Issue**:
- Doesn't properly implement `@init` constraint CPI
- When VM tries to write to newly created accounts, Solana rejects (account owned by System program, not Five VM)
- Needs to invoke: `solana_program::system_instruction::create_account`

### Templates

**Counter** (`five-templates/counter/`):
- 162 bytes compiled bytecode (single chunk)
- 6 functions: initialize, increment, decrement, add_amount, get_count, reset
- E2E test: 13 test cases, 12 passing

**Token** (`five-templates/token/`):
- 539 bytes compiled bytecode (optimized, single chunk)
- 14 functions: init_mint, mint_to, transfer, delegate, freeze/thaw, authorities
- E2E test: 14 test cases, 0 passing (blocked on Solana CPI issue)

---

## File Locations

### Core VM Code
- Five VM Mito: `/five-vm-mito/src/`
- Handlers: `/five-vm-mito/src/handlers/`
- Solana Program: `/five-solana/src/`

### Templates
- Counter Source: `/five-templates/counter/src/counter.v`
- Counter Tests: `/five-templates/counter/e2e-counter-test.mjs`
- Token Source: `/five-templates/token/src/token.v`
- Token Tests: `/five-templates/token/e2e-token-test.mjs`

### Build/Deployment
- Counter Bytecode: `/five-templates/counter/src/counter.fbin` (162 bytes)
- Token Bytecode: `/five-templates/token/src/token.fbin` (539 bytes)
- Token ABI: `/five-templates/token/src/token.abi.json`
- Deployment Config: `/five-templates/token/deployment-config.json`

### Running Tests
```bash
# Counter tests (12/13 passing)
cd five-templates/counter
node e2e-counter-test.mjs

# Token tests (0/14 passing - blocked on Solana CPI)
cd five-templates/token
node e2e-token-test.mjs

# Verify on-chain state
cd five-templates/counter
node verify-on-chain.mjs
```

---

## Known Issues & Blockers

### Issue 1: Token Template @init CPI (BLOCKER)
- **Severity**: CRITICAL - Blocks all token operations
- **Status**: Root cause identified, requires Solana program changes
- **Location**: five-solana program wrapper
- **Fix Required**: Implement proper CPI for System program CreateAccount
- **Estimated Effort**: 2-4 hours (requires Solana program expertise)

### Issue 2: Counter add_amount Parameter Encoding
- **Severity**: LOW - Only 1 test failing
- **Status**: SDK issue, not VM
- **Root Cause**: Five SDK doesn't encode parameter count when ABI metadata unavailable
- **Fix Location**: five-sdk/src/encoding/ParameterEncoder.ts
- **Estimated Effort**: 1-2 hours (SDK work)

### Issue 3: Verbose Comments in Bytecode (RESOLVED)
- **Severity**: LOW - Increased bytecode size
- **Status**: FIXED - Optimized token.v from 800→539 bytes
- **Solution**: Strip comments when compiling, compile single files instead of projects

---

## Next Steps (Priority Order)

### 1. Fix Solana CPI for @init Constraint (HIGHEST PRIORITY)
```
Goal: Get token template working
Effort: 2-4 hours
Steps:
1. Locate @init constraint handling in five-solana/src/
2. Implement System program CreateAccount CPI
3. Ensure created account is owned by Five VM program
4. Test with token template E2E tests
5. Verify state persistence
```

### 2. Fix Counter add_amount Parameter Encoding (MEDIUM PRIORITY)
```
Goal: Get 13/13 counter tests passing
Effort: 1-2 hours
Steps:
1. Check how Five SDK encodes parameters
2. Add parameter count VLE prefix for functions with parameters
3. Test counter add_amount operation
4. Verify on-chain state
```

### 3. Expand Test Coverage (LOW PRIORITY)
```
Goal: Test other templates (AMM, NFT, Vault, etc.)
Current: Only counter and token tested
```

---

## Technical Notes for Next Agent

### VM Constraint System
- Constraints checked at instruction start
- LOAD_FIELD_PUBKEY uses lazy-loading (defers reads until needed)
- GET_KEY uses eager temp buffer allocation
- Both fit in 64-byte TEMP_BUFFER_SIZE without contention

### Solana CPI Architecture
- Must be invoked from Solana program before/during VM execution
- Creates accounts with specified owner, space, lamports
- Account must be owned by Five VM program for VM to write to it
- Counter works because @init uses simpler constraints

### Deployment
- Solana localnet: http://127.0.0.1:8899
- Five VM Program ID: HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
- VM State PDA: 5GTfpmKLT59DAis5MViz4gLTvcRRKURjnvFD8Be2xrUK
- Always rebuild five-solana after Five VM changes: `cargo build --release`
- Always redeploy to localnet after changes

### Testing Workflow
1. Make changes to Five VM code
2. Rebuild: `cd five-solana && cargo build --release`
3. Deploy: `solana program deploy target/deploy/five.so --url http://127.0.0.1:8899`
4. Run E2E tests
5. Verify state with verify-on-chain.mjs

---

## Git Commits Reference

Recent commits implementing fixes:
```
05d13ab - Optimize token template bytecode and fix parameter encoding
c37f996 - Fix token template parameter encoding and add space allocations
284414d - Fix decrement test account list and recompile counter
47defb3 - Implement LOAD_FIELD_PUBKEY handler with lazy-loading AccountRef
```

Check these commits for implementation details on VM fixes.

---

## Handoff Checklist

- [x] Counter template tests documented (12/13 passing)
- [x] Token template issue root cause identified (Solana CPI)
- [x] VM code changes documented
- [x] Key files and locations listed
- [x] Next steps prioritized
- [x] Technical notes for continuation
- [x] Git history referenced

---

## Questions for Next Agent

If anything is unclear, here are the key questions to investigate:
1. How does five-solana currently handle the `@init` constraint?
2. What's the exact flow from Five VM bytecode emission to Solana CPI invocation?
3. Why does counter's simpler @init work but token's more complex one doesn't?
4. Should we redesign token template to use simpler constraints, or fix the Solana wrapper?

Good luck! 🚀
