#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

log() {
  printf '[require-batch-bench] %s\n' "$*"
}

parse_require_counts() {
  local disasm_path="$1"
  local out_path="$2"
  local instruction_count
  local require_dispatch_count
  local require_batch_count

  instruction_count="$(grep -Ec '^[[:space:]]+[0-9a-f]{4}:' "${disasm_path}" || true)"
  require_dispatch_count="$(grep -Ec '^[[:space:]]+[0-9a-f]{4}: .*REQUIRE' "${disasm_path}" || true)"
  require_batch_count="$(grep -Ec '^[[:space:]]+[0-9a-f]{4}: .*REQUIRE_BATCH' "${disasm_path}" || true)"

  jq -n \
    --argjson instruction_count "${instruction_count}" \
    --argjson require_dispatch_count "${require_dispatch_count}" \
    --argjson require_batch_count "${require_batch_count}" \
    '{
      instruction_count: $instruction_count,
      require_dispatch_count: $require_dispatch_count,
      require_batch_count: $require_batch_count
    }' > "${out_path}"
}

collect_static_metrics() {
  local profile="$1"
  local project="$2"
  local artifact_path="${ARTIFACT_DIR}/${project}.${profile}.five"
  local raw_bytecode="${TMP_DIR}/${project}.${profile}.bin"
  local disasm_log="${LOG_DIR}/${project}.${profile}.disasm.log"
  local metrics_json="${METRICS_DIR}/${project}.${profile}.static.json"

  jq -r '.bytecode' "${artifact_path}" | base64 --decode > "${raw_bytecode}"
  cargo run -q -p five-dsl-compiler --bin five -- inspect "${raw_bytecode}" --disasm > "${disasm_log}"
  parse_require_counts "${disasm_log}" "${metrics_json}"
}

parse_pipe_step_cu() {
  local log_file="$1"
  local step_name="$2"
  local line
  line="$(grep -F "${step_name}" "${log_file}" | grep -m1 -E '\| ok=' || true)"
  if [[ -z "${line}" ]]; then
    return 0
  fi
  printf '%s\n' "${line}" | sed -nE 's/.*cu=([0-9]+).*/\1/p' | head -n1
}

parse_single_pool_step_cu() {
  local log_file="$1"
  local step_name="$2"
  local line
  line="$(grep -F "${step_name}" "${log_file}" | grep -m1 -E 'cu=[0-9]+' || true)"
  if [[ -z "${line}" ]]; then
    return 0
  fi
  printf '%s\n' "${line}" | sed -nE 's/.*cu=([0-9]+).*/\1/p' | head -n1
}

run_amm_profile() {
  local profile="$1"
  local build_target="${ROOT_DIR}/5ive-amm/build/main.five"
  local log_file="${LOG_DIR}/5ive-amm.${profile}.runtime.log"
  local out_json="${METRICS_DIR}/5ive-amm.${profile}.runtime.json"

  cp "${ARTIFACT_DIR}/5ive-amm.${profile}.five" "${build_target}"
  (
    cd "${ROOT_DIR}/5ive-amm/client"
    FIVE_RPC_URL="${RPC_URL}" \
    FIVE_VM_PROGRAM_ID="${PROGRAM_ID}" \
    FIVE_NETWORK="localnet" \
    node dist/main.js
  ) > "${log_file}" 2>&1

  local init_cu bootstrap_cu add_cu swap_cu remove_cu total_cu
  init_cu="$(parse_pipe_step_cu "${log_file}" "init_pool")"
  bootstrap_cu="$(parse_pipe_step_cu "${log_file}" "bootstrap_liquidity")"
  add_cu="$(parse_pipe_step_cu "${log_file}" "add_liquidity")"
  swap_cu="$(parse_pipe_step_cu "${log_file}" "swap:a_to_b")"
  remove_cu="$(parse_pipe_step_cu "${log_file}" "remove_liquidity")"

  for value in "${init_cu}" "${bootstrap_cu}" "${add_cu}" "${swap_cu}" "${remove_cu}"; do
    if [[ -z "${value}" ]]; then
      echo "failed parsing 5ive-amm CU values from ${log_file}" >&2
      tail -n 120 "${log_file}" >&2 || true
      exit 1
    fi
  done

  total_cu=$((init_cu + bootstrap_cu + add_cu + swap_cu + remove_cu))

  jq -n \
    --arg log_file "${log_file}" \
    --argjson init_pool "${init_cu}" \
    --argjson bootstrap_liquidity "${bootstrap_cu}" \
    --argjson add_liquidity "${add_cu}" \
    --argjson swap_a_to_b "${swap_cu}" \
    --argjson remove_liquidity "${remove_cu}" \
    --argjson total_cu "${total_cu}" \
    '{
      log_file: $log_file,
      steps: {
        init_pool: $init_pool,
        bootstrap_liquidity: $bootstrap_liquidity,
        add_liquidity: $add_liquidity,
        swap_a_to_b: $swap_a_to_b,
        remove_liquidity: $remove_liquidity
      },
      total_cu: $total_cu
    }' > "${out_json}"
}

run_single_pool_profile() {
  local profile="$1"
  local build_target="${ROOT_DIR}/5ive-single-pool/build/main.five"
  local log_file="${LOG_DIR}/5ive-single-pool.${profile}.runtime.log"
  local out_json="${METRICS_DIR}/5ive-single-pool.${profile}.runtime.json"

  cp "${ARTIFACT_DIR}/5ive-single-pool.${profile}.five" "${build_target}"
  (
    cd "${ROOT_DIR}/5ive-single-pool/client"
    FIVE_RPC_URL="${RPC_URL}" \
    FIVE_VM_PROGRAM_ID="${PROGRAM_ID}" \
    FIVE_NETWORK="localnet" \
    node run-localnet-lst-flow-fixed-local-sdk.mjs
  ) > "${log_file}" 2>&1

  local init_cu deposit_cu withdraw_cu total_cu
  init_cu="$(parse_single_pool_step_cu "${log_file}" "initialize_pool")"
  deposit_cu="$(parse_single_pool_step_cu "${log_file}" "deposit_stake")"
  withdraw_cu="$(parse_single_pool_step_cu "${log_file}" "withdraw_stake")"

  for value in "${init_cu}" "${deposit_cu}" "${withdraw_cu}"; do
    if [[ -z "${value}" ]]; then
      echo "failed parsing 5ive-single-pool CU values from ${log_file}" >&2
      tail -n 120 "${log_file}" >&2 || true
      exit 1
    fi
  done

  total_cu=$((init_cu + deposit_cu + withdraw_cu))

  jq -n \
    --arg log_file "${log_file}" \
    --argjson initialize_pool "${init_cu}" \
    --argjson deposit_stake "${deposit_cu}" \
    --argjson withdraw_stake "${withdraw_cu}" \
    --argjson total_cu "${total_cu}" \
    '{
      log_file: $log_file,
      steps: {
        initialize_pool: $initialize_pool,
        deposit_stake: $deposit_stake,
        withdraw_stake: $withdraw_stake
      },
      total_cu: $total_cu
    }' > "${out_json}"
}

build_profile_artifacts() {
  local profile="$1"
  local disable_batch="$2"

  log "building ${profile} artifacts"
  if [[ "${disable_batch}" == "1" ]]; then
    FIVE_DISABLE_REQUIRE_BATCH=1 node five-cli/dist/index.js build --project 5ive-amm > "${LOG_DIR}/5ive-amm.${profile}.build.log" 2>&1
    FIVE_DISABLE_REQUIRE_BATCH=1 node five-cli/dist/index.js build --project 5ive-single-pool > "${LOG_DIR}/5ive-single-pool.${profile}.build.log" 2>&1
  else
    node five-cli/dist/index.js build --project 5ive-amm > "${LOG_DIR}/5ive-amm.${profile}.build.log" 2>&1
    node five-cli/dist/index.js build --project 5ive-single-pool > "${LOG_DIR}/5ive-single-pool.${profile}.build.log" 2>&1
  fi

  cp "${ROOT_DIR}/5ive-amm/build/main.five" "${ARTIFACT_DIR}/5ive-amm.${profile}.five"
  cp "${ROOT_DIR}/5ive-single-pool/build/main.five" "${ARTIFACT_DIR}/5ive-single-pool.${profile}.five"
}

wait_for_validator() {
  local retries=120
  for _ in $(seq 1 "${retries}"); do
    if solana -u "${RPC_URL}" cluster-version >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  return 1
}

start_validator() {
  log "starting local validator on ${RPC_URL}"
  rm -rf "${LEDGER_DIR}"
  mkdir -p "${LEDGER_DIR}"
  solana-test-validator \
    --reset \
    --ledger "${LEDGER_DIR}" \
    --rpc-port "${RPC_PORT}" \
    --faucet-port "${FAUCET_PORT}" \
    --bpf-program "${PROGRAM_ID}" "${ROOT_DIR}/target/deploy/five.so" \
    > "${LOG_DIR}/validator.log" 2>&1 &
  VALIDATOR_PID="$!"

  if ! wait_for_validator; then
    echo "validator failed to become healthy at ${RPC_URL}" >&2
    tail -n 200 "${LOG_DIR}/validator.log" >&2 || true
    exit 1
  fi
}

bootstrap_vm_state() {
  local payer_pubkey
  payer_pubkey="$(solana-keygen pubkey "${PAYER_KEYPAIR}")"
  solana -u "${RPC_URL}" airdrop 100 "${payer_pubkey}" >/dev/null 2>&1 || true
  node scripts/init-localnet-vm-state.mjs \
    --network localnet \
    --rpc-url "${RPC_URL}" \
    --program-id "${PROGRAM_ID}" \
    > "${LOG_DIR}/init-vm-state.log" 2>&1
  node scripts/init-devnet-fee-vaults.mjs \
    --network localnet \
    --rpc-url "${RPC_URL}" \
    --program-id "${PROGRAM_ID}" \
    > "${LOG_DIR}/init-fee-vaults.log" 2>&1
}

write_reports() {
  local json_report="${REPORT_DIR}/report.json"
  local md_report="${REPORT_DIR}/report.md"
  local generated_at
  generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  jq -n \
    --arg generated_at "${generated_at}" \
    --arg rpc_url "${RPC_URL}" \
    --arg program_id "${PROGRAM_ID}" \
    --arg validator_log "${LOG_DIR}/validator.log" \
    --slurpfile amm_base_static "${METRICS_DIR}/5ive-amm.baseline.static.json" \
    --slurpfile amm_head_static "${METRICS_DIR}/5ive-amm.head.static.json" \
    --slurpfile sp_base_static "${METRICS_DIR}/5ive-single-pool.baseline.static.json" \
    --slurpfile sp_head_static "${METRICS_DIR}/5ive-single-pool.head.static.json" \
    --slurpfile amm_base_runtime "${METRICS_DIR}/5ive-amm.baseline.runtime.json" \
    --slurpfile amm_head_runtime "${METRICS_DIR}/5ive-amm.head.runtime.json" \
    --slurpfile sp_base_runtime "${METRICS_DIR}/5ive-single-pool.baseline.runtime.json" \
    --slurpfile sp_head_runtime "${METRICS_DIR}/5ive-single-pool.head.runtime.json" \
    '
    def delta(base; head):
      {
        baseline: base,
        head: head,
        delta: (head - base),
        delta_percent: (if base == 0 then null else (((head - base) / base) * 100) end)
      };
    {
      generated_at: $generated_at,
      rpc_url: $rpc_url,
      program_id: $program_id,
      validator_log: $validator_log,
      static: {
        "5ive-amm": {
          instruction_count: delta($amm_base_static[0].instruction_count; $amm_head_static[0].instruction_count),
          require_dispatch_count: delta($amm_base_static[0].require_dispatch_count; $amm_head_static[0].require_dispatch_count),
          require_batch_count: delta($amm_base_static[0].require_batch_count; $amm_head_static[0].require_batch_count)
        },
        "5ive-single-pool": {
          instruction_count: delta($sp_base_static[0].instruction_count; $sp_head_static[0].instruction_count),
          require_dispatch_count: delta($sp_base_static[0].require_dispatch_count; $sp_head_static[0].require_dispatch_count),
          require_batch_count: delta($sp_base_static[0].require_batch_count; $sp_head_static[0].require_batch_count)
        }
      },
      runtime: {
        "5ive-amm": {
          steps: {
            init_pool: delta($amm_base_runtime[0].steps.init_pool; $amm_head_runtime[0].steps.init_pool),
            bootstrap_liquidity: delta($amm_base_runtime[0].steps.bootstrap_liquidity; $amm_head_runtime[0].steps.bootstrap_liquidity),
            add_liquidity: delta($amm_base_runtime[0].steps.add_liquidity; $amm_head_runtime[0].steps.add_liquidity),
            swap_a_to_b: delta($amm_base_runtime[0].steps.swap_a_to_b; $amm_head_runtime[0].steps.swap_a_to_b),
            remove_liquidity: delta($amm_base_runtime[0].steps.remove_liquidity; $amm_head_runtime[0].steps.remove_liquidity)
          },
          total_cu: delta($amm_base_runtime[0].total_cu; $amm_head_runtime[0].total_cu),
          logs: {
            baseline: $amm_base_runtime[0].log_file,
            head: $amm_head_runtime[0].log_file
          }
        },
        "5ive-single-pool": {
          steps: {
            initialize_pool: delta($sp_base_runtime[0].steps.initialize_pool; $sp_head_runtime[0].steps.initialize_pool),
            deposit_stake: delta($sp_base_runtime[0].steps.deposit_stake; $sp_head_runtime[0].steps.deposit_stake),
            withdraw_stake: delta($sp_base_runtime[0].steps.withdraw_stake; $sp_head_runtime[0].steps.withdraw_stake)
          },
          total_cu: delta($sp_base_runtime[0].total_cu; $sp_head_runtime[0].total_cu),
          logs: {
            baseline: $sp_base_runtime[0].log_file,
            head: $sp_head_runtime[0].log_file
          }
        }
      },
      acceptance: {
        "5ive-amm": {
          require_dispatch_reduction_pct: (
            if $amm_base_static[0].require_dispatch_count == 0 then null
            else ((($amm_base_static[0].require_dispatch_count - $amm_head_static[0].require_dispatch_count) / $amm_base_static[0].require_dispatch_count) * 100)
            end
          ),
          instruction_drop_pct: (
            if $amm_base_static[0].instruction_count == 0 then null
            else ((($amm_base_static[0].instruction_count - $amm_head_static[0].instruction_count) / $amm_base_static[0].instruction_count) * 100)
            end
          ),
          dispatch_target_met: (
            $amm_base_static[0].require_dispatch_count > 0 and
            ((($amm_base_static[0].require_dispatch_count - $amm_head_static[0].require_dispatch_count) / $amm_base_static[0].require_dispatch_count) * 100) >= 50
          ),
          instruction_target_met: (
            $amm_base_static[0].instruction_count > 0 and
            ((($amm_base_static[0].instruction_count - $amm_head_static[0].instruction_count) / $amm_base_static[0].instruction_count) * 100) >= 8
          )
        },
        "5ive-single-pool": {
          require_dispatch_reduction_pct: (
            if $sp_base_static[0].require_dispatch_count == 0 then null
            else ((($sp_base_static[0].require_dispatch_count - $sp_head_static[0].require_dispatch_count) / $sp_base_static[0].require_dispatch_count) * 100)
            end
          ),
          instruction_drop_pct: (
            if $sp_base_static[0].instruction_count == 0 then null
            else ((($sp_base_static[0].instruction_count - $sp_head_static[0].instruction_count) / $sp_base_static[0].instruction_count) * 100)
            end
          ),
          dispatch_target_met: (
            $sp_base_static[0].require_dispatch_count > 0 and
            ((($sp_base_static[0].require_dispatch_count - $sp_head_static[0].require_dispatch_count) / $sp_base_static[0].require_dispatch_count) * 100) >= 50
          ),
          instruction_target_met: (
            $sp_base_static[0].instruction_count > 0 and
            ((($sp_base_static[0].instruction_count - $sp_head_static[0].instruction_count) / $sp_base_static[0].instruction_count) * 100) >= 8
          )
        }
      }
    }' > "${json_report}"

  {
    echo "# REQUIRE_BATCH Localnet Benchmark"
    echo
    echo "- Generated: ${generated_at}"
    echo "- RPC: ${RPC_URL}"
    echo "- Program ID: ${PROGRAM_ID}"
    echo "- Validator log: ${LOG_DIR}/validator.log"
    echo
    echo "## Static Bytecode Metrics"
    echo
    echo "| Contract | Metric | Baseline | Head | Delta | Delta % |"
    echo "|---|---|---:|---:|---:|---:|"
    jq -r '
      .static
      | to_entries[]
      | .key as $contract
      | .value
      | [
          [$contract, "instruction_count", .instruction_count.baseline, .instruction_count.head, .instruction_count.delta, .instruction_count.delta_percent],
          [$contract, "require_dispatch_count", .require_dispatch_count.baseline, .require_dispatch_count.head, .require_dispatch_count.delta, .require_dispatch_count.delta_percent],
          [$contract, "require_batch_count", .require_batch_count.baseline, .require_batch_count.head, .require_batch_count.delta, .require_batch_count.delta_percent]
        ]
      | .[]
      | @tsv
    ' "${json_report}" | while IFS=$'\t' read -r contract metric baseline head delta pct; do
      printf '| %s | %s | %s | %s | %s | %.2f%% |\n' "${contract}" "${metric}" "${baseline}" "${head}" "${delta}" "${pct}"
    done
    echo
    echo "## Runtime CU Totals"
    echo
    echo "| Contract | Baseline CU | Head CU | Delta | Delta % |"
    echo "|---|---:|---:|---:|---:|"
    jq -r '
      .runtime
      | to_entries[]
      | [.key, .value.total_cu.baseline, .value.total_cu.head, .value.total_cu.delta, .value.total_cu.delta_percent]
      | @tsv
    ' "${json_report}" | while IFS=$'\t' read -r contract baseline head delta pct; do
      printf '| %s | %s | %s | %s | %.2f%% |\n' "${contract}" "${baseline}" "${head}" "${delta}" "${pct}"
    done
    echo
    echo "## Acceptance Checks"
    echo
    echo "| Contract | Dispatch Reduction % | Instruction Drop % | Dispatch >= 50% | Instruction >= 8% |"
    echo "|---|---:|---:|---|---|"
    jq -r '
      .acceptance
      | to_entries[]
      | [.key, .value.require_dispatch_reduction_pct, .value.instruction_drop_pct, .value.dispatch_target_met, .value.instruction_target_met]
      | @tsv
    ' "${json_report}" | while IFS=$'\t' read -r contract dispatch_pct instruction_pct dispatch_ok instruction_ok; do
      printf '| %s | %.2f%% | %.2f%% | %s | %s |\n' "${contract}" "${dispatch_pct}" "${instruction_pct}" "${dispatch_ok}" "${instruction_ok}"
    done
  } > "${md_report}"

  log "wrote ${json_report}"
  log "wrote ${md_report}"
}

cleanup() {
  if [[ -n "${VALIDATOR_PID:-}" ]]; then
    kill "${VALIDATOR_PID}" >/dev/null 2>&1 || true
    wait "${VALIDATOR_PID}" 2>/dev/null || true
  fi
}

require_cmd jq
require_cmd node
require_cmd cargo
require_cmd solana
require_cmd solana-test-validator
require_cmd solana-keygen
require_cmd base64

TS="$(date -u +%Y-%m-%dT%H-%M-%SZ)"
REPORT_DIR="${REPORT_DIR:-${ROOT_DIR}/target/require-batch-localnet/${TS}}"
LOG_DIR="${REPORT_DIR}/logs"
ARTIFACT_DIR="${REPORT_DIR}/artifacts"
METRICS_DIR="${REPORT_DIR}/metrics"
TMP_DIR="${REPORT_DIR}/tmp"
mkdir -p "${LOG_DIR}" "${ARTIFACT_DIR}" "${METRICS_DIR}" "${TMP_DIR}"

RPC_PORT="${FIVE_BENCH_RPC_PORT:-8897}"
FAUCET_PORT="${FIVE_BENCH_FAUCET_PORT:-9907}"
RPC_URL="${FIVE_BENCH_RPC_URL:-http://127.0.0.1:${RPC_PORT}}"
LEDGER_DIR="${FIVE_BENCH_LEDGER_DIR:-${ROOT_DIR}/.localnet-require-batch-ledger}"
PAYER_KEYPAIR="${FIVE_KEYPAIR_PATH:-${HOME}/.config/solana/id.json}"
USE_EXISTING_VALIDATOR="${FIVE_BENCH_USE_EXISTING_VALIDATOR:-0}"
PROGRAM_ID=""
VALIDATOR_PID=""

trap cleanup EXIT

log "building localnet SBF VM artifact"
./scripts/build-five-solana-cluster.sh --cluster localnet > "${LOG_DIR}/build-five-solana-cluster.log" 2>&1
PROGRAM_ID="$(solana-keygen pubkey "${ROOT_DIR}/target/deploy/five-keypair.json")"

build_profile_artifacts baseline 1
build_profile_artifacts head 0

collect_static_metrics baseline 5ive-amm
collect_static_metrics head 5ive-amm
collect_static_metrics baseline 5ive-single-pool
collect_static_metrics head 5ive-single-pool

if [[ "${USE_EXISTING_VALIDATOR}" == "1" ]]; then
  log "using existing validator at ${RPC_URL}"
  if ! solana -u "${RPC_URL}" cluster-version >/dev/null 2>&1; then
    echo "validator is not healthy at ${RPC_URL}" >&2
    exit 1
  fi
  log "deploying local VM program into existing validator"
  solana -u "${RPC_URL}" program deploy \
    --program-id "${ROOT_DIR}/target/deploy/five-keypair.json" \
    "${ROOT_DIR}/target/deploy/five.so" \
    > "${LOG_DIR}/program-deploy.log" 2>&1
else
  start_validator
fi
bootstrap_vm_state

run_amm_profile baseline
run_amm_profile head
run_single_pool_profile baseline
run_single_pool_profile head

# Leave working tree artifacts on the optimized head profile.
cp "${ARTIFACT_DIR}/5ive-amm.head.five" "${ROOT_DIR}/5ive-amm/build/main.five"
cp "${ARTIFACT_DIR}/5ive-single-pool.head.five" "${ROOT_DIR}/5ive-single-pool/build/main.five"

write_reports

log "benchmark complete"
