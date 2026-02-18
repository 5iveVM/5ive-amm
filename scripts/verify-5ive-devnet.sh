#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="$ROOT_DIR/.reports"
REPORT_FILE="$REPORT_DIR/5ive-validation-devnet.json"
TMP_FILE="$REPORT_DIR/5ive-validation.devnet.ndjson"

mkdir -p "$REPORT_DIR"
: > "$TMP_FILE"

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

track_step() {
  local project="$1"
  local step="$2"
  local cmd="$3"
  if (cd "$ROOT_DIR/$project" && bash -lc "$cmd"); then
    echo "{\"project\":\"$project\",\"network\":\"devnet\",\"step\":\"$step\",\"status\":\"pass\"}" >> "$TMP_FILE"
  else
    echo "{\"project\":\"$project\",\"network\":\"devnet\",\"step\":\"$step\",\"status\":\"blocked\",\"blocker\":\"$step failed on devnet (inspect logs and funding/program-id/rpc)\"}" >> "$TMP_FILE"
  fi
}

for project in "${PROJECTS[@]}"; do
  echo "==> $project"
  track_step "$project" "test:onchain:devnet" "npm run test:onchain:devnet"

  if [[ -d "$ROOT_DIR/$project/client" ]]; then
    track_step "$project" "client:run:devnet" "npm run client:run:devnet"
  fi
done

node - <<'NODE' "$TMP_FILE" "$REPORT_FILE"
const fs = require('fs');
const [inFile, outFile] = process.argv.slice(2);
const rows = fs.readFileSync(inFile, 'utf8').trim().split('\n').filter(Boolean).map((l) => JSON.parse(l));
const out = {
  generatedAt: new Date().toISOString(),
  phase: 'devnet-tracked',
  summary: {
    pass: rows.filter(r => r.status === 'pass').length,
    blocked: rows.filter(r => r.status === 'blocked').length
  },
  results: rows
};
fs.writeFileSync(outFile, JSON.stringify(out, null, 2) + '\n');
NODE

echo "Wrote $REPORT_FILE"
