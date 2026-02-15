#!/usr/bin/env node

/**
 * Deploy token template to localnet using hardcoded VM state
 */

import fs from 'fs';
import path from 'path';
import { readFile } from 'node:fs/promises';
import web3 from '../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';
const { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, ComputeBudgetProgram } = web3;
import os from 'node:os';

const RPC_URL = 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = '3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1';
const VM_STATE_PDA = 'AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit';

async function main() {
  const connection = new Connection(RPC_URL, 'confirmed');

  // Load payer keypair
  const keypairPath = path.join(os.homedir(), '.config/solana/id.json');
  const keypairData = JSON.parse(await readFile(keypairPath, 'utf-8'));
  const payer = Keypair.fromSecretKey(Uint8Array.from(keypairData));

  console.log(`\n📋 Deploying Token Template to Localnet`);
  console.log(`   RPC: ${RPC_URL}`);
  console.log(`   Program ID: ${FIVE_PROGRAM_ID}`);
  console.log(`   Payer: ${payer.publicKey.toBase58()}`);
  console.log(`   VM State: ${VM_STATE_PDA}\n`);

  // Load token bytecode
  const bytecodeFile = path.join(process.cwd(), 'five-templates/token/build/five-token-baseline.five');
  if (!fs.existsSync(bytecodeFile)) {
    throw new Error(`Bytecode file not found: ${bytecodeFile}`);
  }

  const fiveFileContent = fs.readFileSync(bytecodeFile, 'utf-8');
  const fiveFile = JSON.parse(fiveFileContent);
  const bytecode = Buffer.from(fiveFile.bytecode, 'base64');

  console.log(`✓ Loaded bytecode: ${bytecode.length} bytes\n`);

  // Create script account keypair
  const scriptKeyPair = Keypair.generate();
  const scriptPubkey = scriptKeyPair.publicKey;

  // Calculate required space
  const requiredSpace = 64 + bytecode.length; // Header + bytecode
  const rent = await connection.getMinimumBalanceForRentExemption(requiredSpace);

  console.log(`⏳ Deploying script...`);
  console.log(`   Script Account: ${scriptPubkey.toBase58()}`);
  console.log(`   Required Space: ${requiredSpace} bytes`);
  console.log(`   Rent: ${rent / 1e9} SOL\n`);

  // Create deploy instruction
  // Format: [8] discriminator + [4] bytecode_len + [1] permissions + [4] metadata_len + metadata + bytecode + [1] fee_shard_index
  const deployData = Buffer.concat([
    Buffer.from([8]), // DEPLOY_INSTRUCTION
    Buffer.from([bytecode.length, 0, 0, 0]), // bytecode length (u32 LE)
    Buffer.from([0]), // permissions
    Buffer.from([0, 0, 0, 0]), // metadata length
    bytecode,
    Buffer.from([0]), // fee_shard_index = 0
  ]);

  // Create account
  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: scriptPubkey,
    lamports: rent,
    space: requiredSpace,
    programId: new PublicKey(FIVE_PROGRAM_ID),
  });

  // Deploy
  const deployIx = new TransactionInstruction({
    programId: new PublicKey(FIVE_PROGRAM_ID),
    keys: [
      { pubkey: scriptPubkey, isSigner: false, isWritable: true },
      { pubkey: new PublicKey(VM_STATE_PDA), isSigner: false, isWritable: true },
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      // Fee vault shard 0
      { pubkey: new PublicKey('HXW6bZsdJW6Be5c51NNpNb9NcVxmHbUrF9oKkt4C1tEH'), isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: deployData,
  });

  // Build transaction
  const tx = new Transaction().add(createAccountIx).add(deployIx);
  const { blockhash } = await connection.getLatestBlockhash('confirmed');
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  tx.partialSign(payer, scriptKeyPair);

  try {
    const sig = await connection.sendRawTransaction(tx.serialize(), {
      skipPreflight: false,
      preflightCommitment: 'confirmed',
      maxRetries: 5,
    });

    console.log(`⏳ Confirming transaction...`);
    await connection.confirmTransaction(sig, 'confirmed');

    console.log(`✓ Token template deployed successfully!`);
    console.log(`   Script Account: ${scriptPubkey.toBase58()}`);
    console.log(`   Signature: ${sig}\n`);

    // Update deployment config
    const configPath = path.join(process.cwd(), 'five-templates/token/deployment-config.json');
    const config = {
      rpcUrl: RPC_URL,
      fiveProgramId: FIVE_PROGRAM_ID,
      vmStatePda: VM_STATE_PDA,
      tokenScriptAccount: scriptPubkey.toBase58(),
      timestamp: new Date().toISOString(),
    };
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
    console.log(`✓ Updated deployment-config.json\n`);

    return {
      scriptAccount: scriptPubkey.toBase58(),
      signature: sig,
    };
  } catch (e) {
    console.error(`✗ Deployment failed: ${e.message}`);
    if (e.logs) {
      console.error('   Program logs:', e.logs);
    }
    throw e;
  }
}

main().catch(err => {
  console.error('\n❌ Error:', err.message);
  process.exit(1);
});
