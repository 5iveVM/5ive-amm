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

  if [[ $rc -eq 0 ]]; then
    echo "    PASS (${duration}s)"
    record_stage "${name}" "pass" "${duration}" "${details}"
    return 0
  fi

  echo "    FAIL (${duration}s, exit=${rc})"
  record_stage "${name}" "fail" "${duration}" "${details}"
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

  echo "Validated artifacts:"
  echo "  - ${keypair_path}"
  echo "  - ${so_path}"
}

run_core_workspace_tests() {
  cargo test --workspace --exclude five --quiet
}

run_bpf_runtime_suites() {
  cargo test -p five --test runtime_bpf_opcode_micro_cu_tests -- --nocapture
  cargo test -p five --test runtime_bpf_cu_tests -- --nocapture
}

run_e2e_smoke_validation() {
  if [[ "${RUN_E2E_SMOKE}" != "1" ]]; then
    echo "Skipping E2E smoke by request (--skip-e2e-smoke)."
    return 0
  fi

  cargo test -p five --test runtime_template_fixture_tests -- --nocapture
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
