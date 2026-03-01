# MVP Full Engineering Gate Checklist

Canonical localnet command:

```bash
./scripts/mvp-release-gate.sh --cluster localnet
```

Canonical devnet command:

```bash
./scripts/mvp-release-gate.sh --cluster devnet
```

Canonical mainnet dry-run command:

```bash
./scripts/mvp-release-gate.sh --cluster mainnet
```

Canonical end-user confidence commands:

```bash
./scripts/run-user-journey-suites.sh --network localnet --program-id <local-program-id> --vm-state <local-vm-state> --token-script-account <local-token-script-account> --amm-script-account <local-amm-script-account> --lending-script-account <local-lending-script-account> --lending-oracle-script-account <local-lending-oracle-script-account>
./scripts/run-user-journey-suites.sh --network devnet --program-id <devnet-program-id> --token-script-account <devnet-token-script-account> --amm-script-account <devnet-amm-script-account> --lending-script-account <devnet-lending-script-account> --lending-oracle-script-account <devnet-lending-oracle-script-account>
```

Optional strict prebuilt-artifact mode (fail instead of auto-build):

```bash
./scripts/mvp-release-gate.sh --cluster localnet --no-build-sbf
```

## Hard Blockers (Must Pass)

1. SBF artifacts exist and are valid at:
- `target/deploy/five-keypair.json`
- `target/deploy/five.so`

2. Generated constants program ID must match deploy keypair pubkey:
- `five-solana/src/generated_constants.rs` `VM_PROGRAM_ID`
- `solana-keygen pubkey target/deploy/five-keypair.json`
- If mismatched, run:
  - `./scripts/build-five-solana-cluster.sh --cluster <localnet|devnet|mainnet>`

3. Core workspace tests pass:
- `cargo test --workspace --exclude five --quiet`

4. BPF runtime CU suites pass:
- `cargo test -p five --test runtime_bpf_opcode_micro_cu_tests -- --nocapture`
- `cargo test -p five --test runtime_bpf_cu_tests -- --nocapture`

5. End-to-end smoke validation passes on localnet and devnet:
- `cargo test -p five --test runtime_template_fixture_tests -- --nocapture`

6. Mainnet uses dry-run semantics only:
- `--cluster mainnet` auto-skips E2E smoke.
- Localnet + devnet evidence must already be attached before any mainnet release decision.

7. Gate report generated:
- `target/mvp-gate/report.json`
- `target/mvp-gate/report.md`

8. End-user confidence journey suites pass on localnet and devnet:
- `./scripts/run-user-journey-suites.sh --network localnet --program-id <local-program-id> --vm-state <local-vm-state> --token-script-account <local-token-script-account> --amm-script-account <local-amm-script-account> --lending-script-account <local-lending-script-account> --lending-oracle-script-account <local-lending-oracle-script-account>`
- `./scripts/run-user-journey-suites.sh --network devnet --program-id <devnet-program-id> --token-script-account <devnet-token-script-account> --amm-script-account <devnet-amm-script-account> --lending-script-account <devnet-lending-script-account> --lending-oracle-script-account <devnet-lending-oracle-script-account>`
- All eleven blocking scenarios must pass:
  - `wallet_onboarding`
  - `token_lifecycle_two_users`
  - `failure_recovery`
  - `resume_existing_state`
  - `duplicate_submit_safety`
  - `amm_pool_onboarding`
  - `amm_two_user_swap_lifecycle`
  - `amm_failure_recovery`
  - `lending_market_onboarding`
  - `lending_borrow_repay_lifecycle`
  - `lending_failure_recovery`
- All script accounts are cluster-specific and must be passed explicitly.
- No deployment-config fallback is allowed; missing any required script account is release-blocking.

## Preflight Parity Check

Before running the full gate, verify artifacts/constants parity:

```bash
solana-keygen pubkey target/deploy/five-keypair.json
rg -n 'pub const VM_PROGRAM_ID' five-solana/src/generated_constants.rs
```

The two program IDs must match. The gate now auto-repairs this drift unless `--no-build-sbf` is set.

## Non-Blocking / Informational

1. Compiler dead-code/deprecation warnings that do not affect correctness or gate exit status.
2. Baseline/allowlist maintenance for performance governance, if current suite already passes.

## Required Signoff Evidence

1. Attach a fresh localnet report from `./scripts/mvp-release-gate.sh --cluster localnet`.
2. Attach a fresh devnet report from `./scripts/mvp-release-gate.sh --cluster devnet`.
3. Attach a fresh mainnet dry-run report from `./scripts/mvp-release-gate.sh --cluster mainnet`.
4. Run SDK validator suites with explicit token script accounts whenever `token_full_e2e` is included:
- `./scripts/run-sdk-validator-suites.sh --network localnet --program-id <local-program-id> --vm-state <local-vm-state> --token-script-account <local-token-script-account>`
- `./scripts/run-sdk-validator-suites.sh --network devnet --program-id <devnet-program-id> --token-script-account <devnet-token-script-account>`
5. Attach a fresh localnet user-journey report from `target/user-journey-runs/<timestamp>/user-journey-report.json`.
6. Attach a fresh devnet user-journey report from `target/user-journey-runs/<timestamp>/user-journey-report.json`.
7. Confirm each engineering gate report `overall_status` is `pass`.
8. Confirm each user-journey report has `allGreen: true` and `PASS: 11`.
9. For mainnet, confirm the E2E stage notes that smoke was intentionally skipped.
10. Record cluster used (`localnet`, `devnet`, or `mainnet`) in release notes.

## Mainnet-Specific Prerequisite

Before any `--cluster mainnet` build or deploy:

1. `five-solana/constants.vm.toml` `[clusters.mainnet].program_id` must be the reserved production program ID, not a copied devnet placeholder.
2. The reserved program ID must be intentionally reviewed before release-candidate tagging.

## Environment and Toolchain Prerequisites

1. Rust toolchain with `cargo` and `rustc` available in `PATH`.
2. Solana SBF build tooling available via `cargo-build-sbf`.
3. Access to project workspace root where `scripts/mvp-release-gate.sh` is executed.
