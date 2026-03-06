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
  "5ive-escrow"
  "5ive-lending"
  "5ive-token"
)

INFORMATIONAL_PROJECTS=(
)

LOCAL_VERIFY_SCRIPT="${ROOT_DIR}/scripts/verify-5ive-projects.sh"

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
export FIVE_VM_CLUSTER="devnet"
export FIVE_RPC_URL="${DEVNET_RPC}"
export FIVE_PROGRAM_ID
export FIVE_KEYPAIR_PATH="${PAYER_PATH}"
export FIVE_VM_PROGRAM_ID="${FIVE_PROGRAM_ID}"
export FIVE_PAYER_PATH="${FIVE_PAYER_PATH:-${PAYER_PATH}}"
export PATH="${ROOT_DIR}/.codex-bin:${PATH}"
derived_vm_state="$(solana find-program-derived-address "${FIVE_PROGRAM_ID}" string:vm_state 2>/dev/null | awk '{print $1}' || true)"
if [[ -n "${derived_vm_state}" ]]; then
  export VM_STATE_PDA="${derived_vm_state}"
fi

DEVNET_OK=true
DEVNET_SKIP_REASON=""
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

check_devnet() {
  echo "==> Devnet preflight checks"

  if ! curl -sf -X POST "${DEVNET_RPC}" \
      -H "Content-Type: application/json" \
      -d '{"jsonrpc":"2.0","id":1,"method":"getHealth"}' \
      --max-time 5 >/dev/null 2>&1; then
    echo "    [SKIP] RPC not reachable at ${DEVNET_RPC}"
    DEVNET_OK=false
    DEVNET_SKIP_REASON="RPC not reachable at ${DEVNET_RPC}"
    return
  fi
  echo "    [OK] RPC reachable"

  local program_info
  program_info="$(curl -sf -X POST "${DEVNET_RPC}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getAccountInfo\",\"params\":[\"${FIVE_PROGRAM_ID}\",{\"encoding\":\"base64\"}]}" \
    --max-time 8 2>/dev/null || echo "{}")"
  if echo "${program_info}" | grep -q '"value":null'; then
    if ! solana account "${FIVE_PROGRAM_ID}" --url "${DEVNET_RPC}" >/dev/null 2>&1; then
      echo "    [SKIP] VM program not found: ${FIVE_PROGRAM_ID}"
      DEVNET_OK=false
      DEVNET_SKIP_REASON="VM program not found: ${FIVE_PROGRAM_ID}"
      return
    fi
  fi
  echo "    [OK] VM program exists: ${FIVE_PROGRAM_ID}"

  if [[ -f "${PAYER_PATH}" ]]; then
    local payer_pubkey
    payer_pubkey="$(solana-keygen pubkey "${PAYER_PATH}" 2>/dev/null || echo "")"
    if [[ -n "${payer_pubkey}" ]]; then
      local balance_resp balance
      balance_resp="$(curl -sf -X POST "${DEVNET_RPC}" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"getBalance\",\"params\":[\"${payer_pubkey}\"]}" \
        --max-time 8 2>/dev/null || echo "{}")"
      balance="$(echo "${balance_resp}" | grep -o '"value":[0-9]*' | head -1 | cut -d: -f2 || echo "0")"
      if [[ "${balance:-0}" -lt 1000000 ]]; then
        echo "    [WARN] Payer balance low (${balance} lamports): ${payer_pubkey}"
      else
        echo "    [OK] Payer visible (${balance} lamports)"
      fi
    fi
  else
    echo "    [WARN] Payer keypair not found at ${PAYER_PATH}"
  fi

  echo "    [OK] Devnet preflight passed"
}

run_step() {
  local project="$1"
  local step="$2"
  local cmd="$3"
  local release_blocking="$4"
  local require_devnet="${5:-false}"

  local safe_project safe_step log_path status blocker
  safe_project="${project//\//_}"
  safe_step="${step//:/_}"
  log_path="${LOG_DIR}/${safe_project}.${safe_step}.log"
  blocker=""

  if [[ "${require_devnet}" == "true" && "${DEVNET_OK}" != "true" ]]; then
    printf 'Skipped: %s\n' "${DEVNET_SKIP_REASON}" > "${log_path}"
    status="skip"
    blocker="${DEVNET_SKIP_REASON}"
  else
    if (
      cd "${ROOT_DIR}/${project}"
      bash -c "${cmd}"
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
  if (/vm program not found|program id|invalid program argument|vm program|0x1e7a/.test(haystack)) return 'program_id';
  if (/rpc not reachable|429|rpc|timed out|timeout|fetch failed|econnrefused|network error/.test(haystack)) return 'rpc';
  if (/insufficient funds|insufficient balance|balance low|lamports|airdrop/.test(haystack)) return 'funding';
  if (/owner mismatch|unauthorized|permission|must sign|not signer|signature verification failed/.test(haystack)) return 'authority';
  if (/account not found|missing account|not initialized|fixture|script account/.test(haystack)) return 'account_fixture';
  if (row.step === 'build' || row.step === 'test' || /compilererror|compile|build failed|testcases is not iterable|typescript|tsc|npm err/.test(haystack)) {
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
  phase: 'devnet-required',
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

extract_projects_from_script() {
  local script_path="$1"
  local array_name="$2"
  awk -v arr="${array_name}" '
    $0 ~ "^" arr "=\\(" { in_array = 1; next }
    in_array && /^\)/ { exit }
    in_array {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0);
      gsub(/"/, "", $0);
      if ($0 != "") print $0;
    }
  ' "${script_path}"
}

assert_projects_match_local() {
  if [[ ! -f "${LOCAL_VERIFY_SCRIPT}" ]]; then
    echo "    [WARN] Local verify script not found: ${LOCAL_VERIFY_SCRIPT}"
    return
  fi

  local local_blocking local_info dev_blocking dev_info
  local_blocking="$(extract_projects_from_script "${LOCAL_VERIFY_SCRIPT}" "BLOCKING_PROJECTS" | tr '\n' ',')"
  local_info="$(extract_projects_from_script "${LOCAL_VERIFY_SCRIPT}" "INFORMATIONAL_PROJECTS" | tr '\n' ',')"
  if [[ ${#BLOCKING_PROJECTS[@]} -gt 0 ]]; then
    dev_blocking="$(printf '%s\n' "${BLOCKING_PROJECTS[@]}" | tr '\n' ',')"
  else
    dev_blocking=""
  fi
  if [[ ${#INFORMATIONAL_PROJECTS[@]} -gt 0 ]]; then
    dev_info="$(printf '%s\n' "${INFORMATIONAL_PROJECTS[@]}" | tr '\n' ',')"
  else
    dev_info=""
  fi

  if [[ "${local_blocking}" != "${dev_blocking}" ]]; then
    echo "    [ERROR] BLOCKING_PROJECTS mismatch vs local verifier"
    echo "            local : ${local_blocking}"
    echo "            devnet: ${dev_blocking}"
    exit 2
  fi
  if [[ "${local_info}" != "${dev_info}" ]]; then
    echo "    [ERROR] INFORMATIONAL_PROJECTS mismatch vs local verifier"
    echo "            local : ${local_info}"
    echo "            devnet: ${dev_info}"
    exit 2
  fi
}

check_devnet
assert_projects_match_local

for project in "${BLOCKING_PROJECTS[@]}"; do
  echo "==> ${project} [blocking]"
  run_step "${project}" "build" "npm run build" "true"
  run_step "${project}" "test" "npm run test" "true"
  if [[ -n "${VM_STATE_PDA:-}" ]]; then
    run_step "${project}" "test:onchain:devnet" "FIVE_PROGRAM_ID=${FIVE_PROGRAM_ID} FIVE_VM_PROGRAM_ID=${FIVE_PROGRAM_ID} VM_STATE_PDA=${VM_STATE_PDA} npm run test:onchain:devnet" "true" "true"
  else
    run_step "${project}" "test:onchain:devnet" "FIVE_PROGRAM_ID=${FIVE_PROGRAM_ID} FIVE_VM_PROGRAM_ID=${FIVE_PROGRAM_ID} npm run test:onchain:devnet" "true" "true"
  fi
done

for project in "${INFORMATIONAL_PROJECTS[@]}"; do
  echo "==> ${project} [informational]"
  run_step "${project}" "build" "npm run build" "false"
  run_step "${project}" "test" "npm run test" "false"
  if [[ -n "${VM_STATE_PDA:-}" ]]; then
    run_step "${project}" "test:onchain:devnet" "FIVE_PROGRAM_ID=${FIVE_PROGRAM_ID} FIVE_VM_PROGRAM_ID=${FIVE_PROGRAM_ID} VM_STATE_PDA=${VM_STATE_PDA} npm run test:onchain:devnet" "false" "true"
  else
    run_step "${project}" "test:onchain:devnet" "FIVE_PROGRAM_ID=${FIVE_PROGRAM_ID} FIVE_VM_PROGRAM_ID=${FIVE_PROGRAM_ID} npm run test:onchain:devnet" "false" "true"
  fi
done

summarize_report

echo "Wrote ${REPORT_FILE}"
echo "Logs:  ${LOG_DIR}"

if [[ "${BLOCKING_ISSUES}" -gt 0 ]]; then
  exit 1
fi
