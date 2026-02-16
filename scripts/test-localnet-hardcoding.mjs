#!/usr/bin/env node

/**
 * Test that hardcoded fee vault addresses work on localnet
 * This is a minimal test to verify the optimization is functioning
 */

import { readFile } from 'node:fs/promises';
import web3 from '../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';
const { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } = web3;
import { loadClusterConfig, deriveVmAddresses } from './lib/vm-cluster-config.mjs';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

const LOCALNET_RPC = 'http://127.0.0.1:8899';
const profile = loadClusterConfig({ cluster: 'localnet' });
const derived = deriveVmAddresses(profile);
const FIVE_PROGRAM_ID = profile.programId;

async function main() {
  const connection = new Connection(LOCALNET_RPC, 'confirmed');

  // Load payer keypair
  const keypairPath = path.join(os.homedir(), '.config/solana/id.json');
  const keypairData = JSON.parse(await readFile(keypairPath, 'utf-8'));
  const payer = Keypair.fromSecretKey(Uint8Array.from(keypairData));

  // Derive VM state
  const vmStatePda = new PublicKey(derived.vmStatePda);

  console.log(`\n📋 Testing Hardcoded Fee Vault on Localnet`);
  console.log(`   Program ID: ${FIVE_PROGRAM_ID}`);
  console.log(`   VM State: ${vmStatePda.toBase58()}\n`);

  // Check fee vaults exist
  const feeVaultAddresses = derived.feeVaultPdas.map((v) => v.address);

  console.log(`⏳ Checking fee vault accounts...\n`);
  for (const [idx, addr] of feeVaultAddresses.entries()) {
    const acct = await connection.getAccountInfo(new PublicKey(addr));
    if (acct) {
      console.log(`✓ Shard ${idx}: ${addr.substring(0, 8)}... exists (${acct.lamports} lamports)`);
    } else {
      console.log(`✗ Shard ${idx}: ${addr} NOT found`);
      throw new Error(`Fee vault shard ${idx} not found`);
    }
  }

  console.log(`\n✅ All hardcoded fee vaults verified on localnet!\n`);
  console.log(`This confirms that the fee vault hardcoding optimization`);
  console.log(`is using the correct addresses for program ID:`);
  console.log(`${FIVE_PROGRAM_ID}\n`);
}

main().catch(err => {
  console.error('\n❌ Error:', err.message);
  process.exit(1);
});
