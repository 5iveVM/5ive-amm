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
4. Confirm each report `overall_status` is `pass`.
5. For mainnet, confirm the E2E stage notes that smoke was intentionally skipped.
6. Record cluster used (`localnet`, `devnet`, or `mainnet`) in release notes.

## Mainnet-Specific Prerequisite

Before any `--cluster mainnet` build or deploy:

1. `five-solana/constants.vm.toml` `[clusters.mainnet].program_id` must be the reserved production program ID, not a copied devnet placeholder.
2. The reserved program ID must be intentionally reviewed before release-candidate tagging.

## Environment and Toolchain Prerequisites

1. Rust toolchain with `cargo` and `rustc` available in `PATH`.
2. Solana SBF build tooling available via `cargo-build-sbf`.
3. Access to project workspace root where `scripts/mvp-release-gate.sh` is executed.
