#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NETWORK="localnet"
PROGRAM_ID="${FIVE_PROGRAM_ID:-}"
VM_STATE="${VM_STATE_PDA:-}"
KEYPAIR_PATH="${FIVE_KEYPAIR_PATH:-${HOME}/.config/solana/id.json}"
TOKEN_SCRIPT_ACCOUNT="${FIVE_TOKEN_SCRIPT_ACCOUNT:-${TOKEN_SCRIPT_ACCOUNT:-}}"
SCENARIOS="${FIVE_SCENARIOS:-token_full_e2e,cpi_spl_mint,cpi_pda_invoke,cpi_anchor_program,cpi_integration}"
RESULTS_DIR=""

usage() {
  cat <<USAGE
Usage: $0 [--network localnet|devnet] [--program-id <pubkey>] [--vm-state <pubkey>] [--keypair <path>] [--token-script-account <pubkey>] [--scenarios csv] [--results-dir path]
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="${2:-}"; shift 2 ;;
    --program-id) PROGRAM_ID="${2:-}"; shift 2 ;;
    --vm-state) VM_STATE="${2:-}"; shift 2 ;;
    --keypair) KEYPAIR_PATH="${2:-}"; shift 2 ;;
    --token-script-account) TOKEN_SCRIPT_ACCOUNT="${2:-}"; shift 2 ;;
    --scenarios) SCENARIOS="${2:-}"; shift 2 ;;
    --results-dir) RESULTS_DIR="${2:-}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 1 ;;
  esac
done

if [[ "${NETWORK}" != "localnet" && "${NETWORK}" != "devnet" ]]; then
  echo "Invalid network: ${NETWORK}" >&2
  exit 1
fi

if [[ -z "${PROGRAM_ID}" ]]; then
  echo "Missing --program-id (or FIVE_PROGRAM_ID)" >&2
  exit 1
fi

if [[ ! -f "${KEYPAIR_PATH}" ]]; then
  echo "Missing keypair file: ${KEYPAIR_PATH}" >&2
  exit 1
fi

if [[ ",${SCENARIOS}," == *",token_full_e2e,"* && -z "${TOKEN_SCRIPT_ACCOUNT}" ]]; then
  echo "Missing --token-script-account (or FIVE_TOKEN_SCRIPT_ACCOUNT) for token_full_e2e." >&2
  exit 1
fi

TS="$(date +%Y%m%d-%H%M%S)"
if [[ -z "${RESULTS_DIR}" ]]; then
  RESULTS_DIR="${ROOT_DIR}/target/sdk-validator-runs/${TS}"
fi
LOG_DIR="${RESULTS_DIR}/logs"
mkdir -p "${LOG_DIR}"

export FIVE_NETWORK="${NETWORK}"
export FIVE_PROGRAM_ID="${PROGRAM_ID}"
export VM_STATE_PDA="${VM_STATE}"
export FIVE_KEYPAIR_PATH="${KEYPAIR_PATH}"
export FIVE_TOKEN_SCRIPT_ACCOUNT="${TOKEN_SCRIPT_ACCOUNT}"
if [[ "${NETWORK}" == "devnet" ]]; then
  export FIVE_RPC_URL="https://api.devnet.solana.com"
else
  export FIVE_RPC_URL="http://127.0.0.1:8899"
fi

STATUS_FILE="${RESULTS_DIR}/status.jsonl"
: > "${STATUS_FILE}"
FAIL_COUNT=0

run_scenario() {
  local key="$1"
  local cwd="$2"
  shift 2
  local cmd="$*"
  local log="${LOG_DIR}/${key}.log"
  local rc
  echo "[RUN] ${key}: ${cmd}"
  set +e
  (
    cd "${cwd}"
    export FIVE_SCENARIO="${key}"
    bash -lc "${cmd}"
  ) > "${log}" 2>&1
  rc=$?
  set -e
  if [[ $rc -eq 0 ]]; then status="PASS"; else status="FAIL"; fi
  if [[ $rc -ne 0 ]]; then FAIL_COUNT=$((FAIL_COUNT + 1)); fi
  printf '{"scenario":"%s","status":"%s","exit_code":%d,"log":"%s","command":"%s"}\n' \
    "${key}" "${status}" "${rc}" "${log}" "$(printf '%s' "${cmd}" | sed 's/"/\\"/g')" >> "${STATUS_FILE}"
  echo "[DONE] ${key}: ${status} (rc=${rc})"
}

IFS=',' read -r -a SCENARIO_LIST <<< "${SCENARIOS}"
for raw in "${SCENARIO_LIST[@]}"; do
  s="$(echo "${raw}" | xargs)"
  case "${s}" in
    token_full_e2e)
      run_scenario "${s}" "${ROOT_DIR}/five-templates/token" "node e2e-token-test.mjs"
      ;;
    cpi_spl_mint)
      run_scenario "${s}" "${ROOT_DIR}/five-templates/cpi-examples" "node e2e-spl-token-mint-test.mjs"
      ;;
    cpi_pda_invoke)
      run_scenario "${s}" "${ROOT_DIR}/five-templates/cpi-examples" "node e2e-pda-invoke-test.mjs"
      ;;
    cpi_anchor_program)
      run_scenario "${s}" "${ROOT_DIR}/five-templates/cpi-examples" "node e2e-anchor-program-test.mjs"
      ;;
    cpi_integration)
      if [[ "${NETWORK}" == "devnet" ]]; then
        run_scenario "${s}" "${ROOT_DIR}/five-templates/cpi-integration-tests" "node test-devnet.mjs"
      else
        run_scenario "${s}" "${ROOT_DIR}/five-templates/cpi-integration-tests" "node test-localnet.mjs"
      fi
      ;;
    *)
      echo "Skipping unsupported scenario: ${s}" >&2
      printf '{"scenario":"%s","status":"SKIPPED","exit_code":0,"log":"","command":""}\n' "${s}" >> "${STATUS_FILE}"
      ;;
  esac
done

REPORT_JSON="${RESULTS_DIR}/sdk-validator-report.json"
node "${ROOT_DIR}/scripts/sdk-validator-report.mjs" \
  --status-file "${STATUS_FILE}" \
  --results-json "${REPORT_JSON}" \
  --network "${NETWORK}" \
  --rpc-url "${FIVE_RPC_URL}" \
  --program-id "${FIVE_PROGRAM_ID}" \
  --vm-state "${VM_STATE_PDA}" \
  --keypair "${FIVE_KEYPAIR_PATH}"

echo "Report written:"
echo "  - ${REPORT_JSON}"
echo "  - ${REPORT_JSON%.json}.md"

if [[ "${FAIL_COUNT}" -gt 0 ]]; then
  echo "SDK validator suites completed with failures: ${FAIL_COUNT}" >&2
  exit 1
fi
