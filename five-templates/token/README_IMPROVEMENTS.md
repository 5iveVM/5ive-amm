# Token Template: E2E Test Improvements

## Overview

This directory contains improved E2E (end-to-end) testing for the Five token contract template. The improvements fix transaction verification issues that were causing false positives (failed transactions marked as successful).

## Quick Start

### Deploy
```bash
npm run deploy
```

### Test
```bash
npm run test:e2e
```

### Debug (if needed)
```bash
npm run test:debug-owner
```

## What's New

### 1. **Better Transaction Verification** ✅
- Failed transactions are now properly detected
- Tests exit with error code 1 on any failure
- Clear error messages show what went wrong

### 2. **VM Error Classification** ✅
- Specific error types identified (IllegalOwner, StackUnderflow, etc.)
- Transaction signatures shown for Solana Explorer
- Relevant logs displayed on failure

### 3. **Ownership Debugging** ✅
- Automated tool to diagnose "IllegalOwner" errors
- Post-deployment ownership verification
- Clear fix guidance provided

### 4. **Comparison Framework** ✅
- Framework ready for baseline vs optimized testing
- Can compare performance when register optimization available

## Documentation

Start with one of these based on your needs:

| Document | Purpose |
|----------|---------|
| **TESTING_QUICK_START.md** | How to run tests and what to expect |
| **TEST_IMPROVEMENTS_SUMMARY.md** | Technical details of all changes |
| **IMPLEMENTATION_CHECKLIST.md** | Verification that everything was implemented |

## Files

### Modified
- `e2e-token-test.mjs` - Improved transaction verification
- `deploy-to-five-vm.mjs` - Added ownership checks
- `package.json` - New test scripts

### New
- `debug-illegal-owner.mjs` - Ownership debugging tool
- `compare-baseline-vs-registers.mjs` - Comparison framework
- `README_IMPROVEMENTS.md` - This file
- Documentation files (see above)

## Test Operations

The test suite covers all core token operations:

1. **init_mint** - Create mint state (decimals, metadata)
2. **init_token_account** - Create token accounts for users
3. **mint_to** - Mint tokens to accounts
4. **transfer** - Transfer tokens between accounts
5. **approve** - Approve delegate to spend tokens
6. **transfer_from** - Transfer via approved delegate
7. **revoke** - Revoke delegate approval
8. **burn** - Burn tokens
9. **freeze_account** - Freeze token account
10. **thaw_account** - Unfreeze token account
11. **disable_mint** - Disable mint authority

## Success Indicators

### ✓ Successful Test Run
```
✓ init_mint succeeded
   Signature: 5pZK2xYqLi9mNoPqRsT...
   CU: 12345

✓ mint_to_User1 succeeded
   Signature: 7qBL3zRp...
   CU: 8910

... (more operations)

🚀 Token E2E Test Completed Successfully!
```

### ✗ Failed Test Run
```
❌ transaction_name FAILED (on-chain error)
   Error: {"InstructionError":[...]}
   VM Error: IllegalOwner
   Signature: 5pZK2xYqLi9mNoPqRsT...
   [Test exits with error code 1]
```

## Common Issues

### "IllegalOwner" Error
The script account isn't owned by the Five VM program.

**Fix:** `npm run deploy`

### "Account not found"
Accounts weren't created on-chain.

**Check:** Run `npm run test:debug-owner`

### Insufficient balance
Payer doesn't have enough SOL for transaction fees.

**Fix:** Fund the payer account

## Environment Variables

### RPC_URL
Set custom RPC endpoint:
```bash
RPC_URL=http://devnet.example.com npm run test:e2e
```

### FIVE_PROGRAM_ID
Override Five VM program ID:
```bash
FIVE_PROGRAM_ID=<program_id> npm run deploy
```

### VM_STATE_PDA
Use existing VM state account:
```bash
VM_STATE_PDA=<pda> npm run deploy
```

## Performance

Typical compute unit (CU) usage:
- init_mint: ~12,000-15,000 CU
- init_token_account: ~8,000-10,000 CU
- mint_to: ~8,000-10,000 CU
- transfer: ~7,000-9,000 CU
- approve: ~6,000-8,000 CU
- revoke: ~6,000-8,000 CU
- burn: ~8,000-10,000 CU
- freeze/thaw: ~8,000-10,000 CU

## Troubleshooting

### Tests Won't Run
1. Ensure localnet is running: `solana-test-validator`
2. Check payer has SOL: `solana balance`
3. Verify deployment: `npm run deploy`

### Tests Fail with Errors
1. Run ownership debugger: `npm run test:debug-owner`
2. Check Solana Explorer with transaction signature
3. Review deployment-config.json for correct addresses

### False Positive Prevention
The improved tests now properly catch failures that were previously missed:
- **Before:** Failed transactions marked as success
- **After:** Failed transactions cause test to exit with error

## For Developers

### Adding New Test Operations

In `e2e-token-test.mjs`:

```javascript
// 1. Create instruction
const ix = await program
    .function('operation_name')
    .accounts({ /* ... */ })
    .args({ /* ... */ })
    .instruction();

// 2. Execute with label
const res = await sendInstruction(connection, ix, signers, 'operation_label');

// 3. Assert success (or test fails)
assertTransactionSuccess(res, 'operation_label');

// 4. Use results if needed
if (res.success) {
    console.log(`CU used: ${res.cu}`);
    console.log(`Signature: ${res.signature}`);
}
```

### Error Classification

The test framework automatically classifies errors:
- `IllegalOwner` - Account not owned by program
- `StackUnderflow` - VM stack too small
- `StackOverflow` - VM stack exceeded limit
- `InvalidInstruction` - Bad opcode
- `AccountNotFound` - Missing account

Add more classifications in `extractVMError()` function.

## Next Steps

1. **Deploy:** `npm run deploy`
2. **Test:** `npm run test:e2e`
3. **Debug if needed:** `npm run test:debug-owner`
4. **Compare when ready:** `node compare-baseline-vs-registers.mjs`

## Related Files

- `deployment-config.json` - Auto-generated deployment config
- `test-state-fiveprogram.json` - Test results and account info
- `src/token.v` - Token contract source code
- `build/five-token-template.five` - Compiled bytecode

## Support

For questions or issues:

1. Check **TESTING_QUICK_START.md** for common problems
2. Run `npm run test:debug-owner` for diagnostics
3. Review **TEST_IMPROVEMENTS_SUMMARY.md** for technical details
4. Check transaction signature in Solana Explorer

---

**Status:** All improvements implemented and verified ✅
