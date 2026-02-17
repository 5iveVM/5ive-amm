# MVP Warnings vs Blockers Register

- Date: 2026-02-17
- Owner: Core Protocol Engineering
- Gate mode: Full engineering gate

## Must-Fix Blockers

1. Missing SBF deploy artifacts for BPF CU/runtime tests.
- Impact: Full gate fails; CU/runtime tests cannot execute.
- Detection: Missing `target/deploy/five-keypair.json` and/or `target/deploy/five.so`.
- Enforcement: `scripts/mvp-release-gate.sh` stage `SBF Artifact Build and Validation`.
- Remediation: `./scripts/build-five-solana-cluster.sh --cluster <localnet|devnet|mainnet>`.

2. Any failure in runtime CU suites.
- Impact: Full gate fails; performance/runtime correctness validation incomplete.
- Detection:
  - `runtime_bpf_opcode_micro_cu_tests`
  - `runtime_bpf_cu_tests`
- Enforcement: `scripts/mvp-release-gate.sh` stage `BPF Runtime CU Suites`.

3. Any failure in end-to-end smoke fixture suite.
- Impact: Full gate fails; no validated end-to-end execution path.
- Detection: `runtime_template_fixture_tests` failure.
- Enforcement: `scripts/mvp-release-gate.sh` stage `E2E Smoke Validation`.

## Non-Blocking Warnings

1. Rust warnings (`dead_code`, `deprecated`) that do not cause test/build failures.
- Impact: Engineering hygiene risk, not an immediate MVP release blocker.
- Current examples:
  - Deprecated constant references in `third_party/pinocchio`.
  - Dead code warnings in `five-solana` harness/constants modules.
- Action: Track cleanup backlog; avoid converting to blocker unless policy changes.

2. Benchmark baseline/allowlist maintenance gaps when test suites still pass.
- Impact: Governance/observability gap, not an immediate blocker if CU suites are green.
- Action: Keep baselines intentional and documented in performance workflow.

## Change Control

1. Update this register on every release-candidate cycle.
2. If a warning is promoted to blocker, update this file and release checklist in the same change.
3. Do not override blocker status without explicit owner signoff.
