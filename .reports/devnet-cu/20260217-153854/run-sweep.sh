#!/usr/bin/env bash
set -u
BASE="$1"
LOGS="$BASE/logs"
STATUS="$BASE/status.jsonl"
export FIVE_CU_NETWORK=devnet
export FIVE_CU_DEVNET_OPT_IN=1
export FIVE_CU_PAYER_KEYPAIR=/Users/ivmidable/.config/solana/id.json
export FIVE_CU_PROGRAM_ID=3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1
export FIVE_CU_SCENARIOS=token_full_e2e,external_non_cpi,external_interface_mapping_non_cpi,external_burst_non_cpi,memory_string_heavy,arithmetic_intensive
export FIVE_CU_RESULTS_FILE="$BASE/validator-cu-report.json"
run_cmd () {
  local key="$1"; shift
  local cwd="$1"; shift
  local cmd="$*"
  local log="$LOGS/${key}.log"
  local start end rc dur
  start=$(date +%s)
  echo "[RUN] $key :: $cmd"
  (
    cd "$cwd"
    bash -lc "$cmd"
  ) >"$log" 2>&1
  rc=$?
  end=$(date +%s)
  dur=$((end-start))
  if [ $rc -eq 0 ]; then st="PASS"; else st="FAIL"; fi
  printf '{"key":"%s","status":"%s","exit_code":%d,"duration_sec":%d,"cwd":"%s","command":%s,"log":%s}\n' \
    "$key" "$st" "$rc" "$dur" "$cwd" \
    "$(printf '%s' "$cmd" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')" \
    "$(printf '%s' "$log" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read()))')" \
    >> "$STATUS"
  echo "[DONE] $key :: $st (rc=$rc, ${dur}s)"
}

# Validator harness
run_cmd validator_cu_orchestrator /Users/ivmidable/Development/five-mono \
  "cargo test -p five --test runtime_validator_cu_tests validator_cu_orchestrator -- --nocapture"

# CPI templates
run_cmd cpi_examples_spl_token /Users/ivmidable/Development/five-mono/five-templates/cpi-examples "npm run test:spl-token-mint"
run_cmd cpi_examples_pda_invoke /Users/ivmidable/Development/five-mono/five-templates/cpi-examples "npm run test:pda-invoke"
run_cmd cpi_examples_anchor_program /Users/ivmidable/Development/five-mono/five-templates/cpi-examples "npm run test:anchor-program"
run_cmd cpi_integration_devnet /Users/ivmidable/Development/five-mono/five-templates/cpi-integration-tests "npm run test:devnet"

# 5ive-* projects
run_cmd p_5ive_token_test /Users/ivmidable/Development/five-mono/5ive-token "npm test"
run_cmd p_5ive_token_client_run /Users/ivmidable/Development/five-mono/5ive-token "npm run client:run"
run_cmd p_5ive_token_client_token /Users/ivmidable/Development/five-mono/5ive-token "npm run client:token"

run_cmd p_5ive_token2_test /Users/ivmidable/Development/five-mono/5ive-token-2 "npm test"
run_cmd p_5ive_token2_client_run /Users/ivmidable/Development/five-mono/5ive-token-2 "npm run client:run"

run_cmd p_5ive_lending_test /Users/ivmidable/Development/five-mono/5ive-lending "npm test"
run_cmd p_5ive_lending2_test /Users/ivmidable/Development/five-mono/5ive-lending-2 "npm test"
run_cmd p_5ive_lending3_test /Users/ivmidable/Development/five-mono/5ive-lending-3 "npm test"
run_cmd p_5ive_lending4_test /Users/ivmidable/Development/five-mono/5ive-lending-4 "npm test"

run_cmd p_5ive_amm_test /Users/ivmidable/Development/five-mono/5ive-amm "npm test"

run_cmd p_5ive_cfd_test /Users/ivmidable/Development/five-mono/5ive-cfd "npm test"
run_cmd p_5ive_cfd_client_run /Users/ivmidable/Development/five-mono/5ive-cfd "npm run client:run"

run_cmd p_5ive_esccrow_test /Users/ivmidable/Development/five-mono/5ive-esccrow "npm test"
run_cmd p_5ive_esccrow_client_run /Users/ivmidable/Development/five-mono/5ive-esccrow "npm run client:run"

# Optional cross-check
run_cmd optional_mvp_release_gate /Users/ivmidable/Development/five-mono "bash scripts/mvp-release-gate.sh --cluster devnet --skip-e2e-smoke"
