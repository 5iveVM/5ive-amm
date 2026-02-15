# Fee Vault Hardcoding Optimization - Localnet Testing Handoff

## Current Status
Fee vault hardcoding optimization is **partially deployed** on localnet with a configuration mismatch that needs resolution.

## What's Deployed
- ✅ Five program (ID: `3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1`) - Updated SBF with fee vault fixes
- ✅ VM State PDA initialized (Address: `AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit`)
- ✅ Fee Vault Shard 0 initialized (Address: `HXW6bZsdJW6Be5c51NNpNb9NcVxmHbUrF9oKkt4C1tEH`)
- ✅ Fee Vault Shard 1 initialized (Address: `4jDYhXWWxdoz1ojPWeAUVrWSbpZTMz3qL3mUqZ1VALsq`)
- ⏳ Token template deployment - **BLOCKED** (see below)

## The Problem
There's an address mismatch between hardcoded constants and the actual localnet VM state:

**Hardcoded VM State PDA (in code):**
```
0x5f, 0x35, 0x23, 0x14, 0x05, 0x93, 0xba, 0xb7,
0x8a, 0x7b, 0xc1, 0x93, 0x95, 0xc4, 0x13, 0x94,
0xeb, 0x88, 0x78, 0x86, 0xd0, 0xd2, 0x07, 0x8c,
0x12, 0x1c, 0x69, 0x63, 0xf3, 0x69, 0x3a, 0x59
```

**Actual Localnet VM State PDA:**
```
AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit
= 0x8a, 0x45, 0xdd, 0x0b, 0x79, 0x66, 0x32, 0x42,
  0x6c, 0x74, 0x0d, 0xe4, 0xdf, 0xfa, 0x05, 0x9f,
  0x12, 0x72, 0xd9, 0x6e, 0x4e, 0x0c, 0x20, 0xe3,
  0xe0, 0xd0, 0x3d, 0x30, 0x88, 0x05, 0x2e, 0x05
```

This causes the `verify_hardcoded_vm_state_account()` function to reject the correct VM state account during `init_large_program` instruction processing.

## Root Cause
The hardcoded constants were generated for the current program ID (`3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1`), but the VM state on localnet was created earlier and doesn't match the expected derived address. This suggests either:
1. The VM state PDA was created differently than expected
2. The derivation logic changed between versions

## Solutions (Choose One)

### Option A: Update Hardcoded Constants to Match Localnet (RECOMMENDED for testing)
1. Derive the correct VM state PDA bytes for program ID `3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1`:
   ```bash
   cd five-templates/token && node -e "
   const {PublicKey} = require('@solana/web3.js');
   const programId = new PublicKey('3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1');
   const seed = Buffer.from('FIVE_VM_STATE_CANONICAL');
   const [vmState, bump] = PublicKey.findProgramAddressSync([seed, Buffer.from([255])], programId);
   console.log('Base58:', vmState.toBase58());
   console.log('Bytes:', Array.from(vmState.toBytes()).map((b,i) => (i%8===0?'\n    ':'') + '0x' + b.toString(16).padStart(2,'0') + ',').join(''));
   "
   ```

2. Update `five-solana/src/common.rs` lines 42-47 with the derived bytes

3. Rebuild SBF and redeploy:
   ```bash
   cargo build-sbf --manifest-path five-solana/Cargo.toml
   solana program deploy target/deploy/five.so --url http://127.0.0.1:8899
   ```

4. Continue with token deployment

### Option B: Recreate Localnet VM State with Matching PDA
1. Destroy current VM state (if possible):
   ```bash
   solana nonce deauthorize <VM_STATE_PDA> --url http://127.0.0.1:8899
   ```

2. Initialize new VM state with correct derivation

3. Continue with option A above

### Option C: Disable Hardcoding for Localnet Testing
1. Modify `five-solana/src/common.rs` to always use dynamic derivation:
   ```rust
   #[cfg(not(test))]
   {
       // Change to use dynamic derivation even in production for localnet compatibility
       let (expected_vm_state, _) = derive_canonical_vm_state_pda(program_id)?;
       if vm_state_account.key() != &expected_vm_state {
           return Err(ProgramError::InvalidArgument);
       }
   }
   ```

2. Rebuild and redeploy

## Files Modified Recently

### Core Implementation
- `five-solana/src/common.rs` - Hardcoded constants and verification functions
- `five-solana/src/instructions/fees.rs` - Fixed system program validation (was comparing to `[0u8; 32]`)
- `five-solana/src/instructions/deploy.rs` - Fee collection in init_large_program
- `five-templates/token/deploy-to-five-vm.mjs` - Updated to include fee vault accounts and hardcoded addresses

### Test Infrastructure
- `scripts/init-devnet-fee-vaults.mjs` - Updated to support localnet with hardcoded VM state
- `five-templates/token/e2e-token-test.mjs` - Loads from `deployment-config.json`
- `five-templates/token/deployment-config.json` - Current localnet addresses

## Next Steps
1. **IMMEDIATE**: Choose one of the three solutions above to resolve the VM state address mismatch
2. **DEPLOY TOKEN**: Once VM state address is fixed, run token deployment:
   ```bash
   cd five-templates/token && node deploy-to-five-vm.mjs
   ```
3. **RUN E2E TESTS**: Execute token e2e tests:
   ```bash
   cd five-templates/token && node e2e-token-test.mjs
   ```
4. **CAPTURE CU METRICS**: Record signatures and CU consumption from e2e test output

## CU Optimization Summary
The hardcoding optimization is designed to:
- **Eliminate 2 PDA derivation syscalls per transaction** (deploy + execute)
- **Savings: ~1500 CU per syscall = ~3000 CU per transaction**
- Uses `#[cfg(not(test))]` to apply hardcoding only in production builds
- Falls back to dynamic derivation in test mode for flexibility

## Deployment Config
Current localnet config at `five-templates/token/deployment-config.json`:
```json
{
  "rpcUrl": "http://127.0.0.1:8899",
  "fiveProgramId": "3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1",
  "vmStatePda": "AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit",
  "tokenScriptAccount": "GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ",
  "timestamp": "2026-02-14T21:30:00.000Z"
}
```

## Key Insights
1. **Fee Vault Sharding**: Using 2 fee vaults (shard 0 and 1) instead of 10 for easier testing
2. **Instruction Format**: Deploy uses 3-byte format `[0xFF, 0x53, fee_shard_index]` (no bump byte)
3. **System Program Check**: Fixed bug where code was comparing to `[0u8; 32]` instead of actual system program
4. **Chunked Deployment**: Token uses `InitLargeProgram` + `AppendBytecode` + `FinalizeScript` for large bytecode (832 bytes)
