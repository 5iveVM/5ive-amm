#!/usr/bin/env node

/**
 * Initialize VM state on localnet
 * Must be run before fee vault initialization
 */

import { readFile } from 'node:fs/promises';
import web3 from '../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';
const { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } = web3;
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

const LOCALNET_RPC = 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = '3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1';
const VM_STATE_SEED = Buffer.from('vm_state', 'utf-8');
const INIT_INSTRUCTION = 0; // Initialize

async function main() {
  const connection = new Connection(LOCALNET_RPC, 'confirmed');

  // Load payer keypair
  const keypairPath = path.join(os.homedir(), '.config/solana/id.json');
  if (!fs.existsSync(keypairPath)) {
    throw new Error(`Keypair not found at ${keypairPath}`);
  }

  const keypairData = JSON.parse(await readFile(keypairPath, 'utf-8'));
  const payer = Keypair.fromSecretKey(Uint8Array.from(keypairData));

  // Derive VM state PDA
  const [vmStatePda, vmStateBump] = PublicKey.findProgramAddressSync(
    [VM_STATE_SEED],
    new PublicKey(FIVE_PROGRAM_ID)
  );

  console.log(`\n📋 Initializing VM State on Localnet`);
  console.log(`   RPC: ${LOCALNET_RPC}`);
  console.log(`   Payer: ${payer.publicKey.toBase58()}`);
  console.log(`   Program ID: ${FIVE_PROGRAM_ID}`);
  console.log(`   VM State PDA: ${vmStatePda.toBase58()}`);
  console.log(`   VM State Bump: ${vmStateBump}\n`);

  // Check if VM state already exists
  const vmStateInfo = await connection.getAccountInfo(vmStatePda);
  if (vmStateInfo) {
    console.log(`✓ VM State already initialized\n`);
    return;
  }

  // Get balance
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`✓ Payer balance: ${balance / 1e9} SOL\n`);

  if (balance < 0.5e9) {
    throw new Error('Insufficient SOL balance. Need at least 0.5 SOL.');
  }

  // Create initialize instruction with 4 accounts
  const data = Buffer.from([INIT_INSTRUCTION, vmStateBump]);

  const instruction = new TransactionInstruction({
    programId: new PublicKey(FIVE_PROGRAM_ID),
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

  try {
    console.log(`⏳ Sending VM state initialization...`);
    const sig = await connection.sendRawTransaction(tx.serialize(), {
      skipPreflight: true,
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
