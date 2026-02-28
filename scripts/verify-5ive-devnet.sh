#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${ROOT_DIR}/.reports"
REPORT_FILE="${REPORT_DIR}/5ive-validation-devnet.json"
TS="$(date +%Y%m%d-%H%M%S)"
TMP_FILE="${REPORT_DIR}/5ive-validation.devnet.${TS}.tsv"
LOG_DIR="${REPORT_DIR}/devnet/${TS}/logs"

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

DEFAULT_DEVNET_PROGRAM_ID="$(resolve_cluster_program_id "devnet")"
DEVNET_RPC="${FIVE_RPC_URL:-https://api.devnet.solana.com}"
FIVE_PROGRAM_ID="${FIVE_PROGRAM_ID:-${DEFAULT_DEVNET_PROGRAM_ID}}"
PAYER_PATH="${FIVE_KEYPAIR_PATH:-$HOME/.config/solana/id.json}"

export FIVE_NETWORK="devnet"
export FIVE_RPC_URL="${DEVNET_RPC}"
export FIVE_PROGRAM_ID
export FIVE_KEYPAIR_PATH="${PAYER_PATH}"
export FIVE_VM_PROGRAM_ID="${FIVE_VM_PROGRAM_ID:-${FIVE_PROGRAM_ID}}"
export FIVE_PAYER_PATH="${FIVE_PAYER_PATH:-${PAYER_PATH}}"

BLOCKING_ISSUES=0

record_row() {
  local project="$1"
  local step="$2"
  local status="$3"
  local release_blocking="$4"
  local log_path="$5"
  local blocker="$6"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${project}" "devnet" "${step}" "${status}" "${release_blocking}" "${log_path}" "${blocker}" >> "${TMP_FILE}"
}

track_step() {
  local project="$1"
  local step="$2"
  local cmd="$3"
  local release_blocking="$4"

  local safe_project safe_step log_path status blocker
  safe_project="${project//\//_}"
  safe_step="${step//:/_}"
  log_path="${LOG_DIR}/${safe_project}.${safe_step}.log"
  blocker=""

  if (
    cd "${ROOT_DIR}/${project}"
    bash -lc "${cmd}"
  ) > "${log_path}" 2>&1; then
    status="pass"
  else
    status="blocked"
    blocker="${step} failed on devnet"
  fi

  if [[ "${status}" == "pass" ]]; then
    echo "    [PASS] ${step}"
  else
    echo "    [BLOCKED] ${step}"
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
  if (/program id|invalid program argument|vm program|0x1e7a/.test(haystack)) return 'program_id';
  if (/429|rpc|timed out|timeout|fetch failed|econnrefused|network error/.test(haystack)) return 'rpc';
  if (/insufficient funds|insufficient balance|lamports|airdrop/.test(haystack)) return 'funding';
  if (/owner mismatch|unauthorized|permission|must sign|not signer|signature verification failed/.test(haystack)) return 'authority';
  if (/account not found|missing account|not initialized|fixture|script account/.test(haystack)) return 'account_fixture';
  if (row.step === 'build' || row.step === 'test' || /compilererror|compile|build failed|typescript|tsc|npm err/.test(haystack)) {
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
  phase: 'devnet-tracked',
  artifacts: {
    logDir,
  },
  summary: summarize(enriched, ['pass', 'blocked']),
  blockingSummary: summarize(blockingRows, ['pass', 'blocked']),
  informationalSummary: summarize(informationalRows, ['pass', 'blocked']),
  allGreen,
  results: enriched,
};

fs.writeFileSync(outFile, JSON.stringify(out, null, 2) + '\n');
NODE
}

for project in "${BLOCKING_PROJECTS[@]}"; do
  echo "==> ${project} [blocking]"
  track_step "${project}" "test:onchain:devnet" "npm run test:onchain:devnet" "true"

  if [[ -d "${ROOT_DIR}/${project}/client" ]]; then
    track_step "${project}" "client:run:devnet" "npm run client:run:devnet" "true"
  fi
done

for project in "${INFORMATIONAL_PROJECTS[@]}"; do
  echo "==> ${project} [informational]"
  track_step "${project}" "test:onchain:devnet" "npm run test:onchain:devnet" "false"

  if [[ -d "${ROOT_DIR}/${project}/client" ]]; then
    track_step "${project}" "client:run:devnet" "npm run client:run:devnet" "false"
  fi
done

summarize_report

echo "Wrote ${REPORT_FILE}"
echo "Logs:  ${LOG_DIR}"

if [[ "${BLOCKING_ISSUES}" -gt 0 ]]; then
  exit 1
fi
