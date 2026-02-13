#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

NODE_SRC="$ROOT_DIR/five-wasm/pkg-node"
CLI_DST="$ROOT_DIR/five-cli/src/assets/vm"
SDK_VM_DST="$ROOT_DIR/five-sdk/src/assets/vm"
SDK_WASM_DST="$ROOT_DIR/five-sdk/src/assets/wasm"

FILES=(
  "five_vm_wasm.js"
  "five_vm_wasm_bg.wasm"
  "five_vm_wasm.d.ts"
  "five_vm_wasm_bg.wasm.d.ts"
)

hash_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

verify_target() {
  local src_root="$1"
  local dst_root="$2"
  local label="$3"

  for file in "${FILES[@]}"; do
    local src="$src_root/$file"
    local dst="$dst_root/$file"
    if [ ! -f "$src" ] || [ ! -f "$dst" ]; then
      echo "MISSING: $label file $file"
      exit 1
    fi

    local src_hash
    local dst_hash
    src_hash="$(hash_file "$src")"
    dst_hash="$(hash_file "$dst")"
    if [ "$src_hash" != "$dst_hash" ]; then
      echo "MISMATCH: $label/$file"
      echo "  src: $src_hash"
      echo "  dst: $dst_hash"
      exit 1
    fi
  done
}

verify_target "$NODE_SRC" "$CLI_DST" "five-cli/src/assets/vm"
verify_target "$NODE_SRC" "$SDK_VM_DST" "five-sdk/src/assets/vm"
verify_target "$NODE_SRC" "$SDK_WASM_DST" "five-sdk/src/assets/wasm"

if [ ! -f "$ROOT_DIR/five-wasm/pkg-bundler/package.json" ]; then
  echo "MISSING: five-wasm/pkg-bundler/package.json"
  exit 1
fi

BUNDLER_FILES=(
  "five_vm_wasm.js"
  "five_vm_wasm_bg.js"
  "five_vm_wasm_bg.wasm"
  "five_vm_wasm.d.ts"
  "five_vm_wasm_bg.wasm.d.ts"
)

for file in "${BUNDLER_FILES[@]}"; do
  if [ ! -f "$ROOT_DIR/five-wasm/pkg-bundler/$file" ]; then
    echo "MISSING: five-wasm/pkg-bundler/$file"
    exit 1
  fi
done

echo "WASM sync verified: CLI, SDK vm/wasm assets match five-wasm/pkg-node."
