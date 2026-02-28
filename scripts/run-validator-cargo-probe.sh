#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TEST_FILE="${1:-}"
TEST_NAME="${2:-}"
OUTPUT_PATH="${3:-}"

if [[ -z "$TEST_FILE" || -z "$TEST_NAME" || -z "$OUTPUT_PATH" ]]; then
  echo "usage: $0 <test-file> <test-name> <output-path>" >&2
  exit 2
fi

mkdir -p "$(dirname "$OUTPUT_PATH")"

if FIVE_CU_PROBE_OUTPUT_PATH="$OUTPUT_PATH" cargo test -q -p five --features validator-harness --test "$TEST_FILE" "$TEST_NAME" -- --ignored --nocapture; then
  exit 0
fi

exit 1
