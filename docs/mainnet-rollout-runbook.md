# Mainnet Rollout Runbook (Canary -> Broad)

## Preconditions
- Solana CLI authenticated with the deployer keypair.
- Mainnet program keypair prepared at `target/deploy/mainnet-five-keypair.json`.
- `five-solana/constants.vm.toml` mainnet program ID matches the deploy keypair pubkey.
- Fresh mainnet constants + artifact generated via:
  - `./scripts/build-five-solana-cluster.sh --cluster mainnet`

## 1) Mainnet Dry-Run Gate
```bash
./scripts/mvp-release-gate.sh --cluster mainnet
```
Expected:
- SBF parity and runtime suites pass.
- E2E smoke is skipped for mainnet by design.

## 2) Deploy VM + Initialize State
Set env:
```bash
export FIVE_NETWORK=mainnet
export FIVE_RPC_URL=https://api.mainnet-beta.solana.com
export FIVE_PROGRAM_ID=$(solana-keygen pubkey target/deploy/mainnet-five-keypair.json)
export FIVE_KEYPAIR_PATH=$HOME/.config/solana/id.json
```

Deploy program:
```bash
solana program deploy target/deploy/five.so \
  --program-id target/deploy/mainnet-five-keypair.json \
  --keypair "$FIVE_KEYPAIR_PATH" \
  --url "$FIVE_RPC_URL"
```

Initialize VM state:
```bash
node scripts/init-localnet-vm-state.mjs \
  --network mainnet \
  --rpc-url "$FIVE_RPC_URL" \
  --program-id "$FIVE_PROGRAM_ID"
```

Initialize fee vaults (strict):
```bash
node scripts/init-devnet-fee-vaults.mjs \
  --network mainnet \
  --rpc-url "$FIVE_RPC_URL" \
  --program-id "$FIVE_PROGRAM_ID" \
  --strict
```

## 3) Configure Fees and Recipient
```bash
node scripts/vm-state-set-fees.mjs \
  --cluster mainnet \
  --rpc-url "$FIVE_RPC_URL" \
  --program-id "$FIVE_PROGRAM_ID" \
  --deploy-fee 10000 \
  --execute-fee 85734

node scripts/vm-state-set-fee-recipient.mjs \
  --cluster mainnet \
  --rpc-url "$FIVE_RPC_URL" \
  --program-id "$FIVE_PROGRAM_ID" \
  --fee-recipient <FEE_RECIPIENT_PUBKEY>
```

## 4) Parity Validation
```bash
node scripts/check-vm-constants-parity.mjs \
  --rpc-url "$FIVE_RPC_URL" \
  --program-id "$FIVE_PROGRAM_ID"

node scripts/vm-state-parity-check.mjs \
  --cluster mainnet \
  --rpc-url "$FIVE_RPC_URL" \
  --program-id "$FIVE_PROGRAM_ID" \
  --expected-authority <AUTHORITY_PUBKEY> \
  --expected-fee-recipient <FEE_RECIPIENT_PUBKEY> \
  --expected-deploy-fee 10000 \
  --expected-execute-fee 85734
```

## 5) Canary Validation
Mainnet-safe canary suite defaults to read/validation scenarios:
```bash
FIVE_EXPECTED_AUTHORITY=<AUTHORITY_PUBKEY> \
FIVE_EXPECTED_FEE_RECIPIENT=<FEE_RECIPIENT_PUBKEY> \
./scripts/run-sdk-validator-suites.sh \
  --network mainnet \
  --program-id "$FIVE_PROGRAM_ID" \
  --vm-state <VM_STATE_PDA> \
  --keypair "$FIVE_KEYPAIR_PATH"
```

Optional write canary (`token_full_e2e`) requires explicit opt-in:
```bash
FIVE_ENABLE_MAINNET_WRITES=1 \
FIVE_TOKEN_SCRIPT_ACCOUNT=<TOKEN_SCRIPT_ACCOUNT> \
./scripts/run-sdk-validator-suites.sh \
  --network mainnet \
  --program-id "$FIVE_PROGRAM_ID" \
  --vm-state <VM_STATE_PDA> \
  --keypair "$FIVE_KEYPAIR_PATH" \
  --scenarios token_full_e2e
```

## 6) Broad Rollout (Core Five)
Projects:
- `5ive-amm`
- `5ive-cfd`
- `5ive-escrow`
- `5ive-lending`
- `5ive-token`

Each project has a `deployment-config.mainnet.json` scaffold. Populate script accounts and timestamps during rollout.

Recommended verify commands (per project):
```bash
cd <project>
npm run build
npm run test
npm run test:onchain:mainnet
```

## 7) Rollback Commands
- Keep SDK/client overrides on explicit env (`FIVE_PROGRAM_ID`) until canary is stable.
- To roll back fees quickly:
```bash
node scripts/vm-state-set-fees.mjs \
  --cluster mainnet \
  --rpc-url "$FIVE_RPC_URL" \
  --program-id "$FIVE_PROGRAM_ID" \
  --deploy-fee 0 \
  --execute-fee 0
```
- To revert client routing, set `FIVE_NETWORK=devnet` and explicit `FIVE_PROGRAM_ID=<DEVNET_PROGRAM_ID>` for operational scripts.

## 8) Evidence Bundle
Create an evidence bundle after rollout:
```bash
scripts/mainnet-rollout-report.sh
```
Output location:
- `.reports/mainnet/<timestamp>/`
