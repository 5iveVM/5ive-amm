#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
SDK_DIR="$ROOT_DIR/five-sdk"
CLI_DIR="$ROOT_DIR/five-cli"

if [ ! -f "$SDK_DIR/package.json" ]; then
  echo "==> Local five-sdk not found at $SDK_DIR; skipping local SDK sync"
  exit 0
fi

echo "==> Building local five-sdk"
npm --prefix "$SDK_DIR" run build

echo "==> Installing local five-sdk into five-cli (no-save)"
npm --prefix "$CLI_DIR" install --no-save "$SDK_DIR"

echo "==> Local SDK sync complete"
