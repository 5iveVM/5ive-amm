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

NODE_FILES=(
  "five_vm_wasm.js"
  "five_vm_wasm_bg.wasm"
  "five_vm_wasm.d.ts"
  "five_vm_wasm_bg.wasm.d.ts"
)

BUNDLER_FILES=(
  "five_vm_wasm.js"
  "five_vm_wasm_bg.js"
  "five_vm_wasm_bg.wasm"
  "five_vm_wasm.d.ts"
  "five_vm_wasm_bg.wasm.d.ts"
  "package.json"
)

has_all_files() {
  local dir="$1"
  shift
  local file
  for file in "$@"; do
    if [ ! -f "$dir/$file" ]; then
      return 1
    fi
  done
  return 0
}

sources_newer_than_artifact() {
  local artifact="$1"
  shift

  if [ ! -f "$artifact" ]; then
    return 0
  fi

  local source_path
  for source_path in "$@"; do
    if [ -d "$source_path" ]; then
      if find "$source_path" -type f -newer "$artifact" -print -quit | grep -q .; then
        return 0
      fi
    elif [ -f "$source_path" ]; then
      if [ "$source_path" -nt "$artifact" ]; then
        return 0
      fi
    fi
  done

  return 1
}

should_rebuild=false
if [ "${FIVE_WASM_REBUILD:-0}" = "1" ]; then
  should_rebuild=true
fi
if [ "${1:-}" = "--rebuild" ]; then
  should_rebuild=true
fi

if [ "$should_rebuild" = false ]; then
  if ! has_all_files "$WASM_NODE_DIR" "${NODE_FILES[@]}"; then
    echo "==> Missing node wasm artifacts; rebuilding"
    should_rebuild=true
  elif ! has_all_files "$WASM_BUNDLER_DIR" "${BUNDLER_FILES[@]}"; then
    echo "==> Missing bundler wasm artifacts; rebuilding"
    should_rebuild=true
  elif sources_newer_than_artifact "$WASM_NODE_DIR/five_vm_wasm_bg.wasm" \
    "$WASM_DIR/src" \
    "$WASM_DIR/Cargo.toml" \
    "$ROOT_DIR/five-dsl-compiler/src" \
    "$ROOT_DIR/five-dsl-compiler/Cargo.toml"; then
    echo "==> Detected newer compiler/wasm sources; rebuilding"
    should_rebuild=true
  fi
fi

if [ "$should_rebuild" = true ]; then
  echo "==> Building five-wasm node and bundler outputs"
  npm --prefix "$WASM_DIR" run build:nodejs
  npm --prefix "$WASM_DIR" run build:bundler
else
  echo "==> Using existing five-wasm artifacts from pkg-node/pkg-bundler"
fi

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
