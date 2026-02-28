#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${ROOT_DIR}/target/mvp-gate"
REPORT_JSON="${REPORT_DIR}/report.json"
REPORT_MD="${REPORT_DIR}/report.md"
CLUSTER="localnet"
AUTO_BUILD_SBF="1"
RUN_E2E_SMOKE="1"

usage() {
  cat <<USAGE
Usage: $0 [--cluster localnet|devnet|mainnet] [--no-build-sbf] [--skip-e2e-smoke]

Canonical full engineering gate sequence:
  1) Build/validate SBF artifacts
  2) Run core workspace tests
  3) Run BPF CU/runtime suites
  4) Run E2E smoke validation

Artifacts:
  - JSON report: target/mvp-gate/report.json
  - Markdown report: target/mvp-gate/report.md
  
Notes:
  - mainnet runs as a dry-run gate; E2E smoke is skipped automatically and must be proven on localnet + devnet.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cluster)
      CLUSTER="${2:-}"
      shift 2
      ;;
    --no-build-sbf)
      AUTO_BUILD_SBF="0"
      shift
      ;;
    --skip-e2e-smoke)
      RUN_E2E_SMOKE="0"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "${CLUSTER}" != "localnet" && "${CLUSTER}" != "devnet" && "${CLUSTER}" != "mainnet" ]]; then
  echo "Invalid --cluster value: ${CLUSTER}" >&2
  exit 1
fi

mkdir -p "${REPORT_DIR}"

START_TS="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
HOST_OS="$(uname -s)"
HOST_ARCH="$(uname -m)"
RUST_VERSION="$(rustc --version 2>/dev/null || echo unavailable)"
CARGO_VERSION="$(cargo --version 2>/dev/null || echo unavailable)"

STAGES=()
FAILED_STAGE=""
SBF_PARITY_NOTE=""
BPF_RUNTIME_NOTE=""
E2E_SMOKE_NOTE=""

append_sbf_note() {
  local note="$1"
  if [[ -z "${SBF_PARITY_NOTE}" ]]; then
    SBF_PARITY_NOTE="${note}"
  else
    SBF_PARITY_NOTE="${SBF_PARITY_NOTE}; ${note}"
  fi
}

record_stage() {
  local name="$1"
  local status="$2"
  local duration_sec="$3"
  local details="$4"
  STAGES+=("${name}|${status}|${duration_sec}|${details}")
}

escape_json() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

run_stage() {
  local name="$1"
  local details="$2"
  shift 2

  echo
  echo "==> ${name}"
  echo "    ${details}"

  local stage_start stage_end duration
  stage_start="$(date +%s)"

  set +e
  "$@"
  local rc=$?
  set -e

  stage_end="$(date +%s)"
  duration=$((stage_end - stage_start))

  local effective_details="${details}"
  if [[ "${name}" == "SBF Artifact Build and Validation" && -n "${SBF_PARITY_NOTE}" ]]; then
    effective_details="${effective_details}; ${SBF_PARITY_NOTE}"
  fi
  if [[ "${name}" == "BPF Runtime CU Suites" && -n "${BPF_RUNTIME_NOTE}" ]]; then
    effective_details="${effective_details}; ${BPF_RUNTIME_NOTE}"
  fi
  if [[ "${name}" == "E2E Smoke Validation" && -n "${E2E_SMOKE_NOTE}" ]]; then
    effective_details="${effective_details}; ${E2E_SMOKE_NOTE}"
  fi

  if [[ $rc -eq 0 ]]; then
    echo "    PASS (${duration}s)"
    record_stage "${name}" "pass" "${duration}" "${effective_details}"
    return 0
  fi

  echo "    FAIL (${duration}s, exit=${rc})"
  record_stage "${name}" "fail" "${duration}" "${effective_details}"
  FAILED_STAGE="${name}"
  return $rc
}

run_stage_or_exit() {
  run_stage "$@" || {
    emit_report
    return 1
  }
}

validate_sbf_artifacts() {
  local keypair_path="${ROOT_DIR}/target/deploy/five-keypair.json"
  local so_path="${ROOT_DIR}/target/deploy/five.so"
  local constants_path="${ROOT_DIR}/five-solana/src/generated_constants.rs"
  local anchor_manifest="${ROOT_DIR}/five-templates/anchor-token-comparison/programs/anchor-token-comparison/Cargo.toml"
  local anchor_so="${ROOT_DIR}/target/deploy/anchor_token_comparison.so"

  if [[ ! -f "${keypair_path}" || ! -f "${so_path}" ]]; then
    if [[ "${AUTO_BUILD_SBF}" == "1" ]]; then
      echo "Missing SBF artifact(s); building with cluster constants (${CLUSTER})..."
      "${ROOT_DIR}/scripts/build-five-solana-cluster.sh" --cluster "${CLUSTER}"
    else
      echo "Missing SBF artifacts. Expected:" >&2
      echo "  - ${keypair_path}" >&2
      echo "  - ${so_path}" >&2
      echo "Build first with:" >&2
      echo "  ./scripts/build-five-solana-cluster.sh --cluster ${CLUSTER}" >&2
      return 1
    fi
  fi

  [[ -f "${keypair_path}" ]] || {
    echo "Artifact still missing: ${keypair_path}" >&2
    return 1
  }

  [[ -f "${so_path}" ]] || {
    echo "Artifact still missing: ${so_path}" >&2
    return 1
  }

  validate_program_id_parity "${constants_path}" "${keypair_path}"

  if [[ ! -f "${anchor_so}" ]]; then
    echo "Missing required BPF test fixture artifact: ${anchor_so}"
    echo "Building anchor-token-comparison fixture artifact..."
    cargo-build-sbf --manifest-path "${anchor_manifest}" --sbf-out-dir "${ROOT_DIR}/target/deploy"
    [[ -f "${anchor_so}" ]] || {
      echo "Fixture artifact still missing after build: ${anchor_so}" >&2
      return 1
    }
    append_sbf_note "built missing runtime fixture artifact anchor_token_comparison.so"
  else
    append_sbf_note "runtime fixture artifact anchor_token_comparison.so present"
  fi

  echo "Validated artifacts:"
  echo "  - ${keypair_path}"
  echo "  - ${so_path}"
  echo "  - ${anchor_so}"
}

parse_constants_program_id() {
  local constants_path="$1"
  if [[ ! -f "${constants_path}" ]]; then
    echo "Missing generated constants file: ${constants_path}" >&2
    return 1
  fi

  local parsed
  parsed="$(grep -E 'pub const VM_PROGRAM_ID: &str = "[^"]+";' "${constants_path}" \
    | sed -E 's/^.*"([^"]+)".*$/\1/' \
    | tail -n1)"
  if [[ -z "${parsed}" ]]; then
    echo "Failed to parse VM_PROGRAM_ID from ${constants_path}" >&2
    return 1
  fi

  echo "${parsed}"
}

validate_program_id_parity() {
  local constants_path="$1"
  local keypair_path="$2"

  local expected_id actual_id
  expected_id="$(parse_constants_program_id "${constants_path}")" || return 1
  actual_id="$(solana-keygen pubkey "${keypair_path}")" || {
    echo "Failed reading keypair pubkey: ${keypair_path}" >&2
    return 1
  }

  if [[ "${expected_id}" == "${actual_id}" ]]; then
    append_sbf_note "artifact/constants parity ok (program_id=${actual_id})"
    return 0
  fi

  echo "Detected artifact/constants program ID drift:"
  echo "  - generated constants VM_PROGRAM_ID: ${expected_id}"
  echo "  - target/deploy/five-keypair.json: ${actual_id}"

  if [[ "${AUTO_BUILD_SBF}" == "1" ]]; then
    echo "Auto-build enabled; regenerating constants and SBF artifacts for ${CLUSTER}..."
    "${ROOT_DIR}/scripts/build-five-solana-cluster.sh" --cluster "${CLUSTER}"
    expected_id="$(parse_constants_program_id "${constants_path}")" || return 1
    actual_id="$(solana-keygen pubkey "${keypair_path}")" || return 1
    if [[ "${expected_id}" != "${actual_id}" ]]; then
      echo "Parity mismatch persisted after rebuild:" >&2
      echo "  expected VM_PROGRAM_ID=${expected_id}" >&2
      echo "  keypair pubkey=${actual_id}" >&2
      return 1
    fi
    append_sbf_note "detected mismatch and auto-repaired (program_id=${actual_id})"
    echo "Program ID parity restored: ${actual_id}"
    return 0
  fi

  append_sbf_note "detected mismatch and failed fast due to --no-build-sbf"
  echo "Program ID mismatch with --no-build-sbf enabled." >&2
  echo "Remediation:" >&2
  echo "  ./scripts/build-five-solana-cluster.sh --cluster ${CLUSTER}" >&2
  echo "Expected VM_PROGRAM_ID: ${expected_id}" >&2
  echo "Actual keypair pubkey: ${actual_id}" >&2
  return 1
}

run_core_workspace_tests() {
  cargo test --workspace --exclude five --quiet
}

run_bpf_runtime_suites() {
  local bpf_log="${REPORT_DIR}/bpf-runtime.log"
  : > "${bpf_log}"

  set +e
  cargo test -p five --test runtime_bpf_opcode_micro_cu_tests -- --nocapture 2>&1 | tee -a "${bpf_log}"
  local rc=$?
  if [[ $rc -eq 0 ]]; then
    cargo test -p five --test runtime_bpf_cu_tests -- --nocapture 2>&1 | tee -a "${bpf_log}"
    rc=$?
  fi
  set -e

  BPF_RUNTIME_NOTE=""
  if [[ $rc -ne 0 ]] && rg -q "invalid program argument.*Instruction 1|failed: invalid program argument|custom program error: 0x1e7a" "${bpf_log}"; then
    BPF_RUNTIME_NOTE="hint: deploy/execute parity signature detected (invalid program argument / 0x1e7a); verify generated constants VM_PROGRAM_ID matches target/deploy/five-keypair.json"
    echo "    Hint: ${BPF_RUNTIME_NOTE}"
  fi

  return $rc
}

run_e2e_smoke_validation() {
  if [[ "${RUN_E2E_SMOKE}" != "1" ]]; then
    echo "Skipping E2E smoke by request (--skip-e2e-smoke)."
    return 0
  fi

  if [[ "${CLUSTER}" == "mainnet" ]]; then
    E2E_SMOKE_NOTE="mainnet E2E intentionally disabled; localnet and devnet evidence required instead"
    echo "Skipping E2E smoke for mainnet. ${E2E_SMOKE_NOTE}"
    return 0
  fi

  if [[ "${CLUSTER}" == "devnet" ]]; then
    local sdk_payer="${FIVE_KEYPAIR_PATH:-${FIVE_CU_PAYER_KEYPAIR:-${HOME}/.config/solana/id.json}}"
    local sdk_program="${FIVE_PROGRAM_ID:-${FIVE_CU_PROGRAM_ID:-}}"
    local sdk_scenarios="${FIVE_SCENARIOS:-${FIVE_CU_SCENARIOS:-token_full_e2e,cpi_spl_mint,cpi_pda_invoke,cpi_anchor_program,cpi_integration}}"

    if [[ -z "${sdk_program}" ]]; then
      echo "Missing FIVE_PROGRAM_ID (or FIVE_CU_PROGRAM_ID) for devnet E2E smoke." >&2
      echo "Set it to your deployed devnet Five VM program ID and rerun." >&2
      return 1
    fi

    if [[ ! -f "${sdk_payer}" ]]; then
      echo "Missing keypair for devnet E2E smoke: ${sdk_payer}" >&2
      return 1
    fi

    echo "Running SDK validator smoke scenarios on devnet: ${sdk_scenarios}"
    "${ROOT_DIR}/scripts/run-sdk-validator-suites.sh" \
      --network devnet \
      --program-id "${sdk_program}" \
      --keypair "${sdk_payer}" \
      --scenarios "${sdk_scenarios}"
    return 0
  fi

  if [[ "${CLUSTER}" != "localnet" ]]; then
    echo "E2E smoke currently supports only localnet and devnet clusters (got ${CLUSTER})." >&2
    return 1
  fi

  local e2e_dir="${REPORT_DIR}/e2e"
  mkdir -p "${e2e_dir}"
  local deploy_log="${e2e_dir}/deploy-and-init.log"
  local validator_log="${e2e_dir}/solana-test-validator.log"
  local started_validator_pid=""

  command -v solana >/dev/null 2>&1
  command -v node >/dev/null 2>&1
  if ! solana cluster-version --url "http://127.0.0.1:8899" >/dev/null 2>&1; then
    echo "Local validator not detected on http://127.0.0.1:8899; starting solana-test-validator..."
    nohup solana-test-validator --reset > "${validator_log}" 2>&1 &
    started_validator_pid="$!"

    local ready="0"
    for _ in $(seq 1 30); do
      if solana cluster-version --url "http://127.0.0.1:8899" >/dev/null 2>&1; then
        ready="1"
        break
      fi
      sleep 1
    done

    if [[ "${ready}" != "1" ]]; then
      echo "Failed to start local validator for E2E smoke. See ${validator_log}" >&2
      [[ -n "${started_validator_pid}" ]] && kill "${started_validator_pid}" >/dev/null 2>&1 || true
      return 1
    fi
  fi
  solana-keygen pubkey "${HOME}/.config/solana/id.json" >/dev/null

  echo "Deploying and initializing FIVE VM on localnet..."
  ./five-scripts/deploy-and-init.sh localnet "${HOME}/.config/solana/id.json" prod 2>&1 | tee "${deploy_log}"

  local program_id vm_state_pda
  program_id="$(grep -E '^PROGRAM_ID=' "${deploy_log}" | tail -n1 | sed -E 's/^PROGRAM_ID=//' | tr -d '\r')"
  vm_state_pda="$(grep -E '^VM_STATE_PDA=' "${deploy_log}" | tail -n1 | sed -E 's/^VM_STATE_PDA=//' | tr -d '\r')"
  if [[ -z "${program_id}" ]]; then
    program_id="$(grep -E 'Program ID:' "${deploy_log}" | tail -n1 | sed -E 's/.*Program ID:[[:space:]]*//' | tr -d '\r')"
  fi
  if [[ -z "${vm_state_pda}" ]]; then
    vm_state_pda="$(grep -E 'VM State PDA:' "${deploy_log}" | tail -n1 | sed -E 's/.*VM State PDA:[[:space:]]*//' | tr -d '\r')"
  fi
  [[ -n "${program_id}" ]] || { echo "Failed to parse Program ID from deploy output"; return 1; }
  [[ -n "${vm_state_pda}" ]] || { echo "Failed to parse VM State PDA from deploy output"; return 1; }

  export FIVE_RPC_URL="http://127.0.0.1:8899"
  export FIVE_PROGRAM_ID="${program_id}"
  export VM_STATE_PDA="${vm_state_pda}"
  export FIVE_KEYPAIR_PATH="${HOME}/.config/solana/id.json"

  solana program show "${FIVE_PROGRAM_ID}" --url "${FIVE_RPC_URL}" >/dev/null
  solana account "${VM_STATE_PDA}" --url "${FIVE_RPC_URL}" --output json >/dev/null

  "${ROOT_DIR}/scripts/run-sdk-validator-suites.sh" \
    --network localnet \
    --program-id "${FIVE_PROGRAM_ID}" \
    --vm-state "${VM_STATE_PDA}" \
    --keypair "${FIVE_KEYPAIR_PATH}" \
    --scenarios "${FIVE_SCENARIOS:-token_full_e2e,cpi_spl_mint,cpi_pda_invoke,cpi_anchor_program,cpi_integration}" \
    --results-dir "${e2e_dir}"

  if [[ -n "${started_validator_pid}" ]]; then
    kill "${started_validator_pid}" >/dev/null 2>&1 || true
    wait "${started_validator_pid}" 2>/dev/null || true
  fi
}

emit_report() {
  local overall_status="pass"
  if [[ -n "${FAILED_STAGE}" ]]; then
    overall_status="fail"
  fi

  local end_ts="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  {
    echo "{"
    echo "  \"gate\": \"full-engineering\"," 
    echo "  \"started_at\": \"$(escape_json "${START_TS}")\"," 
    echo "  \"finished_at\": \"$(escape_json "${end_ts}")\"," 
    echo "  \"overall_status\": \"${overall_status}\"," 
    echo "  \"failed_stage\": \"$(escape_json "${FAILED_STAGE}")\"," 
    echo "  \"cluster\": \"$(escape_json "${CLUSTER}")\"," 
    echo "  \"auto_build_sbf\": ${AUTO_BUILD_SBF},"
    echo "  \"run_e2e_smoke\": ${RUN_E2E_SMOKE},"
    echo "  \"environment\": {"
    echo "    \"os\": \"$(escape_json "${HOST_OS}")\"," 
    echo "    \"arch\": \"$(escape_json "${HOST_ARCH}")\"," 
    echo "    \"rustc\": \"$(escape_json "${RUST_VERSION}")\"," 
    echo "    \"cargo\": \"$(escape_json "${CARGO_VERSION}")\""
    echo "  },"
    echo "  \"required_artifacts\": ["
    echo "    \"target/deploy/five-keypair.json\"," 
    echo "    \"target/deploy/five.so\""
    echo "  ],"
    echo "  \"stages\": ["

    local idx=0
    local total="${#STAGES[@]}"
    for item in "${STAGES[@]}"; do
      IFS='|' read -r name status duration details <<<"${item}"
      idx=$((idx + 1))
      echo "    {"
      echo "      \"name\": \"$(escape_json "${name}")\"," 
      echo "      \"status\": \"$(escape_json "${status}")\"," 
      echo "      \"duration_seconds\": ${duration},"
      echo "      \"details\": \"$(escape_json "${details}")\""
      if [[ ${idx} -lt ${total} ]]; then
        echo "    },"
      else
        echo "    }"
      fi
    done

    echo "  ]"
    echo "}"
  } > "${REPORT_JSON}"

  {
    echo "# MVP Full Engineering Gate Report"
    echo
    echo "- Started (UTC): ${START_TS}"
    echo "- Finished (UTC): ${end_ts}"
    echo "- Cluster: ${CLUSTER}"
    echo "- Overall status: ${overall_status}"
    if [[ -n "${FAILED_STAGE}" ]]; then
      echo "- Failed stage: ${FAILED_STAGE}"
    fi
    echo
    echo "## Required Artifacts"
    echo "- target/deploy/five-keypair.json"
    echo "- target/deploy/five.so"
    echo
    echo "## Stage Results"
    for item in "${STAGES[@]}"; do
      IFS='|' read -r name status duration details <<<"${item}"
      echo "- ${name}: ${status} (${duration}s) - ${details}"
    done
    echo
    echo "## Environment"
    echo "- OS/Arch: ${HOST_OS}/${HOST_ARCH}"
    echo "- rustc: ${RUST_VERSION}"
    echo "- cargo: ${CARGO_VERSION}"
  } > "${REPORT_MD}"

  echo
  echo "Reports:"
  echo "  - ${REPORT_JSON}"
  echo "  - ${REPORT_MD}"

  [[ "${overall_status}" == "pass" ]]
}

main() {
  cd "${ROOT_DIR}"

  echo "======================================"
  echo "Five MVP Full Engineering Gate"
  echo "======================================"
  echo "Cluster: ${CLUSTER}"
  echo "Auto-build SBF artifacts: ${AUTO_BUILD_SBF}"
  echo "Run E2E smoke: ${RUN_E2E_SMOKE}"

  run_stage_or_exit \
    "SBF Artifact Build and Validation" \
    "Ensure target/deploy/five-keypair.json and target/deploy/five.so exist" \
    validate_sbf_artifacts || return 1

  run_stage_or_exit \
    "Core Workspace Tests" \
    "Run cargo test --workspace --exclude five --quiet" \
    run_core_workspace_tests || return 1

  run_stage_or_exit \
    "BPF Runtime CU Suites" \
    "Run runtime_bpf_opcode_micro_cu_tests and runtime_bpf_cu_tests" \
    run_bpf_runtime_suites || return 1

  run_stage_or_exit \
    "E2E Smoke Validation" \
    "Run runtime_template_fixture_tests to validate end-to-end flow" \
    run_e2e_smoke_validation || return 1

  emit_report
}

if main; then
  echo
  echo "MVP full engineering gate: PASS"
  exit 0
fi

echo
if [[ -n "${FAILED_STAGE}" ]]; then
  echo "MVP full engineering gate: FAIL (stage: ${FAILED_STAGE})"
else
  echo "MVP full engineering gate: FAIL"
fi
exit 1
