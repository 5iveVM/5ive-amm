#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TS="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="${ROOT_DIR}/.reports/mainnet/${TS}"
mkdir -p "${OUT_DIR}"

copy_if_exists() {
  local src="$1"
  local dst="$2"
  if [[ -e "${src}" ]]; then
    mkdir -p "$(dirname "${dst}")"
    cp -R "${src}" "${dst}"
  fi
}

copy_if_exists "${ROOT_DIR}/target/mvp-gate/report.json" "${OUT_DIR}/gate-report.json"
copy_if_exists "${ROOT_DIR}/target/mvp-gate/report.md" "${OUT_DIR}/gate-report.md"
copy_if_exists "${ROOT_DIR}/target/sdk-validator-runs" "${OUT_DIR}/sdk-validator-runs"
copy_if_exists "${ROOT_DIR}/five-solana/constants.vm.toml" "${OUT_DIR}/constants.vm.toml"
copy_if_exists "${ROOT_DIR}/five-solana/src/generated_constants.rs" "${OUT_DIR}/generated_constants.rs"
copy_if_exists "${ROOT_DIR}/docs/mainnet-rollout-runbook.md" "${OUT_DIR}/mainnet-rollout-runbook.md"

copy_if_exists "${ROOT_DIR}/5ive-amm/deployment-config.mainnet.json" "${OUT_DIR}/projects/5ive-amm.deployment-config.mainnet.json"
copy_if_exists "${ROOT_DIR}/5ive-cfd/deployment-config.mainnet.json" "${OUT_DIR}/projects/5ive-cfd.deployment-config.mainnet.json"
copy_if_exists "${ROOT_DIR}/5ive-escrow/deployment-config.mainnet.json" "${OUT_DIR}/projects/5ive-escrow.deployment-config.mainnet.json"
copy_if_exists "${ROOT_DIR}/5ive-lending/deployment-config.mainnet.json" "${OUT_DIR}/projects/5ive-lending.deployment-config.mainnet.json"
copy_if_exists "${ROOT_DIR}/5ive-token/deployment-config.mainnet.json" "${OUT_DIR}/projects/5ive-token.deployment-config.mainnet.json"

cat > "${OUT_DIR}/README.txt" <<README
Mainnet rollout evidence bundle
Generated: ${TS}

Includes:
- Gate reports
- SDK validator run outputs (if available)
- VM constants snapshots
- Mainnet runbook snapshot
- Core project mainnet deployment config snapshots
README

echo "Mainnet rollout evidence bundle created: ${OUT_DIR}"
