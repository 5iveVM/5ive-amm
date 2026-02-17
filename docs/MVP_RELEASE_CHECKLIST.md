# MVP Full Engineering Gate Checklist

Canonical command:

```bash
./scripts/mvp-release-gate.sh --cluster localnet
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

5. End-to-end smoke validation passes:
- `cargo test -p five --test runtime_template_fixture_tests -- --nocapture`

6. Gate report generated:
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

1. Attach `target/mvp-gate/report.json` to release decision.
2. Confirm report `overall_status` is `pass`.
3. Confirm report stage list includes all four gate stages with `status = pass`.
4. Record cluster used (`localnet`, `devnet`, or `mainnet`) in release notes.

## Environment and Toolchain Prerequisites

1. Rust toolchain with `cargo` and `rustc` available in `PATH`.
2. Solana SBF build tooling available via `cargo-build-sbf`.
3. Access to project workspace root where `scripts/mvp-release-gate.sh` is executed.
