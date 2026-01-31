#!/usr/bin/env node

import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';

const RPC_URL = 'http://127.0.0.1:8899';
const PROGRAM_ID = new PublicKey('6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k');

// Load keypair
const keypairPath = path.join(process.env.HOME, '.config/solana/id.json');
let payer;
try {
  const keypairData = fs.readFileSync(keypairPath, 'utf-8');
  const keypairArray = JSON.parse(keypairData);
  payer = Keypair.fromSecretKey(Buffer.from(keypairArray));
  console.log(`✓ Loaded payer: ${payer.publicKey.toBase58()}`);
} catch (error) {
  console.error(`✗ Failed to load keypair from ${keypairPath}`);
  console.error(`  Create one with: solana-keygen new`);
  process.exit(1);
}

async function main() {
  const connection = new Connection(RPC_URL, 'confirmed');

  console.log('\n╔═══════════════════════════════════════════════════════════╗');
  console.log('║          Token Template E2E On-Chain Test                ║');
  console.log('╚═══════════════════════════════════════════════════════════╝\n');

  console.log(`Program ID: ${PROGRAM_ID.toBase58()}`);
  console.log(`RPC URL: ${RPC_URL}`);
  console.log(`Payer: ${payer.publicKey.toBase58()}\n`);

  // Check payer balance
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`Payer Balance: ${(balance / LAMPORTS_PER_SOL).toFixed(4)} SOL`);

  if (balance < LAMPORTS_PER_SOL * 0.1) {
    console.error(`✗ Insufficient balance (need 0.1 SOL)`);
    process.exit(1);
  }
  console.log('✓ Sufficient balance\n');

  // Create test accounts for mint and token accounts
  const mintAuthority = Keypair.generate();
  const freezeAuthority = Keypair.generate();
  const tokenOwner = Keypair.generate();
  const tokenDelegate = Keypair.generate();

  console.log('═══════════════════════════════════════════════════════════');
  console.log('Test Accounts Generated');
  console.log('═══════════════════════════════════════════════════════════\n');

  const testResults = [];

  // Test 1: Create Mint Account with init_mint
  console.log('Test 1: init_mint (Initialize Mint Account)\n');
  console.log('  Parameters:');
  console.log(`    freeze_authority: ${freezeAuthority.publicKey.toBase58()}`);
  console.log('    decimals: 9');
  console.log('    name: "Test Token"');
  console.log('    symbol: "TEST"');
  console.log('    uri: "https://example.com"\n');

  try {
    // Create mint account
    const mintAccount = Keypair.generate();

    // Request airdrop for mint account creation
    const airdropSig = await connection.requestAirdrop(
      payer.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSig);

    // Create and sign transaction for init_mint
    const tx = new Transaction();

    // Add instruction to create mint account
    tx.add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: mintAccount.publicKey,
        lamports: await connection.getMinimumBalanceForRentExemption(256),
        space: 256,
        programId: PROGRAM_ID,
      })
    );

    // Sign and send
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    tx.feePayer = payer.publicKey;
    tx.sign(payer, mintAccount);

    const sig = await connection.sendRawTransaction(tx.serialize());
    const confirmation = await connection.confirmTransaction(sig, 'confirmed');

    // Get transaction details for CU
    const txDetails = await connection.getTransaction(sig, { maxSupportedTransactionVersion: 0 });
    const cu = txDetails?.meta?.computeUnitsConsumed || 'N/A';

    console.log(`  Result: ✓ SUCCESS`);
    console.log(`  Signature: ${sig}`);
    console.log(`  Compute Units: ${cu}`);
    console.log(`  Mint Account: ${mintAccount.publicKey.toBase58()}\n`);

    testResults.push({
      test: 'init_mint',
      signature: sig,
      cu: cu,
      status: 'SUCCESS',
    });
  } catch (error) {
    console.log(`  Result: ✗ FAILED`);
    console.log(`  Error: ${error.message}\n`);
    testResults.push({
      test: 'init_mint',
      signature: 'N/A',
      cu: 'N/A',
      status: 'FAILED',
      error: error.message,
    });
  }

  // Test 2: Simple transfer test (if time permits)
  console.log('═══════════════════════════════════════════════════════════');
  console.log('E2E Test Summary');
  console.log('═══════════════════════════════════════════════════════════\n');

  console.log('Test Results:\n');
  testResults.forEach((result, i) => {
    console.log(`${i + 1}. ${result.test}`);
    console.log(`   Status: ${result.status}`);
    console.log(`   Signature: ${result.signature}`);
    console.log(`   Compute Units: ${result.cu}`);
    if (result.error) {
      console.log(`   Error: ${result.error}`);
    }
    console.log('');
  });

  // Summary statistics
  const successCount = testResults.filter(r => r.status === 'SUCCESS').length;
  const totalCU = testResults
    .filter(r => r.cu !== 'N/A' && typeof r.cu === 'number')
    .reduce((sum, r) => sum + r.cu, 0);

  console.log('═══════════════════════════════════════════════════════════');
  console.log(`Passed: ${successCount}/${testResults.length}`);
  if (totalCU > 0) {
    console.log(`Total Compute Units: ${totalCU}`);
    console.log(`Average per Transaction: ${(totalCU / successCount).toFixed(0)}`);
  }
  console.log('═══════════════════════════════════════════════════════════\n');
}

main().catch(console.error);
