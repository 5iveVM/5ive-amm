#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
CLI_DIR="$ROOT_DIR/five-cli"

REQUIRED_STDLIB_FILES=(
  "assets/stdlib/std/types.v"
  "assets/stdlib/std/interfaces/spl_token.v"
  "assets/stdlib/std/interfaces/system_program.v"
  "assets/stdlib/std/interfaces/stake_program.v"
)

for rel in "${REQUIRED_STDLIB_FILES[@]}"; do
  if [ ! -f "$CLI_DIR/$rel" ]; then
    echo "MISSING: five-cli/$rel"
    exit 1
  fi
done

CANONICAL_SPL="$ROOT_DIR/five-stdlib/std/interfaces/spl_token.v"
PACKAGED_SPL="$CLI_DIR/assets/stdlib/std/interfaces/spl_token.v"
if [ -f "$CANONICAL_SPL" ] && ! diff -q "$CANONICAL_SPL" "$PACKAGED_SPL" >/dev/null; then
  echo "STDLIB DRIFT: five-cli/assets/stdlib/std/interfaces/spl_token.v differs from five-stdlib/std/interfaces/spl_token.v"
  exit 1
fi

if [ ! -f "$CLI_DIR/dist/index.js" ]; then
  echo "MISSING: five-cli/dist/index.js (run build first)"
  exit 1
fi

TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/five-stdlib-smoke-XXXXXX")"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

mkdir -p "$TMP_DIR/src"

cat > "$TMP_DIR/five.toml" <<'EOF'
schema_version = 1

[project]
name = "stdlib-packaging-smoke"
version = "0.1.0"
source_dir = "src"
build_dir = "build"
entry_point = "src/main.v"
target = "vm"

[dependencies]
std = { package = "@5ive/std", version = "0.1.0", source = "bundled", link = "inline" }
EOF

cat > "$TMP_DIR/src/main.v" <<'EOF'
use std::interfaces::stake_program;

pub test_call(
    stake_account: account @mut,
    clock_sysvar: account,
    authority: account @signer,
    new_authority: account @signer
) {
    let kind: u32 = 1;
    stake_program::authorize_checked(
        stake_account,
        clock_sysvar,
        authority,
        new_authority,
        kind
    );
}
EOF

(
  cd "$CLI_DIR"
  node ./dist/index.js build --project "$TMP_DIR" >/dev/null
)

if ! find "$TMP_DIR/build" -maxdepth 1 -type f \( -name '*.five' -o -name '*.bin' \) | grep -q .; then
  echo "STDLIB smoke compile did not produce a build artifact"
  exit 1
fi

echo "Stdlib packaging verified: required files present and interface import compiles."
