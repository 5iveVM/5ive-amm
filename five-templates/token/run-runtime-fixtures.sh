#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

cd "$ROOT_DIR"
FIVE_TEMPLATE_FILTER="/five-templates/token/" \
  cargo test -p five --test runtime_template_fixture_tests -- --nocapture
