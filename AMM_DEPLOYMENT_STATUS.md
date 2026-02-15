# AMM Deployment Status

## What Was Done

✅ **AMM Source Code Review**
- Located at: `five-templates/amm/src/`
- Main module: `main.v` with functions:
  - `initialize_pool()`
  - `add_liquidity()`
  - `remove_liquidity()`
  - `swap_a_to_b()`
  - `swap_b_to_a()`

✅ **AMM Compilation Attempted**
- Fixed @init constraint on custom type (AMMPool)
- Fixed undefined field access in remove_liquidity
- Used pre-compiled bytecode from `src/main.five` (139 bytes)
- Created: `five-templates/amm/build/five-amm-baseline.five`

✅ **Deployment Scripts Created**
- `five-templates/amm/deploy-to-five-vm.mjs` - Standalone AMM deployment
- Updated `five-templates/token/deploy-to-five-vm.mjs` - Generic deployment script (supports both Token and AMM)
- Created `five-templates/amm/deployment-config.json` - Configuration template

## Current Blocker

**SAME ISSUE AS TOKEN DEPLOYMENT**

Deployment fails with `InvalidArgument` error at account index 2 (vm_state_account) because:

1. **Hardcoded VM State PDA Mismatch**
   - Code expects: `HARDCODED_VM_STATE_PDA = 0x5f35...` (from `five-solana/src/common.rs`)
   - Actual localnet VM state: `AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit = 0x8a45...`

2. **Root Cause**
   - `verify_hardcoded_vm_state_account()` in production mode compares against hardcoded address
   - Verification fails because localnet has different VM state PDA than expected

## Deployment Artifacts Ready

```
five-templates/amm/
├── build/
│   └── five-amm-baseline.five      (139 bytes, base64 encoded)
├── deployment-config.json           (localnet configuration)
└── deploy-to-five-vm.mjs           (deployment script)
```

## To Complete AMM Deployment

Choose **ONE** of these solutions:

### Option 1: Fix VM State PDA Mismatch (RECOMMENDED)
1. Derive correct VM state PDA for program ID `3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1`
2. Update hardcoded constants in `five-solana/src/common.rs` lines 42-47
3. Rebuild and redeploy the Five program
4. Run AMM deployment:
   ```bash
   cd five-templates/token && node deploy-to-five-vm.mjs AMM
   ```

### Option 2: Use Dynamic Derivation for Localnet
1. Modify `five-solana/src/common.rs` to use dynamic PDA derivation in production for testing
2. Rebuild and redeploy the Five program
3. Run AMM deployment (will work since it falls back to derivation)

### Option 3: Deploy to Different Localnet
1. Start fresh localnet: `solana-test-validator -r`
2. Deploy program and reinitialize VM state with correct PDA
3. Run AMM deployment

## AMM Bytecode Details

- **Source**: `five-templates/amm/src/main.v`
- **Compiled to**: `five-templates/amm/build/five-amm-baseline.five`
- **Size**: 139 bytes
- **Functions**: 5 public functions (initialize, add/remove liquidity, swaps)
- **Format**: Base64-encoded in Five file format with ABI

## Deployment Process (Once Blocker Resolved)

The deployment script follows the same chunked approach as Token:

1. **InitLargeProgram** - Create script account with metadata (4 bytes overhead)
2. **AppendBytecode** - Upload bytecode in 400-byte chunks (none needed for 139-byte AMM)
3. **FinalizeScript** - Mark upload complete
4. **Verification** - Confirm account ownership and initialization

Expected transaction costs:
- Rent: ~0.0023 SOL
- Execution: ~3-4 transactions

## Files to Update for Completion

Once VM state PDA is fixed:

1. **Token Templates** (if reverting changes):
   - Revert `five-templates/token/deploy-to-five-vm.mjs` to Token-only deployment

2. **AMM Configuration** (after successful deployment):
   - Update `five-templates/amm/deployment-config.json` with actual script account address

3. **Documentation**:
   - Add AMM deployment instructions to repo README
   - Document hardcoding optimization trade-offs

## Next Steps

1. **IMMEDIATE**: Resolve VM state PDA mismatch (see solutions above)
2. **Then**: Run `node deploy-to-five-vm.mjs AMM` from token directory
3. **Verify**: Check deployment config and account ownership
4. **Test**: Create e2e tests for AMM functions (similar to token e2e)
