#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$ROOT_DIR/five-wasm"
WASM_NODE_DIR="$WASM_DIR/pkg-node"
WASM_BUNDLER_DIR="$WASM_DIR/pkg-bundler"

SDK_VM_DIR="$ROOT_DIR/five-sdk/src/assets/vm"
SDK_WASM_DIR="$ROOT_DIR/five-sdk/src/assets/wasm"
CLI_VM_DIR="$ROOT_DIR/five-cli/src/assets/vm"

echo "==> Building five-wasm node and bundler outputs"
npm --prefix "$WASM_DIR" run build:nodejs
npm --prefix "$WASM_DIR" run build:bundler

copy_node_assets() {
  local dest="$1"
  mkdir -p "$dest"
  rm -f "$dest"/five_vm_wasm*
  COPYFILE_DISABLE=1 cp "$WASM_NODE_DIR"/five_vm_wasm* "$dest"/
  if [ -f "$WASM_NODE_DIR/package.json" ]; then
    COPYFILE_DISABLE=1 cp "$WASM_NODE_DIR/package.json" "$dest"/package.json
  fi
}

echo "==> Syncing node wasm artifacts to five-sdk and five-cli"
copy_node_assets "$SDK_VM_DIR"
copy_node_assets "$SDK_WASM_DIR"
copy_node_assets "$CLI_VM_DIR"

if [ ! -f "$WASM_BUNDLER_DIR/package.json" ]; then
  echo "ERROR: missing bundler package at $WASM_BUNDLER_DIR"
  exit 1
fi

echo "==> Sync complete"
