#!/usr/bin/env node

/**
 * Minimal CPI Test - Simplified
 * 
 * Tests basic Five VM execution without @init or PDA.
 * Uses direct Solana transactions for deployment.
 */

import {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    SystemProgram,
    LAMPORTS_PER_SOL,
} from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Configuration
const RPC_URL = 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = new PublicKey('HzC7dhS3gbcTPoLmwSGFcTSnAqdDpdtERP5n5r9wyY4k');
const VM_STATE_PDA = new PublicKey('BDSWCHg6aA5hVvjwts12B3uMWVV3q8UH3qU7FMUD7JX2');

async function main() {
    console.log('\n========================================');
    console.log('Minimal CPI Test');
    console.log('========================================\n');

    const connection = new Connection(RPC_URL, 'confirmed');

    // Load payer keypair
    const payerPath = path.join(process.env.HOME, '.config/solana/id.json');
    const payerKeypair = Keypair.fromSecretKey(
        Uint8Array.from(JSON.parse(fs.readFileSync(payerPath, 'utf-8')))
    );
    console.log(`Payer: ${payerKeypair.publicKey.toBase58()}`);

    // Create recipient
    const recipient = Keypair.generate();
    console.log(`Recipient: ${recipient.publicKey.toBase58()}`);

    // Step 1: Compile the CPI test script
    console.log('\n--- Step 1: Compile CPI Test Script ---');
    const sourcePath = path.join(__dirname, 'src/cpi_test.v');
    const buildDir = path.join(__dirname, 'build');

    if (!fs.existsSync(buildDir)) {
        fs.mkdirSync(buildDir, { recursive: true });
    }

    const compilerPath = path.join(__dirname, '../../target/debug/debug_compile');
    try {
        execSync(`${compilerPath} ${sourcePath}`, { cwd: path.dirname(sourcePath) });
        console.log('✓ Compilation successful');
    } catch (e) {
        console.error('✗ Compilation failed:', e.message);
        process.exit(1);
    }

    // Read compiled bytecode
    const binPath = sourcePath.replace('.v', '.bin');
    const bytecode = fs.readFileSync(binPath);
    console.log(`✓ Bytecode: ${bytecode.length} bytes`);

    // Step 2: Deploy the script using single-chunk deploy
    console.log('\n--- Step 2: Deploy Script ---');
    const scriptKeypair = Keypair.generate();
    console.log(`Script Account: ${scriptKeypair.publicKey.toBase58()}`);

    // Calculate script account size: header (64) + bytecode
    const SCRIPT_HEADER_SIZE = 64;
    const scriptAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
    const scriptRent = await connection.getMinimumBalanceForRentExemption(scriptAccountSize);

    console.log(`  Script account size: ${scriptAccountSize} bytes`);
    console.log(`  Rent: ${scriptRent / LAMPORTS_PER_SOL} SOL`);

    // Create the script account
    const createAccountTx = new Transaction().add(
        SystemProgram.createAccount({
            fromPubkey: payerKeypair.publicKey,
            newAccountPubkey: scriptKeypair.publicKey,
            lamports: scriptRent,
            space: scriptAccountSize,
            programId: FIVE_PROGRAM_ID,
        })
    );

    const createSig = await connection.sendTransaction(createAccountTx, [payerKeypair, scriptKeypair], {
        skipPreflight: false
    });
    await connection.confirmTransaction(createSig, 'confirmed');
    console.log(`✓ Script account created`);

    // Deploy bytecode using discriminator 8 (Deploy)
    // Format: [8, len_u32_le, permissions_u8, bytecode...]
    const deployData = Buffer.alloc(6 + bytecode.length);
    deployData[0] = 8; // Deploy discriminator
    deployData.writeUInt32LE(bytecode.length, 1);  // Bytecode length
    deployData[5] = 0;  // Permissions (none)
    bytecode.copy(deployData, 6);  // Bytecode

    const deployIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },  // accounts[0] = Script
            { pubkey: VM_STATE_PDA, isSigner: false, isWritable: true },            // accounts[1] = VM State
            { pubkey: payerKeypair.publicKey, isSigner: true, isWritable: true },   // accounts[2] = Owner/Payer
        ],
        data: deployData,
    });

    const deployTx = new Transaction().add(deployIx);
    try {
        const deploySig = await connection.sendTransaction(deployTx, [payerKeypair], {
            skipPreflight: false  // Enable preflight to see errors
        });
        await connection.confirmTransaction(deploySig, 'confirmed');
        console.log(`✓ Script deployed`);

        // Verify the deployment by checking script account data
        const scriptInfo = await connection.getAccountInfo(scriptKeypair.publicKey);
        if (scriptInfo) {
            const magic = scriptInfo.data.slice(0, 4);
            console.log(`  Header magic: ${magic[0]} ${magic[1]} ${magic[2]} ${magic[3]} (expected: 53 73 86 69 = "5IVE")`);
            console.log(`  First 20 bytes: ${Array.from(scriptInfo.data.slice(0, 20)).map(b => b.toString(16).padStart(2, '0')).join(' ')}`);
        }
    } catch (e) {
        console.error('✗ Deploy failed:', e.message);
        if (e.logs) {
            console.log('Deploy logs:');
            e.logs.forEach(msg => console.log(`  ${msg}`));
        }
    }

    // Step 3: Execute init_pda function
    console.log('\n--- Step 3: Execute init_pda ---');

    // Calculate PDA: seeds=["dummy"]
    const [pda, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from("dummy_v3")],
        FIVE_PROGRAM_ID
    );
    console.log(`PDA: ${pda.toBase58()} (bump: ${bump})`);

    // Function index 0 = init_pda
    // Params: none explicitly passed (seeds in @init are hardcoded in DSL)
    // Account args:
    // param0 = new_pda (PDA)
    // param1 = user (Payer)

    // Execute instruction data
    // Discriminator 9 = Execute
    const executeData = Buffer.from([
        9,  // Execute discriminator
        0,  // Function index 0 (VLE)
        0,  // Param count 0 (VLE)
    ]);

    const ixKeys = [
        { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },   // Script
        { pubkey: VM_STATE_PDA, isSigner: false, isWritable: false },             // VM State
        { pubkey: pda, isSigner: false, isWritable: true },                       // param0 = new_pda
        { pubkey: payerKeypair.publicKey, isSigner: true, isWritable: true },     // param1 = user (signer)
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },  // System Program
        { pubkey: payerKeypair.publicKey, isSigner: true, isWritable: true },     // Admin
    ];

    console.log('\nAccount Layout:');
    ixKeys.forEach((key, i) => {
        console.log(`  [${i}] ${key.pubkey.toBase58().slice(0, 8)}... signer=${key.isSigner} writable=${key.isWritable}`);
    });

    const executeIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: ixKeys,
        data: executeData,
    });

    const executeTx = new Transaction().add(executeIx);

    try {
        const executeSig = await connection.sendTransaction(executeTx, [payerKeypair], {
            skipPreflight: true,
            maxRetries: 3
        });
        await connection.confirmTransaction(executeSig, 'confirmed');

        const txDetails = await connection.getTransaction(executeSig, {
            maxSupportedTransactionVersion: 0
        });

        const success = txDetails?.meta?.err === null;
        console.log(`\n${success ? '✓' : '✗'} Transaction ${success ? 'succeeded' : 'failed'}!`);
        console.log(`Signature: ${executeSig}`);
        console.log(`Compute Units: ${txDetails?.meta?.computeUnitsConsumed || 'N/A'}`);

        if (txDetails?.meta?.logMessages) {
            console.log('\nProgram Logs:');
            txDetails.meta.logMessages.forEach(msg => console.log(`  ${msg}`));
        }
    } catch (e) {
        console.error('\n✗ Transaction failed:', e.message);
        if (e.logs) {
            console.log('\nProgram Logs:');
            e.logs.forEach(msg => console.log(`  ${msg}`));
        }
    }

    console.log('\n========================================');
    console.log('Test Complete');
    console.log('========================================\n');
}

main().catch(console.error);
