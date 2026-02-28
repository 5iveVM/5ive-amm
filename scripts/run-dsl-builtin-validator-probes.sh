#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

NETWORK="localnet"
RESULTS_DIR=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network)
      NETWORK="${2:-}"
      shift 2
      ;;
    --results-dir)
      RESULTS_DIR="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ "$NETWORK" != "localnet" ]]; then
  echo "builtin validator probes currently support only --network localnet" >&2
  exit 2
fi

if [[ -z "${FIVE_PROGRAM_ID:-}" ]]; then
  echo "missing FIVE_PROGRAM_ID" >&2
  exit 2
fi

if [[ -z "${VM_STATE_PDA:-}" ]]; then
  echo "missing VM_STATE_PDA" >&2
  exit 2
fi

if [[ -z "${FIVE_KEYPAIR_PATH:-}" ]]; then
  export FIVE_KEYPAIR_PATH="$HOME/.config/solana/id.json"
fi

if [[ -z "${FIVE_RPC_URL:-}" ]]; then
  export FIVE_RPC_URL="http://127.0.0.1:8899"
fi

if [[ -z "$RESULTS_DIR" ]]; then
  RESULTS_DIR="$ROOT_DIR/target/sdk-validator-runs/$(date -u +%Y-%m-%dT%H-%M-%SZ)/builtin-localnet"
fi

mkdir -p "$RESULTS_DIR"

export FIVE_CU_NETWORK="$NETWORK"
export FIVE_CU_PROGRAM_ID="$FIVE_PROGRAM_ID"
export FIVE_CU_PAYER_KEYPAIR="$FIVE_KEYPAIR_PATH"
export FIVE_CU_RPC_URL="$FIVE_RPC_URL"

bash "$ROOT_DIR/scripts/run-validator-cargo-probe.sh" \
  runtime_validator_stdlib_probe_tests \
  validator_stdlib_time_and_sysvar_onchain \
  "$RESULTS_DIR/runtime_validator_stdlib_probe_tests-validator_stdlib_time_and_sysvar_onchain.json"

bash "$ROOT_DIR/scripts/run-validator-cargo-probe.sh" \
  runtime_validator_account_probe_tests \
  validator_account_probe_onchain \
  "$RESULTS_DIR/runtime_validator_account_probe_tests-validator_account_probe_onchain.json"

echo "Builtin validator probe artifacts written to $RESULTS_DIR"
