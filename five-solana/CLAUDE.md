# five-solana/CLAUDE.md

This file documents the authoritative BPF-CU benchmark workflow for the Solana program crate.

## Benchmark Entry Points

- Unified runner: `scripts/ci-bpf-bench.sh` (repo root)
- Micro opcode suite: `five-solana/tests/runtime_bpf_opcode_micro_cu_tests.rs`
- Scenario suite: `five-solana/tests/runtime_bpf_cu_tests.rs`
- Perf utilities: `five-solana/tests/harness/perf.rs`

## Standard Commands

Run from repo root:

```bash
# Build SBF + run micro + scenario suites
./scripts/ci-bpf-bench.sh

# Select a specific baseline snapshot key
FIVE_BENCH_BASELINE_COMMIT=<baseline-key> ./scripts/ci-bpf-bench.sh

# Run suites independently
cargo test -p five --test runtime_bpf_opcode_micro_cu_tests -- --nocapture
cargo test -p five --test runtime_bpf_cu_tests -- --nocapture
```

## Output Contract

Keep these lines stable for tooling/parsing:

- `BENCH family=<...> opcode=<...> variant=<...> deploy=<...> execute=<...> total=<...>`
- `SCENARIO name=<...> execute=<...> total=<...>`

## Baseline and Allowlist

- Baselines: `five-solana/tests/benchmarks/baseline/<commit>.json`
- Allowlist: `five-solana/tests/benchmarks/allowlist/<commit>.json`

Regression behavior:

- If baseline file or entry is missing, harness prints `baseline_missing` or `baseline_entry_missing` and continues.
- If a baseline exists, CU regressions in `deploy`, `execute`, or `total` fail unless allowlisted.

## Performance Rules

1. BPF CU is the source of truth.
2. Preserve semantics and security checks (signer/writable/owner/bounds).
3. Add/adjust micro and scenario coverage before landing optimizations.
4. No silent rebaseline: update baseline intentionally with rationale.

## Scenario Notes

- `scenario_high_cpi_density_bpf_compute_units` and `scenario_memory_string_heavy_bpf_compute_units` are active.
- `scenario_high_external_call_fanout_bpf_compute_units` is currently a regression hook line; heavy external fanout is still exercised by:
  - `external_token_transfer_burst_non_cpi_bpf_compute_units`
  - `external_token_transfer_mass_non_cpi_bpf_compute_units`
