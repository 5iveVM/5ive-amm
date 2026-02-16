#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLUSTER=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cluster)
      CLUSTER="${2:-}"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ -z "${CLUSTER}" ]]; then
  echo "Usage: $0 --cluster <localnet|devnet|mainnet>" >&2
  exit 1
fi

if [[ "${CLUSTER}" != "localnet" && "${CLUSTER}" != "devnet" && "${CLUSTER}" != "mainnet" ]]; then
  echo "Invalid cluster: ${CLUSTER}" >&2
  exit 1
fi

cd "${ROOT_DIR}"

echo "[build-five-solana-cluster] generating constants for ${CLUSTER}"
node scripts/generate-vm-constants.mjs --cluster "${CLUSTER}"

echo "[build-five-solana-cluster] building SBF artifact"
cargo-build-sbf --manifest-path five-solana/Cargo.toml --no-default-features --features production --sbf-out-dir target/deploy

echo "[build-five-solana-cluster] summary"
node -e '
const fs = require("fs");
const web3 = require("./five-cli/node_modules/@solana/web3.js/lib/index.cjs.js");
const src = fs.readFileSync("five-solana/src/generated_constants.rs","utf8");
const m = (re) => { const x = src.match(re); return x ? x[1] : "n/a"; };
const bytesToBase58 = (blk) => {
  const arr = blk.split(",").map(s=>s.trim()).filter(s=>s.startsWith("0x")).map(s=>parseInt(s,16));
  if (arr.length !== 32) return "n/a";
  return new web3.PublicKey(new Uint8Array(arr)).toBase58();
};
console.log("  cluster: " + m(/pub const GENERATED_CLUSTER: &str = "([^"]+)";/));
console.log("  program_id: " + m(/pub const VM_PROGRAM_ID: &str = "([^"]+)";/));
const vmBlock = src.match(/pub const HARDCODED_VM_STATE_PDA: \[u8; 32\] = \[([\s\S]*?)\];/);
if (vmBlock) {
  console.log("  vm_state: " + bytesToBase58(vmBlock[1]));
}
for (const match of src.matchAll(/pub const HARDCODED_FEE_VAULT_(\d+): \[u8; 32\] = \[([\s\S]*?)\];/g)) {
  const idx = match[1];
  console.log(`  fee_vault[${idx}]: ` + bytesToBase58(match[2]));
}
'

echo "[build-five-solana-cluster] done"
