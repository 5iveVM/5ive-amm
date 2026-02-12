# five-vm-mito/CLAUDE.md

This file documents VM-side performance and hotpath optimization workflow.

## Primary Goal

Minimize on-chain BPF compute units while preserving VM correctness and security invariants.

## Validation Loop

Run from repo root:

```bash
# Quick compile safety for VM changes
cargo test -p five-vm-mito --lib --no-run

# Full benchmark validation (builds SBF and runs benchmark suites)
./scripts/ci-bpf-bench.sh

# Optional direct suites
cargo test -p five --test runtime_bpf_opcode_micro_cu_tests -- --nocapture
cargo test -p five --test runtime_bpf_cu_tests -- --nocapture
```

## Hotpath Targets

Prioritize inspection of:

- Dispatch and opcode grouping: `five-vm-mito/src/execution.rs`
- Stack/locals handlers: `five-vm-mito/src/handlers/stack_ops.rs`, `five-vm-mito/src/handlers/locals.rs`
- Memory/input decode paths: `five-vm-mito/src/context.rs`, `five-vm-mito/src/handlers/memory.rs`
- External/system calls: `five-vm-mito/src/handlers/functions.rs`, `five-vm-mito/src/handlers/system/invoke.rs`

## Optimization Guidelines

1. Measure first, optimize second.
2. Prefer zero-copy and slice-based decode over per-byte loops.
3. Remove redundant temporary allocations and intermediate copies.
4. Collapse repeated checks only when equivalent safety is preserved.
5. Keep signer/writable/owner checks and account bounds checks intact.
6. Re-run micro + scenario suites on every hotpath change.

## Regression Gate Policy

- Micro regressions and scenario regressions are checked via `five-solana/tests/harness/perf.rs`.
- If a baseline exists for a test, CU increases fail unless allowlisted.
- Baseline snapshots and allowlist files:
  - `five-solana/tests/benchmarks/baseline/<commit>.json`
  - `five-solana/tests/benchmarks/allowlist/<commit>.json`
