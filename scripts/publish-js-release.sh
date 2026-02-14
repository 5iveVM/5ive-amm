#!/usr/bin/env bash
set -euo pipefail

# 5IVE JS Release Script
# Order: five-wasm -> five-sdk -> five-cli
#
# Usage:
#   ./scripts/publish-js-release.sh --dry-run
#   ./scripts/publish-js-release.sh --publish
#
# Token options (first match wins):
# 1) Paste token into NPM_TOKEN_INLINE below (manual one-off; remove after use)
# 2) Export NPM_TOKEN in shell
# 3) Interactive prompt (hidden)

NPM_TOKEN_INLINE=""

MODE="dry-run"
if [[ "${1:-}" == "--publish" ]]; then
  MODE="publish"
elif [[ "${1:-}" == "--dry-run" || -z "${1:-}" ]]; then
  MODE="dry-run"
else
  echo "Usage: $0 [--dry-run|--publish]"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
WASM_DIR="$ROOT_DIR/five-wasm"
SDK_DIR="$ROOT_DIR/five-sdk"
CLI_DIR="$ROOT_DIR/five-cli"

echo "==> Mode: $MODE"
echo "==> Repo: $ROOT_DIR"

if ! command -v npm >/dev/null 2>&1; then
  echo "ERROR: npm not found in PATH"
  exit 1
fi

# Resolve token
NPM_TOKEN_RESOLVED="${NPM_TOKEN_INLINE:-}"
if [[ -z "$NPM_TOKEN_RESOLVED" ]]; then
  NPM_TOKEN_RESOLVED="${NPM_TOKEN:-}"
fi
if [[ -z "$NPM_TOKEN_RESOLVED" ]]; then
  read -r -s -p "Enter npm granular token: " NPM_TOKEN_RESOLVED
  echo
fi
if [[ -z "$NPM_TOKEN_RESOLVED" ]]; then
  echo "ERROR: npm token is required"
  exit 1
fi

# Temporary npm auth config (avoids persisting token in ~/.npmrc)
TMP_NPMRC="$(mktemp)"
cleanup() {
  rm -f "$TMP_NPMRC"
}
trap cleanup EXIT

cat > "$TMP_NPMRC" <<EONPMRC
registry=https://registry.npmjs.org/
//registry.npmjs.org/:_authToken=${NPM_TOKEN_RESOLVED}
EONPMRC

export NPM_CONFIG_USERCONFIG="$TMP_NPMRC"

echo "==> Verifying npm auth"
npm whoami >/dev/null

# 1) Build five-wasm (latest compiler -> wasm outputs)
echo "==> [1/7] Building five-wasm (node + bundler)"
npm --prefix "$WASM_DIR" run build:nodejs
npm --prefix "$WASM_DIR" run build:bundler

# 2) Build five-sdk (includes wasm sync)
echo "==> [2/7] Building five-sdk"
npm --prefix "$SDK_DIR" run build

# 3) Build five-cli (includes wasm sync)
echo "==> [3/7] Building five-cli"
npm --prefix "$CLI_DIR" run build

# 4) Verify wasm sync consistency
echo "==> [4/7] Verifying wasm sync"
bash "$ROOT_DIR/scripts/verify-wasm-sync.sh"

# 5) Validate SDK package contents
echo "==> [5/7] Validating five-sdk package contents"
SDK_PACK_OUT="$(cd "$SDK_DIR" && npm pack --dry-run 2>&1)"
echo "$SDK_PACK_OUT" | rg -q "dist/" || {
  echo "ERROR: five-sdk dry-run pack missing dist/ content"
  exit 1
}

# 6) Validate CLI package contents (must include all AGENTS templates)
echo "==> [6/7] Validating five-cli package contents"
CLI_PACK_OUT="$(cd "$CLI_DIR" && npm pack --dry-run 2>&1)"
for f in "templates/AGENTS.md" "templates/AGENTS_CHECKLIST.md" "templates/AGENTS_REFERENCE.md"; do
  echo "$CLI_PACK_OUT" | rg -q "$f" || {
    echo "ERROR: five-cli package missing $f"
    exit 1
  }
done

if [[ "$MODE" == "dry-run" ]]; then
  echo "==> [7/7] Dry-run complete. No publish performed."
  echo "Run with --publish to publish five-sdk then five-cli."
  exit 0
fi

# 7) Publish in dependency order: SDK first, then CLI
echo "==> [7/7] Publishing packages"
echo "==> Publishing @5ive-tech/sdk"
(cd "$SDK_DIR" && npm publish --access public)

echo "==> Publishing @5ive-tech/cli"
(cd "$CLI_DIR" && npm publish --access public)

echo "==> Publish complete."
