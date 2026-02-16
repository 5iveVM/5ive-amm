#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import web3 from '../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';

const { Connection, PublicKey } = web3;
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function parseArgs(argv) {
  const args = {};
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--rpc-url' && argv[i + 1]) args.rpcUrl = argv[++i];
    else if (a === '--program-id' && argv[i + 1]) args.programId = argv[++i];
    else if (a === '--vm-state' && argv[i + 1]) args.vmState = argv[++i];
    else if (a === '--constants' && argv[i + 1]) args.constantsPath = argv[++i];
  }
  return args;
}

function parseGeneratedConstants(raw) {
  const programMatch = raw.match(/pub const VM_PROGRAM_ID: &str = "([^"]+)";/);
  if (!programMatch) throw new Error('Unable to parse VM_PROGRAM_ID from generated constants');
  const programId = new PublicKey(programMatch[1]);

  const vmMatch = raw.match(/pub const HARDCODED_VM_STATE_PDA: \[u8; 32\] = \[([\s\S]*?)\];/);
  if (!vmMatch) throw new Error('Unable to parse HARDCODED_VM_STATE_PDA');
  const vmBytes = vmMatch[1]
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s.startsWith('0x'))
    .map((s) => Number.parseInt(s, 16));
  if (vmBytes.length !== 32) throw new Error(`Invalid VM PDA byte length: ${vmBytes.length}`);
  const vmState = new PublicKey(Uint8Array.from(vmBytes));

  const vaults = [];
  const re = /pub const HARDCODED_FEE_VAULT_(\d+): \[u8; 32\] = \[([\s\S]*?)\];/g;
  let m;
  while ((m = re.exec(raw)) !== null) {
    const idx = Number.parseInt(m[1], 10);
    const bytes = m[2]
      .split(',')
      .map((s) => s.trim())
      .filter((s) => s.startsWith('0x'))
      .map((s) => Number.parseInt(s, 16));
    if (bytes.length !== 32) throw new Error(`Invalid fee vault ${idx} byte length: ${bytes.length}`);
    vaults.push({ idx, pubkey: new PublicKey(Uint8Array.from(bytes)) });
  }
  vaults.sort((a, b) => a.idx - b.idx);
  return { programId, vmState, vaults };
}

async function main() {
  const args = parseArgs(process.argv);
  const constantsPath = path.resolve(args.constantsPath || path.join(__dirname, '..', 'five-solana', 'src', 'generated_constants.rs'));
  const raw = fs.readFileSync(constantsPath, 'utf-8');
  const constants = parseGeneratedConstants(raw);
  const rpcUrl = args.rpcUrl || process.env.FIVE_RPC_URL || 'http://127.0.0.1:8899';
  const connection = new Connection(rpcUrl, 'confirmed');

  if (args.programId) {
    const inputProgram = new PublicKey(args.programId);
    if (!inputProgram.equals(constants.programId)) {
      throw new Error(`program_id mismatch: generated=${constants.programId.toBase58()} input=${inputProgram.toBase58()}`);
    }
  }
  if (args.vmState) {
    const inputVm = new PublicKey(args.vmState);
    if (!inputVm.equals(constants.vmState)) {
      throw new Error(`vm_state mismatch: generated=${constants.vmState.toBase58()} input=${inputVm.toBase58()}`);
    }
  }

  const programInfo = await connection.getAccountInfo(constants.programId);
  if (!programInfo) {
    throw new Error(`program account missing on chain: ${constants.programId.toBase58()}`);
  }
  const vmInfo = await connection.getAccountInfo(constants.vmState);
  if (!vmInfo) {
    throw new Error(`vm_state account missing on chain: ${constants.vmState.toBase58()}`);
  }
  if (!vmInfo.owner.equals(constants.programId)) {
    throw new Error(`vm_state owner mismatch: owner=${vmInfo.owner.toBase58()} program=${constants.programId.toBase58()}`);
  }

  for (const v of constants.vaults) {
    const info = await connection.getAccountInfo(v.pubkey);
    if (!info) {
      throw new Error(`fee_vault[${v.idx}] missing on chain: ${v.pubkey.toBase58()}`);
    }
    if (!info.owner.equals(constants.programId)) {
      throw new Error(`fee_vault[${v.idx}] owner mismatch: owner=${info.owner.toBase58()} program=${constants.programId.toBase58()}`);
    }
  }

  console.log('VM_CONSTANTS_PARITY_OK');
  console.log(`  rpc_url: ${rpcUrl}`);
  console.log(`  program_id: ${constants.programId.toBase58()}`);
  console.log(`  vm_state: ${constants.vmState.toBase58()}`);
  for (const v of constants.vaults) {
    console.log(`  fee_vault[${v.idx}]: ${v.pubkey.toBase58()}`);
  }
}

main().catch((e) => {
  console.error(`VM_CONSTANTS_PARITY_FAIL: ${e.message}`);
  process.exit(1);
});
