#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="$ROOT_DIR/.reports"
REPORT_FILE="$REPORT_DIR/5ive-validation.json"
TMP_FILE="$REPORT_DIR/5ive-validation.local.ndjson"

mkdir -p "$REPORT_DIR"
: > "$TMP_FILE"

# Localnet configuration
LOCALNET_RPC="${FIVE_RPC_URL:-http://127.0.0.1:8899}"
FIVE_PROGRAM_ID="${FIVE_PROGRAM_ID:-HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg}"
PAYER_PATH="${FIVE_KEYPAIR_PATH:-$HOME/.config/solana/id.json}"

PROJECTS=(
  "5ive-amm"
  "5ive-cfd"
  "5ive-esccrow"
  "5ive-lending"
  "5ive-lending-2"
  "5ive-lending-3"
  "5ive-lending-4"
  "5ive-token"
  "5ive-token-2"
)

# ============================================================================
# Localnet preflight checks
# ============================================================================

LOCALNET_OK=true
LOCALNET_SKIP_REASON=""

check_localnet() {
  echo "==> Localnet preflight checks"

  # 1) RPC reachable
  if ! curl -sf -X POST "$LOCALNET_RPC" \
      -H "Content-Type: application/json" \
      -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
      --max-time 3 >/dev/null 2>&1; then
    echo "    [SKIP] RPC not reachable at $LOCALNET_RPC"
    LOCALNET_OK=false
    LOCALNET_SKIP_REASON="RPC not reachable at $LOCALNET_RPC"
    return
  fi
  echo "    [OK] RPC reachable"

  # 2) VM program exists
  local program_info
  program_info=$(curl -sf -X POST "$LOCALNET_RPC" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"$FIVE_PROGRAM_ID\",{\"encoding\":\"base64\"}]}" \
    --max-time 5 2>/dev/null || echo "{}")
  if echo "$program_info" | grep -q '"value":null'; then
    echo "    [SKIP] VM program not found: $FIVE_PROGRAM_ID"
    LOCALNET_OK=false
    LOCALNET_SKIP_REASON="VM program not found: $FIVE_PROGRAM_ID"
    return
  fi
  echo "    [OK] VM program exists: $FIVE_PROGRAM_ID"

  # 3) Payer funded
  if [[ -f "$PAYER_PATH" ]]; then
    local payer_pubkey
    payer_pubkey=$(solana-keygen pubkey "$PAYER_PATH" 2>/dev/null || echo "")
    if [[ -n "$payer_pubkey" ]]; then
      local balance_resp
      balance_resp=$(curl -sf -X POST "$LOCALNET_RPC" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getBalance\",\"params\":[\"$payer_pubkey\"]}" \
        --max-time 5 2>/dev/null || echo "{}")
      local balance
      balance=$(echo "$balance_resp" | grep -o '"value":[0-9]*' | head -1 | cut -d: -f2 || echo "0")
      if [[ "${balance:-0}" -lt 1000000 ]]; then
        echo "    [WARN] Payer balance low ($balance lamports): $payer_pubkey"
      else
        echo "    [OK] Payer funded ($balance lamports)"
      fi
    fi
  else
    echo "    [WARN] Payer keypair not found at $PAYER_PATH"
  fi

  echo "    [OK] Localnet preflight passed"
}

check_localnet

# ============================================================================
# Per-project validation
# ============================================================================

run_step() {
  local project="$1"
  local step="$2"
  local cmd="$3"
  local require_localnet="${4:-false}"

  if [[ "$require_localnet" == "true" && "$LOCALNET_OK" != "true" ]]; then
    echo "{\"project\":\"$project\",\"network\":\"localnet\",\"step\":\"$step\",\"status\":\"skip\",\"blocker\":\"$LOCALNET_SKIP_REASON\"}" >> "$TMP_FILE"
    return
  fi

  if (cd "$ROOT_DIR/$project" && bash -lc "$cmd"); then
    echo "{\"project\":\"$project\",\"network\":\"localnet\",\"step\":\"$step\",\"status\":\"pass\"}" >> "$TMP_FILE"
  else
    echo "{\"project\":\"$project\",\"network\":\"localnet\",\"step\":\"$step\",\"status\":\"fail\",\"blocker\":\"$step failed\"}" >> "$TMP_FILE"
  fi
}

for project in "${PROJECTS[@]}"; do
  echo "==> $project"
  run_step "$project" "build" "npm run build"
  run_step "$project" "test" "npm run test"
  run_step "$project" "test:onchain:local" "npm run test:onchain:local" "true"

  if [[ -d "$ROOT_DIR/$project/client" ]]; then
    run_step "$project" "client:run:local" "npm run client:run:local" "true"
  fi
done

node - <<'NODE' "$TMP_FILE" "$REPORT_FILE"
const fs = require('fs');
const [inFile, outFile] = process.argv.slice(2);
const rows = fs.readFileSync(inFile, 'utf8').trim().split('\n').filter(Boolean).map((l) => JSON.parse(l));
let ok = true;
for (const r of rows) if (r.status === 'fail') ok = false;
const out = {
  generatedAt: new Date().toISOString(),
  phase: 'local-required',
  summary: {
    pass: rows.filter(r => r.status === 'pass').length,
    fail: rows.filter(r => r.status === 'fail').length,
    skip: rows.filter(r => r.status === 'skip').length
  },
  allGreen: ok,
  results: rows
};
fs.writeFileSync(outFile, JSON.stringify(out, null, 2) + '\n');
NODE

echo "Wrote $REPORT_FILE"
