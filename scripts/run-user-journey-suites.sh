#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
NETWORK="localnet"
PROGRAM_ID="${FIVE_PROGRAM_ID:-}"
VM_STATE="${VM_STATE_PDA:-}"
KEYPAIR_PATH="${FIVE_KEYPAIR_PATH:-${HOME}/.config/solana/id.json}"
TOKEN_SCRIPT_ACCOUNT="${FIVE_TOKEN_SCRIPT_ACCOUNT:-${TOKEN_SCRIPT_ACCOUNT:-}}"
AMM_SCRIPT_ACCOUNT="${FIVE_AMM_SCRIPT_ACCOUNT:-${AMM_SCRIPT_ACCOUNT:-}}"
LENDING_SCRIPT_ACCOUNT="${FIVE_LENDING_SCRIPT_ACCOUNT:-${LENDING_SCRIPT_ACCOUNT:-}}"
LENDING_ORACLE_SCRIPT_ACCOUNT="${FIVE_LENDING_ORACLE_SCRIPT_ACCOUNT:-${LENDING_ORACLE_SCRIPT_ACCOUNT:-}}"
SCENARIOS="${FIVE_SCENARIOS:-wallet_onboarding,token_lifecycle_two_users,failure_recovery,resume_existing_state,duplicate_submit_safety,amm_pool_onboarding,amm_two_user_swap_lifecycle,amm_failure_recovery,lending_market_onboarding,lending_borrow_repay_lifecycle,lending_failure_recovery}"
RESULTS_DIR=""

usage() {
  cat <<USAGE
Usage: $0 [--network localnet|devnet] [--program-id <pubkey>] [--vm-state <pubkey>] [--keypair <path>] [--token-script-account <pubkey>] [--amm-script-account <pubkey>] [--lending-script-account <pubkey>] [--lending-oracle-script-account <pubkey>] [--scenarios csv] [--results-dir path]
Provision localnet script accounts first with: node ./scripts/provision-user-journey-localnet.mjs [--shell]
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="${2:-}"; shift 2 ;;
    --program-id) PROGRAM_ID="${2:-}"; shift 2 ;;
    --vm-state) VM_STATE="${2:-}"; shift 2 ;;
    --keypair) KEYPAIR_PATH="${2:-}"; shift 2 ;;
    --token-script-account) TOKEN_SCRIPT_ACCOUNT="${2:-}"; shift 2 ;;
    --amm-script-account) AMM_SCRIPT_ACCOUNT="${2:-}"; shift 2 ;;
    --lending-script-account) LENDING_SCRIPT_ACCOUNT="${2:-}"; shift 2 ;;
    --lending-oracle-script-account) LENDING_ORACLE_SCRIPT_ACCOUNT="${2:-}"; shift 2 ;;
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

scenario_selected() {
  [[ ",${SCENARIOS}," == *",$1,"* ]]
}

if scenario_selected "wallet_onboarding" || \
   scenario_selected "token_lifecycle_two_users" || \
   scenario_selected "failure_recovery" || \
   scenario_selected "resume_existing_state" || \
   scenario_selected "duplicate_submit_safety"; then
  if [[ -z "${TOKEN_SCRIPT_ACCOUNT}" ]]; then
    echo "Missing --token-script-account (or FIVE_TOKEN_SCRIPT_ACCOUNT) for token-backed user-journey scenarios." >&2
    exit 1
  fi
fi

if scenario_selected "amm_pool_onboarding" || \
   scenario_selected "amm_two_user_swap_lifecycle" || \
   scenario_selected "amm_failure_recovery"; then
  if [[ -z "${AMM_SCRIPT_ACCOUNT}" ]]; then
    echo "Missing --amm-script-account (or FIVE_AMM_SCRIPT_ACCOUNT) for AMM user-journey scenarios." >&2
    exit 1
  fi
fi

if scenario_selected "lending_market_onboarding" || \
   scenario_selected "lending_borrow_repay_lifecycle" || \
   scenario_selected "lending_failure_recovery"; then
  if [[ -z "${LENDING_SCRIPT_ACCOUNT}" ]]; then
    echo "Missing --lending-script-account (or FIVE_LENDING_SCRIPT_ACCOUNT) for lending user-journey scenarios." >&2
    exit 1
  fi
  if [[ -z "${LENDING_ORACLE_SCRIPT_ACCOUNT}" ]]; then
    echo "Missing --lending-oracle-script-account (or FIVE_LENDING_ORACLE_SCRIPT_ACCOUNT) for lending user-journey scenarios." >&2
    exit 1
  fi
fi

TS="$(date +%Y%m%d-%H%M%S)"
if [[ -z "${RESULTS_DIR}" ]]; then
  RESULTS_DIR="${ROOT_DIR}/target/user-journey-runs/${TS}"
fi
LOG_DIR="${RESULTS_DIR}/logs"
mkdir -p "${LOG_DIR}"

export FIVE_NETWORK="${NETWORK}"
export FIVE_PROGRAM_ID="${PROGRAM_ID}"
export VM_STATE_PDA="${VM_STATE}"
export FIVE_KEYPAIR_PATH="${KEYPAIR_PATH}"
export FIVE_TOKEN_SCRIPT_ACCOUNT="${TOKEN_SCRIPT_ACCOUNT}"
export FIVE_AMM_SCRIPT_ACCOUNT="${AMM_SCRIPT_ACCOUNT}"
export FIVE_LENDING_SCRIPT_ACCOUNT="${LENDING_SCRIPT_ACCOUNT}"
export FIVE_LENDING_ORACLE_SCRIPT_ACCOUNT="${LENDING_ORACLE_SCRIPT_ACCOUNT}"
export FIVE_RESULTS_DIR="${RESULTS_DIR}"
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
  local script_name="$2"
  local blocking="$3"
  local family="$4"
  local log="${LOG_DIR}/${key}.log"
  local cmd="node ${script_name}"
  local rc
  echo "[RUN] ${key}: ${cmd}"
  set +e
  (
    cd "${ROOT_DIR}/five-templates/user-journeys"
    export FIVE_SCENARIO="${key}"
    export FIVE_SCENARIO_ARTIFACT_DIR="${RESULTS_DIR}/${key}"
    mkdir -p "${FIVE_SCENARIO_ARTIFACT_DIR}"
    bash -lc "${cmd}"
  ) > "${log}" 2>&1
  rc=$?
  set -e
  local status="PASS"
  if [[ $rc -ne 0 ]]; then
    status="FAIL"
    FAIL_COUNT=$((FAIL_COUNT + 1))
  fi
  printf '{"scenario":"%s","status":"%s","exit_code":%d,"log":"%s","command":"%s","blocking":%s,"family":"%s"}\n' \
    "${key}" "${status}" "${rc}" "${log}" "$(printf '%s' "${cmd}" | sed 's/"/\\"/g')" "${blocking}" "${family}" >> "${STATUS_FILE}"
  echo "[DONE] ${key}: ${status} (rc=${rc})"
}

IFS=',' read -r -a SCENARIO_LIST <<< "${SCENARIOS}"
for raw in "${SCENARIO_LIST[@]}"; do
  scenario="$(echo "${raw}" | xargs)"
  case "${scenario}" in
    wallet_onboarding)
      run_scenario "${scenario}" "wallet-onboarding.mjs" "true" "token"
      ;;
    token_lifecycle_two_users)
      run_scenario "${scenario}" "token-lifecycle-two-users.mjs" "true" "token"
      ;;
    failure_recovery)
      run_scenario "${scenario}" "failure-recovery.mjs" "true" "token"
      ;;
    resume_existing_state)
      run_scenario "${scenario}" "resume-existing-state.mjs" "true" "token"
      ;;
    duplicate_submit_safety)
      run_scenario "${scenario}" "duplicate-submit-safety.mjs" "true" "token"
      ;;
    amm_pool_onboarding)
      run_scenario "${scenario}" "amm-pool-onboarding.mjs" "true" "amm"
      ;;
    amm_two_user_swap_lifecycle)
      run_scenario "${scenario}" "amm-two-user-swap-lifecycle.mjs" "true" "amm"
      ;;
    amm_failure_recovery)
      run_scenario "${scenario}" "amm-failure-recovery.mjs" "true" "amm"
      ;;
    lending_market_onboarding)
      run_scenario "${scenario}" "lending-market-onboarding.mjs" "true" "lending"
      ;;
    lending_borrow_repay_lifecycle)
      run_scenario "${scenario}" "lending-borrow-repay-lifecycle.mjs" "true" "lending"
      ;;
    lending_failure_recovery)
      run_scenario "${scenario}" "lending-failure-recovery.mjs" "true" "lending"
      ;;
    *)
      echo "Skipping unsupported scenario: ${scenario}" >&2
      printf '{"scenario":"%s","status":"SKIPPED","exit_code":0,"log":"","command":"","blocking":false,"family":"unknown"}\n' "${scenario}" >> "${STATUS_FILE}"
      ;;
  esac
done

REPORT_JSON="${RESULTS_DIR}/user-journey-report.json"
node "${ROOT_DIR}/scripts/user-journey-report.mjs" \
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
  echo "User journey suites completed with failures: ${FAIL_COUNT}" >&2
  exit 1
fi
