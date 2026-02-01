# Token Template: Testing Quick Start Guide

## Quick Test Execution

### Deploy the Token Contract

```bash
npm run deploy
```

This will:
1. Compile the token contract to bytecode
2. Create script and VM state accounts on-chain
3. Upload bytecode in chunks
4. Finalize the script account
5. Save configuration to `deployment-config.json`
6. Verify account ownership post-deployment

**Output example:**
```
✓ Deployment Complete
  Script Account: GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ
  VM State: DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys
✓ Config saved to deployment-config.json
▶ Verifying account ownership...
  ✓ Script account owner correct
  ✓ VM state owner correct
```

### Run E2E Tests

```bash
npm run test:e2e
```

Tests 9 core token operations:
1. ✓ init_mint - Initialize mint state
2. ✓ init_token_account - Create token accounts for users
3. ✓ mint_to - Mint tokens
4. ✓ transfer - Transfer tokens between accounts
5. ✓ approve - Approve delegate
6. ✓ transfer_from - Transfer via delegate
7. ✓ revoke - Revoke delegate
8. ✓ burn - Burn tokens
9. ✓ freeze/thaw - Freeze and thaw accounts

**Output example:**
```
✓ init_mint succeeded
   Signature: 5pZK2xYqLi9m...
   CU: 12345

✓ mint_to_User1 succeeded
   Signature: 7qBL3zRp...
   CU: 8910

✓ transfer succeeded
   Signature: 9sKL4aUq...
   CU: 7654

... (more operations)

🚀 Token E2E Test Completed Successfully!
```

### Troubleshooting: "IllegalOwner" Errors

If you see errors like:
```
❌ transaction_name FAILED (on-chain error)
   Error: {"InstructionError":[0,"Custom"]}
   VM Error: IllegalOwner
```

Run the ownership debugger:
```bash
npm run test:debug-owner
```

**Output example:**
```
╔═══════════════════════════════════════════════════════════╗
║ Debug: Account Ownership Analysis                         ║
╚═══════════════════════════════════════════════════════════╝

Script Account: GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ
  Owner: GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ ❌
  Expected: 6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k
  Match: ❌

❌ ISSUE FOUND: Script account is not owned by Five VM program!
   This causes "Provided owner is not allowed" / "IllegalOwner" error

   FIX: Redeploy script account with correct owner:

   npm run deploy
```

**Fix:** Redeploy with correct configuration
```bash
npm run deploy
npm run test:e2e
```

## Testing Different Scenarios

### Test with Custom RPC

```bash
RPC_URL=http://devnet.example.com npm run test:e2e
```

### Compare Baseline vs Register-Optimized (when available)

```bash
node compare-baseline-vs-registers.mjs
```

Shows:
- Side-by-side CU usage comparison
- Operation success/failure status
- Identifies register-specific issues

## What Each Test Does

### init_mint
- Creates a new mint with authority and freeze authority
- Initializes mint state on-chain

### init_token_account
- Creates token accounts for 3 test users
- Links each account to the mint

### mint_to
- Mints 1000 tokens to User1
- Mints 500 tokens each to User2 and User3

### transfer
- Transfers 100 tokens from User2 to User3

### approve & transfer_from
- User3 approves User2 as delegate for 150 tokens
- User2 transfers 50 tokens from User3 to User1

### revoke
- User3 revokes User2's delegate authority

### burn
- User1 burns 100 of their tokens

### freeze/thaw
- Freeze User2's token account
- Thaw User2's token account

### disable_mint
- Disables the mint authority (prevents future minting)

## Output Interpretation

### ✓ Successful Transaction
```
✓ operation_name succeeded
   Signature: 5pZK2xYqLi9mNoPqRsT...
   CU: 12345
```
- Transaction succeeded on-chain
- Signature shows the transaction ID
- CU shows compute units consumed

### ❌ Failed Transaction
```
❌ operation_name FAILED (on-chain error)
   Error: {"InstructionError":[...]}
   VM Error: IllegalOwner
   Signature: 5pZK2xYqLi9mNoPqRsT...
```
- Transaction failed on-chain
- VM Error shows the specific type
- Signature allows you to check Solana Explorer
- Test exits with error code 1

## Performance Expectations

Typical CU usage for token operations on localnet:
- init_mint: 12,000 - 15,000 CU
- init_token_account: 8,000 - 10,000 CU
- mint_to: 8,000 - 10,000 CU
- transfer: 7,000 - 9,000 CU
- approve/revoke: 6,000 - 8,000 CU
- burn: 8,000 - 10,000 CU
- freeze/thaw: 8,000 - 10,000 CU

Note: These are estimates and may vary based on:
- Solana network version
- VM implementation
- Account state
- Bytecode optimization

## Common Issues

| Issue | Solution |
|-------|----------|
| "Script account not found" | Run `npm run deploy` first |
| "IllegalOwner" error | Run `npm run test:debug-owner`, then `npm run deploy` |
| "VM State account not found" | Check deployment-config.json exists |
| RPC connection errors | Verify localnet is running: `solana-test-validator` |
| Insufficient balance | Fund payer with more SOL |

## Test Files

- **e2e-token-test.mjs** - Main test suite (9 operations)
- **compare-baseline-vs-registers.mjs** - Performance comparison
- **debug-illegal-owner.mjs** - Ownership diagnostics
- **deploy-to-five-vm.mjs** - Deployment script
- **deployment-config.json** - Auto-generated, stores account addresses

## Next Steps

After successful tests:
1. Review the test state in `test-state-fiveprogram.json`
2. Check Solana Explorer for transaction details
3. Verify account balances match expectations
4. Compare baseline vs register-optimized when available

## Contact & Debugging

For detailed error analysis:
1. Check transaction signature in Solana Explorer
2. Review logs in test output
3. Run ownership debugger: `npm run test:debug-owner`
4. Check deployment-config.json for correct account addresses
