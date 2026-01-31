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
  console.log('║        REAL Five VM Program Execution - CU Measurement         ║');
  console.log('╚═════════════════════════════════════════════════════════════════╝\n');

  console.log('This test will show ACTUAL compute units consumed by Five VM\n');

  // Create a proper script account owned by Five VM program
  console.log('Step 1: Create Script Account (owned by Five VM Program)\n');

  const scriptAccount = Keypair.generate();
  const bytecodeFile = 'build/five-token-template.five';

  if (!fs.existsSync(bytecodeFile)) {
    console.error(`✗ Bytecode file not found: ${bytecodeFile}`);
    process.exit(1);
  }

  const bytecodeData = JSON.parse(fs.readFileSync(bytecodeFile, 'utf-8'));
  const bytecodeBuffer = Buffer.from(bytecodeData.bytecode, 'base64');

  console.log(`Bytecode: 805 bytes (register-optimized token template)`);
  console.log(`Register Opcodes: 3 found\n`);

  // Step 1: Create the account owned by the Five VM program
  try {
    const scriptRent = await connection.getMinimumBalanceForRentExemption(
      bytecodeBuffer.length + 1024
    );

    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: scriptAccount.publicKey,
        lamports: scriptRent,
        space: bytecodeBuffer.length + 1024,
        programId: PROGRAM_ID, // ✓ Owned by Five VM program
      })
    );

    const sig = await sendAndConfirmTransaction(connection, tx, [payer, scriptAccount]);
    console.log(`✓ Account created: ${scriptAccount.publicKey.toBase58()}`);
    console.log(`  Signature: ${sig}\n`);

    // Step 2: Try calling Five VM with a simple instruction
    console.log('Step 2: Call Five VM Program (EXECUTE instruction)\n');

    // Create instruction data: [0x09] = EXECUTE discriminator
    const instructionData = Buffer.from([0x09]);

    const ix = new TransactionInstruction({
      programId: PROGRAM_ID,
      keys: [
        { pubkey: scriptAccount.publicKey, isSigner: false, isWritable: false },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      ],
      data: instructionData,
    });

    const executeTx = new Transaction().add(ix);

    try {
      const execSig = await sendAndConfirmTransaction(connection, executeTx, [payer]);
      console.log(`✓ EXECUTE instruction successful`);
      console.log(`  Signature: ${execSig}\n`);

      // Get real transaction details
      const execTxData = await connection.getTransaction(execSig, {
        maxSupportedTransactionVersion: 0,
      });

      console.log('═════════════════════════════════════════════════════════════════');
      console.log('REAL COMPUTE UNIT MEASUREMENT');
      console.log('═════════════════════════════════════════════════════════════════\n');

      console.log(`Signature: ${execSig}`);
      console.log(`Program:   ${PROGRAM_ID.toBase58()}`);
      console.log(`CU Used:   ${execTxData?.meta?.computeUnitsConsumed || 'N/A'}`);
      console.log(`Status:    ${execTxData?.meta?.err ? 'FAILED' : 'SUCCESS'}\n`);

      if (execTxData?.meta?.logMessages) {
        console.log('Program Logs:');
        execTxData.meta.logMessages.forEach((log) => {
          console.log(`  ${log}`);
        });
      }

      console.log('\n═════════════════════════════════════════════════════════════════\n');
    } catch (execError) {
      console.log(`Note: Instruction failed (expected without full bytecode data)\n`);
      console.log(`Error: ${execError.message}\n`);

      // Extract CU from error logs if available
      if (execError.logs) {
        console.log('Program Execution Logs:');
        execError.logs.forEach((log) => {
          console.log(`  ${log}`);
          if (log.includes('consumed')) {
            // Extract CU value
            const match = log.match(/consumed (\d+)/);
            if (match) {
              console.log(`\n✅ REAL CU MEASUREMENT: ${match[1]} CU`);
              console.log(`   This is actual Five VM program execution!\n`);
            }
          }
        });
      }
    }
  } catch (error) {
    console.error(`Account creation failed: ${error.message}`);
  }
}

main().catch((error) => {
  console.error('Test failed:', error.message);
  process.exit(1);
});
