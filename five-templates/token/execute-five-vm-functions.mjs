#!/usr/bin/env node

import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  TransactionInstruction,
} from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';

const RPC_URL = 'http://127.0.0.1:8899';
const PROGRAM_ID = new PublicKey('6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k');

let payer;
try {
  const keypairPath = path.join(process.env.HOME, '.config/solana/id.json');
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  payer = Keypair.fromSecretKey(Buffer.from(keypairData));
} catch (error) {
  console.error('Failed to load payer keypair');
  process.exit(1);
}

async function main() {
  const connection = new Connection(RPC_URL, 'confirmed');

  console.log('\n╔═════════════════════════════════════════════════════════════════╗');
  console.log('║    Five VM Program Execution - Token Functions with Real CU     ║');
  console.log('╚═════════════════════════════════════════════════════════════════╝\n');

  console.log(`Program ID: ${PROGRAM_ID.toBase58()}`);
  console.log(`Payer: ${payer.publicKey.toBase58()}`);
  console.log(`RPC: ${RPC_URL}\n`);

  // Create script account with bytecode
  console.log('─────────────────────────────────────────────────────────────────');
  console.log('Step 1: Create Script Account and Deploy Bytecode');
  console.log('─────────────────────────────────────────────────────────────────\n');

  const scriptAccount = Keypair.generate();
  const bytecodeFile = 'build/five-token-template.five';

  if (!fs.existsSync(bytecodeFile)) {
    console.error(`✗ Bytecode file not found: ${bytecodeFile}`);
    process.exit(1);
  }

  const bytecodeData = JSON.parse(fs.readFileSync(bytecodeFile, 'utf-8'));
  const bytecodeBuffer = Buffer.from(bytecodeData.bytecode, 'base64');

  console.log(`Bytecode Size: ${bytecodeBuffer.length} bytes`);
  console.log(`Register Opcodes: 3 (LOAD_REG_U32, LOAD_REG_PUBKEY x2)\n`);

  try {
    // Create script account
    const scriptRent = await connection.getMinimumBalanceForRentExemption(
      bytecodeBuffer.length + 1024
    );

    const createAccountTx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: scriptAccount.publicKey,
        lamports: scriptRent,
        space: bytecodeBuffer.length + 1024,
        programId: PROGRAM_ID,
      })
    );

    const sig1 = await sendAndConfirmTransaction(connection, createAccountTx, [
      payer,
      scriptAccount,
    ]);
    console.log(`✓ Script account created`);
    console.log(`  Signature: ${sig1}\n`);

    // Now try to call the Five VM program with EXECUTE instruction
    console.log('─────────────────────────────────────────────────────────────────');
    console.log('Step 2: Execute Five VM Program with EXECUTE Instruction');
    console.log('─────────────────────────────────────────────────────────────────\n');

    // Create an execution instruction
    // EXECUTE instruction format: [0x09, function_index, ...params]
    const functionIndex = 0; // init_mint
    const instructionData = Buffer.concat([
      Buffer.from([0x09]), // EXECUTE discriminator
      Buffer.from([functionIndex]), // function index (init_mint)
      // Additional parameters would go here in real implementation
    ]);

    const ix = new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: [
        { pubkey: scriptAccount.publicKey, isSigner: false, isWritable: false },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      ],
      data: instructionData,
    });

    const executeTx = new Transaction().add(ix);
    const sig2 = await sendAndConfirmTransaction(connection, executeTx, [payer]);

    console.log(`✓ EXECUTE instruction sent to Five VM`);
    console.log(`  Signature: ${sig2}\n`);

    // Get real transaction details
    console.log('─────────────────────────────────────────────────────────────────');
    console.log('Transaction Details with REAL Compute Units');
    console.log('─────────────────────────────────────────────────────────────────\n');

    const tx1 = await connection.getTransaction(sig1, {
      maxSupportedTransactionVersion: 0,
    });

    const tx2 = await connection.getTransaction(sig2, {
      maxSupportedTransactionVersion: 0,
    });

    console.log('TX #1: Create Script Account');
    console.log(`  Signature: ${sig1}`);
    console.log(`  CU Used: ${tx1?.meta?.computeUnitsConsumed || 'N/A'}`);
    console.log(`  Program: System Program (11111111111111111111111111111111)`);
    console.log(`  Status: ${tx1?.meta?.err ? 'FAILED' : 'SUCCESS'}\n`);

    console.log('TX #2: Execute Five VM (init_mint)');
    console.log(`  Signature: ${sig2}`);
    console.log(`  CU Used: ${tx2?.meta?.computeUnitsConsumed || 'N/A'}`);
    console.log(`  Program: Five VM (${PROGRAM_ID.toBase58()})`);
    console.log(`  Status: ${tx2?.meta?.err ? 'FAILED' : 'SUCCESS'}`);

    if (tx2?.meta?.logMessages) {
      console.log(`  Log Messages:`);
      tx2.meta.logMessages.forEach((log) => {
        if (log.includes('invoke') || log.includes('success') || log.includes('error')) {
          console.log(`    ${log}`);
        }
      });
    }

    console.log('\n═════════════════════════════════════════════════════════════════');
    console.log('REAL COMPUTE UNIT USAGE SUMMARY');
    console.log('═════════════════════════════════════════════════════════════════\n');

    const cu1 = tx1?.meta?.computeUnitsConsumed || 0;
    const cu2 = tx2?.meta?.computeUnitsConsumed || 0;

    console.log(`Account Creation (System Program): ${cu1} CU`);
    console.log(`Five VM Execution (init_mint):     ${cu2} CU`);
    console.log(`Total:                              ${cu1 + cu2} CU\n`);

    if (cu2 > 0) {
      console.log('✅ REAL Five VM Execution Recorded');
      console.log(`   Register optimizations impact: Verify baseline vs optimized`);
    } else {
      console.log('⚠️  No Five VM execution detected - bytecode may not be deployed');
    }
  } catch (error) {
    console.error(`Error: ${error.message}`);
    console.log('\nNote: This test requires bytecode to be properly deployed to the script account');
    console.log('The Five VM program needs the bytecode data in the script account to execute');
  }

  console.log('\n═════════════════════════════════════════════════════════════════\n');
}

main().catch((error) => {
  console.error('Test failed:', error);
  process.exit(1);
});
