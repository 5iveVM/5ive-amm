#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import bs58Pkg from "../five-cli/node_modules/bs58/index.js";
const bs58 = bs58Pkg.default ?? bs58Pkg;

function readArg(name, fallback = undefined) {
  const flag = `--${name}`;
  const idx = process.argv.indexOf(flag);
  if (idx >= 0 && idx + 1 < process.argv.length) return process.argv[idx + 1];
  return fallback;
}

function runSolana(args) {
  const out = execFileSync("solana", args, { encoding: "utf8" }).trim();
  return out;
}

const rpcUrl = readArg("rpc-url", "http://127.0.0.1:8899");
const programIdRaw = readArg("program-id", "4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d");
const vmStateRaw = readArg("vm-state", "8ip3qGGETf8774jo6kXbsTTrMm5V9bLuGC4znmyZjT3z");
const expectedAuthorityRaw = readArg("expected-authority");
const expectedFeeRecipientRaw = readArg("expected-fee-recipient");
const expectedDeployFeeRaw = readArg("expected-deploy-fee", "10000");
const expectedExecuteFeeRaw = readArg("expected-execute-fee", "85734");

if (!expectedAuthorityRaw || !expectedFeeRecipientRaw) {
  console.error("missing --expected-authority <pubkey> and/or --expected-fee-recipient <pubkey>");
  process.exit(2);
}

const expectedAuthority = expectedAuthorityRaw;
const expectedFeeRecipient = expectedFeeRecipientRaw;
const expectedDeployFee = Number(expectedDeployFeeRaw);
const expectedExecuteFee = Number(expectedExecuteFeeRaw);

const pdaResult = JSON.parse(
  runSolana([
    "-u",
    rpcUrl,
    "find-program-derived-address",
    programIdRaw,
    "string:vm_state",
    "--output",
    "json-compact",
  ]),
);
const canonicalVmState = pdaResult.address;
const bump = pdaResult.bumpSeed;

if (canonicalVmState !== vmStateRaw) {
  console.error(`vm_state mismatch: expected canonical ${canonicalVmState}, got ${vmStateRaw}`);
  process.exit(1);
}

const accountResult = JSON.parse(
  runSolana([
    "-u",
    rpcUrl,
    "account",
    vmStateRaw,
    "--output",
    "json-compact",
  ]),
);
const owner = accountResult.account.owner;
const rawBase64 = accountResult.account.data[0];
const data = Buffer.from(rawBase64, "base64");

if (owner !== programIdRaw) {
  console.error(`vm_state owner mismatch: expected ${programIdRaw}, got ${owner}`);
  process.exit(1);
}
if (data.length < 88) {
  console.error(`vm_state data too small: expected >=88, got ${data.length}`);
  process.exit(1);
}

const authorityBytes = data.subarray(0, 32);
const authorityBase58 = bs58.encode(authorityBytes);
const feeRecipientBytes = data.subarray(32, 64);
const feeRecipientBase58 = bs58.encode(feeRecipientBytes);
const scriptCount = data.readBigUInt64LE(64);
const deployFeeLamports = data.readUInt32LE(72);
const executeFeeLamports = data.readUInt32LE(76);
const isInitialized = data[80] === 1;

console.log("VM_STATE_PARITY");
console.log(`  rpc_url: ${rpcUrl}`);
console.log(`  program_id: ${programIdRaw}`);
console.log(`  vm_state: ${vmStateRaw}`);
console.log(`  canonical_bump: ${bump}`);
console.log(`  owner: ${owner}`);
console.log(`  authority: ${authorityBase58}`);
console.log(`  fee_recipient: ${feeRecipientBase58}`);
console.log(`  script_count: ${scriptCount.toString()}`);
console.log(`  deploy_fee_lamports: ${deployFeeLamports}`);
console.log(`  execute_fee_lamports: ${executeFeeLamports}`);
console.log(`  is_initialized: ${isInitialized}`);

if (authorityBase58 !== expectedAuthority) {
  console.error(`authority mismatch: expected ${expectedAuthority}, got ${authorityBase58}`);
  process.exit(1);
}
if (feeRecipientBase58 !== expectedFeeRecipient) {
  console.error(`fee recipient mismatch: expected ${expectedFeeRecipient}, got ${feeRecipientBase58}`);
  process.exit(1);
}
if (deployFeeLamports !== expectedDeployFee) {
  console.error(`deploy fee mismatch: expected ${expectedDeployFee}, got ${deployFeeLamports}`);
  process.exit(1);
}
if (executeFeeLamports !== expectedExecuteFee) {
  console.error(`execute fee mismatch: expected ${expectedExecuteFee}, got ${executeFeeLamports}`);
  process.exit(1);
}
if (!isInitialized) {
  console.error("vm_state is not initialized");
  process.exit(1);
}

console.log("VM_STATE_PARITY_OK");
