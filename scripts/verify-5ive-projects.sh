#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${ROOT_DIR}/.reports"
REPORT_FILE="${REPORT_DIR}/5ive-validation.json"
TS="$(date +%Y%m%d-%H%M%S)"
TMP_FILE="${REPORT_DIR}/5ive-validation.local.${TS}.tsv"
LOG_DIR="${REPORT_DIR}/local/${TS}/logs"

mkdir -p "${REPORT_DIR}" "${LOG_DIR}"
: > "${TMP_FILE}"

BLOCKING_PROJECTS=(
  "5ive-amm"
  "5ive-cfd"
  "5ive-esccrow"
  "5ive-lending-2"
  "5ive-token"
  "5ive-token-2"
)

INFORMATIONAL_PROJECTS=(
  "5ive-lending"
  "5ive-lending-3"
  "5ive-lending-4"
)

resolve_cluster_program_id() {
  local cluster="$1"
  awk -F'"' -v cluster="${cluster}" '
    $0 ~ "^\\[clusters\\." cluster "\\]$" { in_cluster = 1; next }
    /^\[/ && in_cluster { exit }
    in_cluster && $1 ~ /^program_id = / { print $2; exit }
  ' "${ROOT_DIR}/five-solana/constants.vm.toml"
}

DEFAULT_LOCAL_PROGRAM_ID="$(resolve_cluster_program_id "localnet")"
LOCALNET_RPC="${FIVE_RPC_URL:-http://127.0.0.1:8899}"
FIVE_PROGRAM_ID="${FIVE_PROGRAM_ID:-${DEFAULT_LOCAL_PROGRAM_ID}}"
PAYER_PATH="${FIVE_KEYPAIR_PATH:-$HOME/.config/solana/id.json}"

export FIVE_NETWORK="localnet"
export FIVE_RPC_URL="${LOCALNET_RPC}"
export FIVE_PROGRAM_ID
export FIVE_KEYPAIR_PATH="${PAYER_PATH}"
export FIVE_VM_PROGRAM_ID="${FIVE_VM_PROGRAM_ID:-${FIVE_PROGRAM_ID}}"
export FIVE_PAYER_PATH="${FIVE_PAYER_PATH:-${PAYER_PATH}}"

LOCALNET_OK=true
LOCALNET_SKIP_REASON=""
BLOCKING_ISSUES=0

record_row() {
  local project="$1"
  local step="$2"
  local status="$3"
  local release_blocking="$4"
  local log_path="$5"
  local blocker="$6"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${project}" "localnet" "${step}" "${status}" "${release_blocking}" "${log_path}" "${blocker}" >> "${TMP_FILE}"
}

check_localnet() {
  echo "==> Localnet preflight checks"

  if ! curl -sf -X POST "${LOCALNET_RPC}" \
      -H "Content-Type: application/json" \
      -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
      --max-time 3 >/dev/null 2>&1; then
    echo "    [SKIP] RPC not reachable at ${LOCALNET_RPC}"
    LOCALNET_OK=false
    LOCALNET_SKIP_REASON="RPC not reachable at ${LOCALNET_RPC}"
    return
  fi
  echo "    [OK] RPC reachable"

  local program_info
  program_info="$(curl -sf -X POST "${LOCALNET_RPC}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"${FIVE_PROGRAM_ID}\",{\"encoding\":\"base64\"}]}" \
    --max-time 5 2>/dev/null || echo "{}")"
  if echo "${program_info}" | grep -q '"value":null'; then
    echo "    [SKIP] VM program not found: ${FIVE_PROGRAM_ID}"
    LOCALNET_OK=false
    LOCALNET_SKIP_REASON="VM program not found: ${FIVE_PROGRAM_ID}"
    return
  fi
  echo "    [OK] VM program exists: ${FIVE_PROGRAM_ID}"

  if [[ -f "${PAYER_PATH}" ]]; then
    local payer_pubkey
    payer_pubkey="$(solana-keygen pubkey "${PAYER_PATH}" 2>/dev/null || echo "")"
    if [[ -n "${payer_pubkey}" ]]; then
      local balance_resp balance
      balance_resp="$(curl -sf -X POST "${LOCALNET_RPC}" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getBalance\",\"params\":[\"${payer_pubkey}\"]}" \
        --max-time 5 2>/dev/null || echo "{}")"
      balance="$(echo "${balance_resp}" | grep -o '"value":[0-9]*' | head -1 | cut -d: -f2 || echo "0")"
      if [[ "${balance:-0}" -lt 1000000 ]]; then
        echo "    [WARN] Payer balance low (${balance} lamports): ${payer_pubkey}"
      else
        echo "    [OK] Payer funded (${balance} lamports)"
      fi
    fi
  else
    echo "    [WARN] Payer keypair not found at ${PAYER_PATH}"
  fi

  echo "    [OK] Localnet preflight passed"
}

run_step() {
  local project="$1"
  local step="$2"
  local cmd="$3"
  local release_blocking="$4"
  local require_localnet="${5:-false}"

  local safe_project safe_step log_path status blocker
  safe_project="${project//\//_}"
  safe_step="${step//:/_}"
  log_path="${LOG_DIR}/${safe_project}.${safe_step}.log"
  blocker=""

  if [[ "${require_localnet}" == "true" && "${LOCALNET_OK}" != "true" ]]; then
    printf 'Skipped: %s\n' "${LOCALNET_SKIP_REASON}" > "${log_path}"
    status="skip"
    blocker="${LOCALNET_SKIP_REASON}"
  else
    if (
      cd "${ROOT_DIR}/${project}"
      bash -lc "${cmd}"
    ) > "${log_path}" 2>&1; then
      status="pass"
    else
      status="fail"
      blocker="${step} failed"
    fi
  fi

  if [[ "${status}" == "pass" ]]; then
    echo "    [PASS] ${step}"
  else
    echo "    [${status^^}] ${step}"
  fi

  if [[ "${release_blocking}" == "true" && "${status}" != "pass" ]]; then
    BLOCKING_ISSUES=$((BLOCKING_ISSUES + 1))
  fi

  record_row "${project}" "${step}" "${status}" "${release_blocking}" "${log_path}" "${blocker}"
}

summarize_report() {
  node - <<'NODE' "${TMP_FILE}" "${REPORT_FILE}" "${LOG_DIR}"
const fs = require('fs');
const [inFile, outFile, logDir] = process.argv.slice(2);

const lines = fs.readFileSync(inFile, 'utf8').trim().split('\n').filter(Boolean);
const rows = lines.map((line) => {
  const [project, network, step, status, releaseBlockingRaw, logPath, blocker] = line.split('\t');
  return {
    project,
    network,
    step,
    status,
    releaseBlocking: releaseBlockingRaw === 'true',
    logPath,
    blocker: blocker || '',
  };
});

function readLog(logPath) {
  if (!logPath) return '';
  try {
    return fs.readFileSync(logPath, 'utf8');
  } catch {
    return '';
  }
}

function parseSignature(logText) {
  const match = logText.match(/(?:Signature|signature|sig)\s*:\s*([1-9A-HJ-NP-Za-km-z]{20,120})/);
  return match ? match[1] : null;
}

function parseMetaErr(logText) {
  const match = logText.match(/(?:Meta\.err|meta\.err)\s*:\s*(.+)/);
  return match ? match[1].trim() : null;
}

function parseComputeUnits(logText) {
  const match = logText.match(/(?:Compute units|compute units|CU)\s*:\s*([0-9]+)/);
  return match ? Number(match[1]) : null;
}

function classifyFailure(row, logText) {
  if (row.status === 'pass') return null;
  const haystack = `${row.blocker}\n${logText}`.toLowerCase();
  if (/vm program not found|program id|invalid program argument|0x1e7a/.test(haystack)) return 'program_id';
  if (/rpc not reachable|econnrefused|timed out|timeout|fetch failed|network error/.test(haystack)) return 'rpc';
  if (/insufficient funds|insufficient balance|balance low|lamports|airdrop/.test(haystack)) return 'funding';
  if (/owner mismatch|unauthorized|permission|must sign|not signer|signature verification failed/.test(haystack)) return 'authority';
  if (/account not found|missing account|not initialized|fixture/.test(haystack)) return 'account_fixture';
  if (row.step === 'build' || row.step === 'test' || /compilererror|compile|build failed|testcases is not iterable|typescript|tsc/.test(haystack)) {
    return 'compile_or_build';
  }
  return 'unknown';
}

function summarize(subset, statuses) {
  const summary = {};
  for (const status of statuses) {
    summary[status] = subset.filter((row) => row.status === status).length;
  }
  summary.total = subset.length;
  return summary;
}

const enriched = rows.map((row) => {
  const logText = readLog(row.logPath);
  return {
    ...row,
    failureClass: classifyFailure(row, logText),
    signature: parseSignature(logText),
    metaErr: parseMetaErr(logText),
    computeUnits: parseComputeUnits(logText),
  };
});

const blockingRows = enriched.filter((row) => row.releaseBlocking);
const informationalRows = enriched.filter((row) => !row.releaseBlocking);
const allGreen = blockingRows.every((row) => row.status === 'pass');

const out = {
  generatedAt: new Date().toISOString(),
  phase: 'local-required',
  artifacts: {
    logDir,
  },
  summary: summarize(enriched, ['pass', 'fail', 'skip']),
  blockingSummary: summarize(blockingRows, ['pass', 'fail', 'skip']),
  informationalSummary: summarize(informationalRows, ['pass', 'fail', 'skip']),
  allGreen,
  results: enriched,
};

fs.writeFileSync(outFile, JSON.stringify(out, null, 2) + '\n');
NODE
}

check_localnet

for project in "${BLOCKING_PROJECTS[@]}"; do
  echo "==> ${project} [blocking]"
  run_step "${project}" "build" "npm run build" "true"
  run_step "${project}" "test" "npm run test" "true"
  run_step "${project}" "test:onchain:local" "npm run test:onchain:local" "true" "true"

  if [[ -d "${ROOT_DIR}/${project}/client" ]]; then
    run_step "${project}" "client:run:local" "npm run client:run:local" "true" "true"
  fi
done

for project in "${INFORMATIONAL_PROJECTS[@]}"; do
  echo "==> ${project} [informational]"
  run_step "${project}" "build" "npm run build" "false"
  run_step "${project}" "test" "npm run test" "false"
  run_step "${project}" "test:onchain:local" "npm run test:onchain:local" "false" "true"

  if [[ -d "${ROOT_DIR}/${project}/client" ]]; then
    run_step "${project}" "client:run:local" "npm run client:run:local" "false" "true"
  fi
done

summarize_report

echo "Wrote ${REPORT_FILE}"
echo "Logs:  ${LOG_DIR}"

if [[ "${BLOCKING_ISSUES}" -gt 0 ]]; then
  exit 1
fi
