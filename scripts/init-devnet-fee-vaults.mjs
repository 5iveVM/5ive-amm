#!/usr/bin/env node

/**
 * Initialize fee vault shards on devnet
 * This must be run before deploying scripts to ensure fee collection accounts exist.
 */

import { readFile } from 'node:fs/promises';
import web3 from '../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';
const { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } = web3;
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';

const DEVNET_RPC = 'https://api.devnet.solana.com';
const FIVE_PROGRAM_ID = '4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d';
const VM_STATE_ACCOUNT = '8ip3qGGETf8774jo6kXbsTTrMm5V9bLuGC4znmyZjT3z';

// Fee vault seed matches Rust: b"\xFFfive_vm_fee_vault_v1"
const FEE_VAULT_SEED = Buffer.from([
  0xff, 0x66, 0x69, 0x76, 0x65, 0x5f, 0x76, 0x6d, 0x5f, 0x66, 0x65, 0x65,
  0x5f, 0x76, 0x61, 0x75, 0x6c, 0x74, 0x5f, 0x76, 0x31,
]);

const DEFAULT_FEE_VAULT_SHARD_COUNT = 10;
const INIT_FEE_VAULT_INSTRUCTION = 11;

async function deriveFeeVault(shardIndex) {
  const [pda, bump] = PublicKey.findProgramAddressSync(
    [FEE_VAULT_SEED, Buffer.from([shardIndex])],
    new PublicKey(FIVE_PROGRAM_ID)
  );
  return { address: pda.toBase58(), bump };
}

async function main() {
  const connection = new Connection(DEVNET_RPC, 'confirmed');

  // Load payer keypair
  const keypairPath = path.join(os.homedir(), '.config/solana/id.json');
  if (!fs.existsSync(keypairPath)) {
    throw new Error(`Keypair not found at ${keypairPath}`);
  }

  const keypairData = JSON.parse(await readFile(keypairPath, 'utf-8'));
  const payer = Keypair.fromSecretKey(Uint8Array.from(keypairData));

  console.log(`\n📋 Initializing Fee Vault Shards on Devnet`);
  console.log(`   Payer: ${payer.publicKey.toBase58()}`);
  console.log(`   Program ID: ${FIVE_PROGRAM_ID}`);
  console.log(`   VM State: ${VM_STATE_ACCOUNT}\n`);

  // Check VM state account exists
  const vmStateInfo = await connection.getAccountInfo(new PublicKey(VM_STATE_ACCOUNT));
  if (!vmStateInfo) {
    throw new Error(`VM State account ${VM_STATE_ACCOUNT} not found on devnet`);
  }
  console.log(`✓ VM State account found (${vmStateInfo.data.length} bytes)`);

  // Get current balance
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`✓ Payer balance: ${balance / 1e9} SOL\n`);

  if (balance < 1e9) {
    throw new Error('Insufficient SOL balance. Need at least 1 SOL for initialization.');
  }

  // Initialize fee vault shards
  const txSignatures = [];
  for (let shardIndex = 0; shardIndex < DEFAULT_FEE_VAULT_SHARD_COUNT; shardIndex++) {
    const vault = await deriveFeeVault(shardIndex);

    // Check if vault already exists
    const vaultInfo = await connection.getAccountInfo(new PublicKey(vault.address));
    if (vaultInfo) {
      console.log(`✓ Shard ${shardIndex}: Already initialized (${vault.address})`);
      continue;
    }

    // Create init fee vault instruction
    const data = Buffer.from([INIT_FEE_VAULT_INSTRUCTION, shardIndex & 0xff, vault.bump & 0xff]);

    const instruction = new TransactionInstruction({
      programId: new PublicKey(FIVE_PROGRAM_ID),
      keys: [
        { pubkey: new PublicKey(VM_STATE_ACCOUNT), isSigner: false, isWritable: false },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: new PublicKey(vault.address), isSigner: false, isWritable: true },
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
      console.log(`⏳ Shard ${shardIndex}: Sending initialization...`);
      const sig = await connection.sendRawTransaction(tx.serialize(), {
        skipPreflight: true,
        preflightCommitment: 'confirmed',
        maxRetries: 5,
      });

      // Wait for confirmation
      await connection.confirmTransaction(sig, 'confirmed');
      txSignatures.push(sig);
      console.log(`✓ Shard ${shardIndex}: Initialized (${vault.address})`);
      console.log(`   Signature: ${sig}\n`);
    } catch (e) {
      console.error(`✗ Shard ${shardIndex}: Failed - ${e.message}`);
      if (e.logs) {
        console.error('   Program logs:', e.logs);
      }
      throw e;
    }
  }

  console.log(`\n✅ Fee vault initialization complete!`);
  console.log(`   Total shards initialized: ${txSignatures.length}`);
  console.log(`   Ready for script deployment on devnet\n`);

  // Print shard addresses for reference
  console.log('Fee Vault Shard Addresses:');
  for (let i = 0; i < DEFAULT_FEE_VAULT_SHARD_COUNT; i++) {
    const vault = await deriveFeeVault(i);
    console.log(`   Shard ${i}: ${vault.address}`);
  }
}

main().catch(err => {
  console.error('\n❌ Error:', err.message);
  process.exit(1);
});
