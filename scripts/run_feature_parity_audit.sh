#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/4] Generate feature parity matrix report"
cargo run -q -p five-dsl-compiler --bin feature_parity_report

echo "[2/4] Run compiler discrepancy suites"
cargo test -q -p five-dsl-compiler --test lending_regression_casted_locals
cargo test -q -p five-dsl-compiler --test diagnostics_reserved_keyword_function

echo "[3/4] Run protocol/compiler alignment suites"
cargo test -q -p five-dsl-compiler --test protocol_alignment_tests
cargo test -q -p five-protocol --features test-fixtures --test execute_payload_fixtures
cargo test -q -p five-vm-mito --test execute_payload_alignment_tests
cargo test -q -p five --test deploy_verification_tests verifier_and_parser_align_on_shared_fixtures

echo "[4/4] Run runtime harness representative suite"
cargo test -q -p five --test runtime_template_fixture_tests

echo "Feature parity audit completed."
echo "Report: $ROOT_DIR/target/feature-parity/matrix.md"
