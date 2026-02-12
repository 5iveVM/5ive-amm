#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BASELINE_COMMIT="${FIVE_BENCH_BASELINE_COMMIT:-local}"
export FIVE_BENCH_BASELINE_COMMIT="$BASELINE_COMMIT"

echo "[bench] baseline commit: $FIVE_BENCH_BASELINE_COMMIT"

echo "[bench] building SBF"
cargo-build-sbf --manifest-path five-solana/Cargo.toml

echo "[bench] running opcode micro CU suite"
cargo test -p five --test runtime_bpf_opcode_micro_cu_tests -- --nocapture

echo "[bench] running scenario CU suite"
cargo test -p five --test runtime_bpf_cu_tests -- --nocapture
