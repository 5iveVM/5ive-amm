#!/usr/bin/env node

/**
 * Initialize VM state on localnet/devnet/mainnet.
 * Must be run before fee vault initialization.
 */

import { readFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import web3 from '../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';
import { loadClusterConfig, resolveClusterFromEnvOrDefault } from './lib/vm-cluster-config.mjs';
const { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } = web3;
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));

const VM_STATE_SEED = Buffer.from('vm_state', 'utf-8');
const INIT_INSTRUCTION = 0; // Initialize
// VM state account base (56 bytes) + canonical service registry extension (160 bytes).
const VM_STATE_SPACE = 216;

function parseArgs(argv) {
  const args = { network: 'localnet' };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--network' && argv[i + 1]) args.network = argv[++i];
    else if (a === '--rpc-url' && argv[i + 1]) args.rpcUrl = argv[++i];
    else if (a === '--program-id' && argv[i + 1]) args.programId = argv[++i];
    else if (a === '--keypair' && argv[i + 1]) args.keypairPath = argv[++i];
    else if (a === '--strict') args.strict = true;
  }
  return args;
}

function defaultsForNetwork(network) {
  if (network === 'devnet') {
    return { rpcUrl: 'https://api.devnet.solana.com' };
  }
  if (network === 'mainnet' || network === 'mainnet-beta') {
    return { rpcUrl: 'https://api.mainnet-beta.solana.com' };
  }
  return { rpcUrl: 'http://127.0.0.1:8899' };
}

async function main() {
  const args = parseArgs(process.argv);
  const cluster = args.network === 'localnet' ? 'localnet' : (args.network === 'mainnet' || args.network === 'mainnet-beta' ? 'mainnet' : 'devnet');
  const defaults = defaultsForNetwork(args.network);
  const rpcUrl = args.rpcUrl || process.env.FIVE_RPC_URL || defaults.rpcUrl;
  const configProgramId = loadClusterConfig({ cluster: process.env.FIVE_VM_CLUSTER || cluster || resolveClusterFromEnvOrDefault() }).programId;
  const programIdRaw = args.programId || process.env.FIVE_PROGRAM_ID || configProgramId;
  const programId = new PublicKey(programIdRaw);

  const connection = new Connection(rpcUrl, 'confirmed');

  // Load payer keypair
  const keypairPath =
    args.keypairPath ||
    process.env.FIVE_KEYPAIR_PATH ||
    path.join(os.homedir(), '.config/solana/id.json');
  if (!fs.existsSync(keypairPath)) {
    throw new Error(`Keypair not found at ${keypairPath}`);
  }

  const keypairData = JSON.parse(await readFile(keypairPath, 'utf-8'));
  const payer = Keypair.fromSecretKey(Uint8Array.from(keypairData));

  // Derive VM state PDA
  const [vmStatePda, vmStateBump] = PublicKey.findProgramAddressSync(
    [VM_STATE_SEED],
    programId
  );

  console.log(`\n📋 Initializing VM State on ${args.network.toUpperCase()}`);
  console.log(`   RPC: ${rpcUrl}`);
  console.log(`   Payer: ${payer.publicKey.toBase58()}`);
  console.log(`   Program ID: ${programId.toBase58()}`);
  console.log(`   VM State PDA: ${vmStatePda.toBase58()}`);
  console.log(`   VM State Bump: ${vmStateBump}\n`);

  // Check if VM state already exists
  const vmStateInfo = await connection.getAccountInfo(vmStatePda);
  if (vmStateInfo) {
    console.log(`✓ VM State already initialized\n`);
    return;
  }

  if (args.strict) {
    console.log('⏳ Strict parity precheck (compiled constants vs chain)...');
    const result = spawnSync('node', [
      path.join(SCRIPT_DIR, 'check-vm-constants-parity.mjs'),
      '--rpc-url', rpcUrl,
      '--program-id', programId.toBase58(),
      '--vm-state', vmStatePda.toBase58(),
    ], { stdio: 'inherit' });
    if (result.status !== 0) {
      throw new Error('Strict parity precheck failed');
    }
  }

  // Get balance
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`✓ Payer balance: ${balance / 1e9} SOL\n`);

  // Create initialize instruction with 4 accounts
  const data = Buffer.from([INIT_INSTRUCTION, vmStateBump]);

  const instruction = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: vmStatePda, isSigner: false, isWritable: true },
      { pubkey: payer.publicKey, isSigner: true, isWritable: false }, // authority (must sign)
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },   // payer for account creation
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  });

  // Create and send transaction
  const tx = new Transaction().add(instruction);
  const { blockhash } = await connection.getLatestBlockhash('confirmed');
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  tx.partialSign(payer);

  // Estimate minimum required SOL for this single initialization:
  // rent-exempt VM state account + one tx fee + small safety margin.
  const vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SPACE);
  const txFee = Number((await connection.getFeeForMessage(tx.compileMessage(), 'confirmed')).value || 0);
  const safety = 10_000; // 0.00001 SOL
  const requiredLamports = vmStateRent + txFee + safety;
  console.log(
    `✓ Estimated required: ${(requiredLamports / 1e9).toFixed(9)} SOL ` +
    `(rent ${(vmStateRent / 1e9).toFixed(9)} + fee ${(txFee / 1e9).toFixed(9)} + safety ${(safety / 1e9).toFixed(9)})`
  );
  if (balance < requiredLamports) {
    const shortfall = requiredLamports - balance;
    throw new Error(
      `Insufficient SOL balance. Need ${(requiredLamports / 1e9).toFixed(9)} SOL ` +
      `(short by ${(shortfall / 1e9).toFixed(9)} SOL).`
    );
  }

  try {
    console.log(`⏳ Sending VM state initialization...`);
    const sig = await connection.sendRawTransaction(tx.serialize(), {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
      maxRetries: 5,
    });

    // Wait for confirmation
    await connection.confirmTransaction(sig, 'confirmed');
    console.log(`✓ VM State initialized`);
    console.log(`   Signature: ${sig}\n`);
  } catch (e) {
    console.error(`✗ Failed - ${e.message}`);
    if (e.logs) {
      console.error('   Program logs:', e.logs);
    }
    throw e;
  }

  console.log(`✅ VM state initialization complete!\n`);
}

main().catch(err => {
  console.error('\n❌ Error:', err.message);
  process.exit(1);
});
