#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/4] Validate DSL coverage contracts and generate reports"
node scripts/validate-dsl-feature-matrix.mjs
cargo run -q -p five-dsl-compiler --bin feature_parity_report
node scripts/generate-dsl-exhaustive-report.mjs

echo "[2/4] Run manifest-backed compiler and execution suites"
cargo test -q -p five-dsl-compiler --test dsl_feature_matrix
cargo test -q -p five-vm-mito --test dsl_feature_matrix
cargo test -q -p five-vm-wasm --test dsl_feature_matrix
cargo test -q -p five-lsp --test dsl_feature_matrix
node scripts/run-dsl-feature-matrix-cli.mjs

echo "[3/4] Run protocol/compiler alignment suites"
cargo test -q -p five-dsl-compiler --test protocol_alignment_tests
cargo test -q -p five-protocol --features test-fixtures --test execute_payload_fixtures
cargo test -q -p five-vm-mito --test execute_payload_alignment_tests
cargo test -q -p five --test deploy_verification_tests verifier_and_parser_align_on_shared_fixtures

echo "[4/4] Run runtime harness representative suite"
cargo test -q -p five --test runtime_feature_matrix_tests

if [[ "${FIVE_REQUIRE_LOCALNET_MATRIX:-0}" == "1" ]]; then
  echo "[5/5] Run required localnet validator matrix"
  node scripts/run-dsl-validator-matrix.mjs \
    --network localnet \
    --program-id "${FIVE_PROGRAM_ID:-}" \
    --vm-state "${VM_STATE_PDA:-}" \
    --keypair "${FIVE_KEYPAIR_PATH:-$HOME/.config/solana/id.json}"
fi

if [[ "${FIVE_REQUIRE_LOCALNET_BUILTIN_MATRIX:-0}" == "1" ]]; then
  echo "[extra] Run builtin localnet validator matrix"
  node scripts/run-dsl-builtin-validator-matrix.mjs \
    --network localnet \
    --program-id "${FIVE_PROGRAM_ID:-}" \
    --vm-state "${VM_STATE_PDA:-}" \
    --keypair "${FIVE_KEYPAIR_PATH:-$HOME/.config/solana/id.json}"
fi

echo "Feature parity audit completed."
echo "Report: $ROOT_DIR/target/feature-parity/matrix.md"
echo "Builtin report: $ROOT_DIR/target/feature-parity/builtin-matrix.md"
echo "Feature inventory: $ROOT_DIR/target/feature-parity/feature-inventory.json"
