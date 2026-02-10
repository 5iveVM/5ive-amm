#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT_DIR"

cargo test -p five \
  --test runtime_smoke_tests \
  --test runtime_fee_and_validation_tests \
  --test runtime_script_fixture_tests \
  --test runtime_template_fixture_tests \
  --test runtime_syscall_cpi_tests \
  -- --nocapture
