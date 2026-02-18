#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="$ROOT_DIR/.reports"
REPORT_FILE="$REPORT_DIR/5ive-validation.json"
TMP_FILE="$REPORT_DIR/5ive-validation.local.ndjson"

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

run_step() {
  local project="$1"
  local step="$2"
  local cmd="$3"
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
  run_step "$project" "test:onchain:local" "npm run test:onchain:local"

  if [[ -d "$ROOT_DIR/$project/client" ]]; then
    run_step "$project" "client:run:local" "npm run client:run:local"
  fi
done

node - <<'NODE' "$TMP_FILE" "$REPORT_FILE"
const fs = require('fs');
const [inFile, outFile] = process.argv.slice(2);
const rows = fs.readFileSync(inFile, 'utf8').trim().split('\n').filter(Boolean).map((l) => JSON.parse(l));
let ok = true;
for (const r of rows) if (r.status !== 'pass') ok = false;
const out = {
  generatedAt: new Date().toISOString(),
  phase: 'local-required',
  summary: { pass: rows.filter(r => r.status === 'pass').length, fail: rows.filter(r => r.status !== 'pass').length },
  allGreen: ok,
  results: rows
};
fs.writeFileSync(outFile, JSON.stringify(out, null, 2) + '\n');
NODE

echo "Wrote $REPORT_FILE"
