#!/usr/bin/env node

/**
 * Test that the hardcoded fee vault addresses are being verified correctly
 * on localnet
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

// Hardcoded addresses from common.rs
const HARDCODED_FEE_VAULT_0 = '3UYQEdrwkD7YqZzQm9R7yt3NYJNPBcDUYBzxNqSzSBGN';
const HARDCODED_FEE_VAULT_1 = 'GBLnGfT3PvfWRz6Y8hY5gB4JknTpfpRG3oWD1xqkV8p';

async function main() {
  const connection = new Connection(LOCALNET_RPC, 'confirmed');

  // Load payer keypair
  const keypairPath = path.join(os.homedir(), '.config/solana/id.json');
  if (!fs.existsSync(keypairPath)) {
    throw new Error(`Keypair not found at ${keypairPath}`);
  }

  const keypairData = JSON.parse(await readFile(keypairPath, 'utf-8'));
  const payer = Keypair.fromSecretKey(Uint8Array.from(keypairData));

  // Derive VM state for localnet
  const [vmStatePda] = PublicKey.findProgramAddressSync(
    [VM_STATE_SEED],
    new PublicKey(FIVE_PROGRAM_ID)
  );

  console.log(`\n📋 Testing Hardcoded Fee Vault Verification on Localnet`);
  console.log(`   RPC: ${LOCALNET_RPC}`);
  console.log(`   Program ID: ${FIVE_PROGRAM_ID}`);
  console.log(`   VM State: ${vmStatePda.toBase58()}\n`);

  // Set fees to enable fee collection
  console.log(`⏳ Setting deploy and execute fees...`);
  const setFeesData = Buffer.from([6, 10, 0x27, 0x00, 0x00, 0xE6, 0x4E, 0x01, 0x00]);

  const setFeesInstruction = new TransactionInstruction({
    programId: new PublicKey(FIVE_PROGRAM_ID),
    keys: [
      { pubkey: vmStatePda, isSigner: false, isWritable: true },
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
    ],
    data: setFeesData,
  });

  const setFeesTx = new Transaction().add(setFeesInstruction);
  const { blockhash } = await connection.getLatestBlockhash('confirmed');
  setFeesTx.recentBlockhash = blockhash;
  setFeesTx.feePayer = payer.publicKey;
  setFeesTx.partialSign(payer);

  try {
    const sig = await connection.sendRawTransaction(setFeesTx.serialize(), {
      skipPreflight: true,
      preflightCommitment: 'confirmed',
      maxRetries: 5,
    });
    await connection.confirmTransaction(sig, 'confirmed');
    console.log(`✓ Fees set successfully\n`);
  } catch (e) {
    console.error(`✗ Failed to set fees: ${e.message}`);
    throw e;
  }

  // Now test deploy with hardcoded fee vault
  console.log(`⏳ Testing deploy instruction with hardcoded fee vault verification...`);

  // Create a minimal valid bytecode
  const bytecode = Buffer.from([0x35, 0x49, 0x56, 0x45, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00]);

  // Create deploy instruction
  // Format: discriminator(1) + bytecode_len(4) + permissions(1) + metadata_len(4) + metadata + bytecode + fee_shard_index(1)
  const deployData = Buffer.concat([
    Buffer.from([8]), // DEPLOY_INSTRUCTION discriminator
    Buffer.from([bytecode.length, 0, 0, 0]), // bytecode length (little-endian u32)
    Buffer.from([0]), // permissions
    Buffer.from([0, 0, 0, 0]), // metadata length
    bytecode,
    Buffer.from([0]), // fee_shard_index = 0 (should use hardcoded address)
  ]);

  // Derive script account
  const scriptKeyPair = Keypair.generate();

  const deployInstruction = new TransactionInstruction({
    programId: new PublicKey(FIVE_PROGRAM_ID),
    keys: [
      { pubkey: scriptKeyPair.publicKey, isSigner: false, isWritable: true },
      { pubkey: vmStatePda, isSigner: false, isWritable: true },
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: new PublicKey(HARDCODED_FEE_VAULT_0), isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: deployData,
  });

  // Create script account for storage
  const rent = await connection.getMinimumBalanceForRentExemption(100 + bytecode.length);

  const createScriptInstruction = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: scriptKeyPair.publicKey,
    lamports: rent,
    space: 100 + bytecode.length,
    programId: new PublicKey(FIVE_PROGRAM_ID),
  });

  const deployTx = new Transaction().add(createScriptInstruction).add(deployInstruction);
  deployTx.recentBlockhash = (await connection.getLatestBlockhash('confirmed')).blockhash;
  deployTx.feePayer = payer.publicKey;
  deployTx.partialSign(payer, scriptKeyPair);

  try {
    const sig = await connection.sendRawTransaction(deployTx.serialize(), {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
      maxRetries: 5,
    });
    await connection.confirmTransaction(sig, 'confirmed');
    console.log(`✓ Deploy succeeded with hardcoded fee vault verification!`);
    console.log(`   Signature: ${sig}\n`);
  } catch (e) {
    console.error(`✗ Deploy failed: ${e.message}`);
    if (e.logs) {
      console.error('   Program logs:', e.logs);
    }
    throw e;
  }

  console.log(`✅ Hardcoded fee vault verification test passed!\n`);
}

main().catch(err => {
  console.error('\n❌ Error:', err.message);
  process.exit(1);
});
