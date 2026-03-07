#!/usr/bin/env bash
set -euo pipefail

PROGRAM_ID="${FIVE_VM_PROGRAM_ID:-5ive58PJUPaTyAe7tvU1bvBi25o7oieLLTRsJDoQNJst}"
VM_STATE_ACCOUNT="${FIVE_VM_STATE_ACCOUNT:-8ip3qGGETf8774jo6kXbsTTrMm5V9bLuGC4znmyZjT3z}"
RPC_URL="${FIVE_LOCALNET_RPC_URL:-http://127.0.0.1:8899}"
WS_URL="${FIVE_LOCALNET_WS_URL:-ws://127.0.0.1:8900}"
LEDGER_DIR="${FIVE_LOCALNET_LEDGER:-.localnet-vm-parity-ledger}"
UPSTREAM_URL="${FIVE_UPSTREAM_URL:-devnet}"
LOG_FILE="${FIVE_LOCALNET_LOG:-/tmp/five-localnet-vm-parity.log}"

echo "[localnet-vm-parity] starting validator"
echo "  program id: ${PROGRAM_ID}"
echo "  vm state:   ${VM_STATE_ACCOUNT}"
echo "  ledger:     ${LEDGER_DIR}"
echo "  upstream:   ${UPSTREAM_URL}"
echo "  log file:   ${LOG_FILE}"

mkdir -p "${LEDGER_DIR}"

nohup solana-test-validator \
  --reset \
  --ledger "${LEDGER_DIR}" \
  --url "${UPSTREAM_URL}" \
  --clone-upgradeable-program "${PROGRAM_ID}" \
  --clone "${VM_STATE_ACCOUNT}" \
  --rpc-port 8899 \
  --faucet-port 9900 \
  >"${LOG_FILE}" 2>&1 </dev/null &

VALIDATOR_PID=$!
echo "[localnet-vm-parity] pid=${VALIDATOR_PID}"

for _ in $(seq 1 60); do
  if solana -u "${RPC_URL}" cluster-version >/dev/null 2>&1; then
    echo "[localnet-vm-parity] validator is healthy at ${RPC_URL}"
    echo "[localnet-vm-parity] websocket expected at ${WS_URL}"
    echo "[localnet-vm-parity] stop with: kill ${VALIDATOR_PID}"
    exit 0
  fi
  sleep 1
done

echo "[localnet-vm-parity] validator failed to become healthy"
echo "[localnet-vm-parity] recent logs:"
tail -n 120 "${LOG_FILE}" || true
exit 1
