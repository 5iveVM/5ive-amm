#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

NETWORK="localnet"
RESULTS_DIR=""
PROGRAM_ID="${FIVE_PROGRAM_ID:-}"
VM_STATE="${VM_STATE_PDA:-}"
KEYPAIR="${FIVE_KEYPAIR_PATH:-$HOME/.config/solana/id.json}"

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
    --program-id)
      PROGRAM_ID="${2:-}"
      shift 2
      ;;
    --vm-state)
      VM_STATE="${2:-}"
      shift 2
      ;;
    --keypair)
      KEYPAIR="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$RESULTS_DIR" ]]; then
  RESULTS_DIR="$ROOT_DIR/target/sdk-validator-runs/$(date -u +%Y-%m-%dT%H-%M-%SZ)/builtin-localnet"
fi

export FIVE_PROGRAM_ID="$PROGRAM_ID"
export VM_STATE_PDA="$VM_STATE"
export FIVE_KEYPAIR_PATH="$KEYPAIR"

bash "$ROOT_DIR/scripts/run-dsl-builtin-validator-probes.sh" \
  --network "$NETWORK" \
  --results-dir "$RESULTS_DIR"

node "$ROOT_DIR/scripts/run-dsl-builtin-validator-matrix.mjs" \
  --network "$NETWORK" \
  --results-dir "$RESULTS_DIR" \
  --program-id "$PROGRAM_ID" \
  --vm-state "$VM_STATE" \
  --keypair "$KEYPAIR"
