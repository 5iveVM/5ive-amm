#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

NETWORK="localnet"
RPC_URL=""
PROGRAM_SO="${ROOT_DIR}/target/deploy/five.so"
PROGRAM_KEYPAIR=""
PAYER_KEYPAIR="${HOME}/.config/solana/id.json"
PROGRAM_ID=""
VM_STATE=""
SHARDS=""
SKIP_DEPLOY=0

usage() {
  cat <<USAGE
Usage: $0 [options]

Minimal VM rollout only (no tests):
  1) Deploy VM program
  2) Initialize VM state PDA
  3) Initialize fee-vault shards

Options:
  --network <localnet|devnet|mainnet>
  --rpc-url <url>                       Optional; defaults by network
  --program-so <path>                   Default: target/deploy/five.so
  --program-keypair <path>              Required unless --skip-deploy
  --payer-keypair <path>                Default: ~/.config/solana/id.json
  --program-id <pubkey>                 Optional; derived from --program-keypair when omitted
  --vm-state <pubkey>                   Optional; derived by init script when omitted
  --shards <N>                          Optional; defaults from constants.vm.toml
  --skip-deploy                         Skip program deploy step
  -h, --help
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --network) NETWORK="${2:-}"; shift 2 ;;
    --rpc-url) RPC_URL="${2:-}"; shift 2 ;;
    --program-so) PROGRAM_SO="${2:-}"; shift 2 ;;
    --program-keypair) PROGRAM_KEYPAIR="${2:-}"; shift 2 ;;
    --payer-keypair) PAYER_KEYPAIR="${2:-}"; shift 2 ;;
    --program-id) PROGRAM_ID="${2:-}"; shift 2 ;;
    --vm-state) VM_STATE="${2:-}"; shift 2 ;;
    --shards) SHARDS="${2:-}"; shift 2 ;;
    --skip-deploy) SKIP_DEPLOY=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage >&2; exit 1 ;;
  esac
done

if [[ "${NETWORK}" != "localnet" && "${NETWORK}" != "devnet" && "${NETWORK}" != "mainnet" ]]; then
  echo "Invalid --network: ${NETWORK}" >&2
  exit 1
fi

if [[ -z "${RPC_URL}" ]]; then
  case "${NETWORK}" in
    localnet) RPC_URL="http://127.0.0.1:8899" ;;
    devnet) RPC_URL="https://api.devnet.solana.com" ;;
    mainnet) RPC_URL="https://api.mainnet-beta.solana.com" ;;
  esac
fi

if [[ ! -f "${PAYER_KEYPAIR}" ]]; then
  echo "Missing payer keypair: ${PAYER_KEYPAIR}" >&2
  exit 1
fi

if [[ "${SKIP_DEPLOY}" -eq 0 ]]; then
  if [[ -z "${PROGRAM_KEYPAIR}" ]]; then
    echo "--program-keypair is required unless --skip-deploy is set" >&2
    exit 1
  fi
  if [[ ! -f "${PROGRAM_KEYPAIR}" ]]; then
    echo "Missing program keypair: ${PROGRAM_KEYPAIR}" >&2
    exit 1
  fi
  if [[ ! -f "${PROGRAM_SO}" ]]; then
    echo "Missing program binary: ${PROGRAM_SO}" >&2
    exit 1
  fi
fi

if [[ -z "${PROGRAM_ID}" ]]; then
  if [[ -n "${PROGRAM_KEYPAIR}" && -f "${PROGRAM_KEYPAIR}" ]]; then
    PROGRAM_ID="$(solana address -k "${PROGRAM_KEYPAIR}")"
  else
    echo "Missing --program-id and cannot derive it without --program-keypair" >&2
    exit 1
  fi
fi

echo "=== Minimal VM Deploy Plan ==="
echo "Network:         ${NETWORK}"
echo "RPC URL:         ${RPC_URL}"
echo "Program ID:      ${PROGRAM_ID}"
echo "Program binary:  ${PROGRAM_SO}"
echo "Program keypair: ${PROGRAM_KEYPAIR:-<not set>}"
echo "Payer keypair:   ${PAYER_KEYPAIR}"
echo "VM state:        ${VM_STATE:-<derive in init script>}"
echo "Shards:          ${SHARDS:-<cluster default>}"
echo "Steps: deploy + init vm_state + init fee_vaults"
echo "=============================="

if [[ "${SKIP_DEPLOY}" -eq 0 ]]; then
  echo "[1/3] Deploying VM program"
  solana program deploy "${PROGRAM_SO}" \
    --program-id "${PROGRAM_KEYPAIR}" \
    --url "${RPC_URL}" \
    --keypair "${PAYER_KEYPAIR}"
else
  echo "[1/3] Skipping deploy (--skip-deploy)"
fi

echo "[2/3] Initializing VM state"
INIT_VM_CMD=(
  node "${ROOT_DIR}/scripts/init-localnet-vm-state.mjs"
  --network "${NETWORK}"
  --rpc-url "${RPC_URL}"
  --program-id "${PROGRAM_ID}"
  --keypair "${PAYER_KEYPAIR}"
)
"${INIT_VM_CMD[@]}"

echo "[3/3] Initializing fee vault shards"
INIT_VAULT_CMD=(
  node "${ROOT_DIR}/scripts/init-devnet-fee-vaults.mjs"
  --network "${NETWORK}"
  --rpc-url "${RPC_URL}"
  --program-id "${PROGRAM_ID}"
  --keypair "${PAYER_KEYPAIR}"
)
if [[ -n "${VM_STATE}" ]]; then
  INIT_VAULT_CMD+=(--vm-state "${VM_STATE}")
fi
if [[ -n "${SHARDS}" ]]; then
  INIT_VAULT_CMD+=(--shards "${SHARDS}")
fi
"${INIT_VAULT_CMD[@]}"

echo "✅ Minimal VM deploy flow complete (no validator tests were run)."
