# Devnet CU / Fee-Vault Handoff

Date: 2026-02-15

## Summary
- Devnet fee vault failure was caused by deploying a fresh program ID while the VM build uses **hardcoded** `vm_state` + fee vault PDAs for **program ID `3Sz...` only**.
- Fix: deploy VM to devnet **using program ID `3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1`**, then initialize canonical vm_state + hardcoded fee vaults.
- Token/AMM scripts were deployed on devnet to that VM program ID.
- The built-in token E2E test (`five-templates/token/e2e-token-test.mjs`) still fails with `InvalidInstructionData` at `init_mint` because the execute wire format in that script doesn’t match the on-chain expectations in this VM build.

## Root Cause (Fee Vault Failure)
- Production build verifies **hardcoded vm_state + fee vault addresses** and **bumps**.
- Any random devnet program ID fails fee-vault init and execute due to `verify_hardcoded_vm_state_account` + `verify_hardcoded_fee_vault_account` checks.
- Relevant code:
  - `five-solana/src/common.rs` (hardcoded PDAs and validation)
  - `five-solana/src/instructions/fees.rs` (fee vault init)
  - `five-solana/src/instructions/execute.rs` (fee vault verification)
  - `five-solana/src/instructions/deploy.rs` (hardcoded vm_state verification)

## Devnet Deployment (Current State)
- VM program ID: `3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1`
- vm_state PDA: `AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit`
- Fee vault shard 0: `HXW6bZsdJW6Be5c51NNpNb9NcVxmHbUrF9oKkt4C1tEH`
- Fee vault shard 1: `4jDYhXWWxdoz1ojPWeAUVrWSbpZTMz3qL3mUqZ1VALsq`

### vm_state values (devnet)
- `deploy_fee_lamports = 10000`
- `execute_fee_lamports = 85734`
- `fee_vault_shard_count = 2`

### Devnet Signatures (confirmed)
- VM deploy: `48fYc8K5s4WgRL4APWUKynNvqForhVr5fiZUR7qLNt9u9tCXKU3uCY8gxJGHKPuHe6LoFabojsUGhLSe2rFXSneC`
- vm_state init: `4yeAZgrtW3bdpDfFyoKwsksEqCLjL65dHZ8gVngVfky7zgJQVKQ3VZEu853DmBgRHSP2iWdNVvza7h4m7YQ95eVz`
- fee vault shard0 init: `WzKaFmeqQr3PHu1BAowRQofGioF9WP9pL8oWBgBK25xkevneYnqD924v21zDWxao9VMzZbe9thrS2SvQCKnbAhH`
- fee vault shard1 init: `5qCAGybwLtHUu3ucSAaC8t7qDvyLHP7rj1wxUpvNCmp9e2XwsVqXuQM5Z1wqZ32iUXPx3JS1b29Y5vrN9PJTRKcE`

### Devnet Script Accounts (deployed)
- Token script account: `BhZsLNy2RHwCXzMDJpBgghm12kTJBKnNU3wKZgD92hVZ`
- AMM script account: `2caUNt4w57YHmLZsmM85DsUiamSEchYRUMjUkdqwFEzG`

## Known Test Failure
- `five-templates/token/e2e-token-test.mjs` fails at `init_mint` with `InvalidInstructionData`.
- This is due to execute payload mismatch between the SDK’s legacy encoding and the on-chain execute format used by this VM build.
- On-chain expects:
  - Execute discriminator `9`
  - Fee header `[0xFF, 0x53, fee_shard_index, fee_vault_bump]`
  - `function_index` as **u32 LE**
  - `param_count` as **u32 LE**
  - encoded params
- See: `five-sdk/src/modules/execute.ts` for canonical encode path.

## Report Artifact
- Consolidated localnet+devnet CU + signatures report:
  - `five-solana/tests/benchmarks/validator-runs/localnet-devnet-cli-cu-report.json`

## If You Need a Successful Devnet Token Test
Options:
1. Run a custom execute that constructs the correct execute payload (fee header + u32 index + u32 param count), and pass correct account metas for the function.
2. Patch `e2e-token-test.mjs` to use the new execute format or route through `FiveSDK.executeOnSolana` with updated encoding.

## Commands Used (for reproducibility)
- Deploy VM to devnet with hardcoded-compatible ID:
  - `solana program deploy /Users/ivmidable/Development/five-mono/target/deploy/five.so --program-id /Users/ivmidable/Development/five-mono/target/deploy/five-keypair.json -u https://api.devnet.solana.com`
- Init vm_state + fee vaults (custom script used to enforce hardcoded addresses):
  - VM state seed: `vm_state`
  - Fee vault seed: `\xFFfive_vm_fee_vault_v1`
  - Shards 0/1 with bumps 255/254

## Constraints to Remember
- Any devnet deploy MUST use program ID `3Sz...` (hardcoded PDAs).
- Fresh program IDs will always fail fee-vault init and execute.
