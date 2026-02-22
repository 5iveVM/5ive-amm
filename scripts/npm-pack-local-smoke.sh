#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
SDK_DIR="$ROOT_DIR/five-sdk"
CLI_DIR="$ROOT_DIR/five-cli"

TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/five-pack-smoke-XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT

if [ ! -f "$SDK_DIR/package.json" ] || [ ! -f "$CLI_DIR/package.json" ]; then
  echo "Missing five-sdk or five-cli package directory."
  exit 1
fi

echo "==> Packing local SDK"
SDK_TGZ="$(cd "$SDK_DIR" && npm pack --silent)"

echo "==> Packing local CLI"
CLI_TGZ="$(cd "$CLI_DIR" && npm pack --silent)"

WORK_DIR="$TMP_ROOT/work"
mkdir -p "$WORK_DIR"
cd "$WORK_DIR"

echo "==> Installing packed SDK + CLI"
npm init -y >/dev/null 2>&1
npm install "$SDK_DIR/$SDK_TGZ" "$CLI_DIR/$CLI_TGZ" >/dev/null

CLI_BIN="$WORK_DIR/node_modules/.bin/5ive"
PROJECT_DIR="$WORK_DIR/app"

echo "==> Running init/build/compile smoke"
"$CLI_BIN" init "$PROJECT_DIR" --no-git >/dev/null
"$CLI_BIN" build --project "$PROJECT_DIR" >/dev/null

cat > "$PROJECT_DIR/src/stdlib-interface-smoke.v" <<'EOF'
script main {
  use std::interfaces::spl_token;
  pub fn run(source: Account, destination: Account, authority: Account) {
    spl_token::transfer(source, destination, authority, 1);
    std::interfaces::spl_token::approve(source, destination, authority, 1);
  }
}
EOF

"$CLI_BIN" compile "$PROJECT_DIR/src/stdlib-interface-smoke.v" \
  -o "$PROJECT_DIR/build/stdlib-interface-smoke.five" >/dev/null

cat > "$PROJECT_DIR/src/stdlib-interface-legacy.v" <<'EOF'
script main {
  use std::interfaces::spl_token;
  pub fn run(source: Account, destination: Account, authority: Account) {
    SPLToken.transfer(source, destination, authority, 1);
  }
}
EOF

if "$CLI_BIN" compile "$PROJECT_DIR/src/stdlib-interface-legacy.v" \
  -o "$PROJECT_DIR/build/stdlib-interface-legacy.five" >/dev/null 2>&1; then
  echo "Expected legacy object-style call to fail, but compile succeeded."
  exit 1
fi

echo "Pack smoke passed."
