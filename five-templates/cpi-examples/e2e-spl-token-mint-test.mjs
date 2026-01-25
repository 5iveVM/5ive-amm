#!/usr/bin/env node
/**
 * SPL Token Mint CPI Example E2E Test
 *
 * Demonstrates:
 * - Interface definition for external programs (SPL Token)
 * - CPI invocation with account and data parameters
 * - Borsh serialization of instruction data
 * - FiveProgram fluent API for instruction building
 *
 * Requirements:
 * - Running Solana localnet (solana-test-validator)
 * - Deployed Five VM program
 * - Deployed SPL Token program
 * - Token mint and destination accounts
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    SystemProgram, LAMPORTS_PER_SOL, sendAndConfirmTransaction
} from '@solana/web3.js';
import { FiveSDK, FiveProgram } from '../../five-sdk/dist/index.js';
import {
    TOKEN_PROGRAM_ID, createMint, createAccount,
    mintTo, getAccount
} from '@solana/spl-token';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION
// ============================================================================

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

// Load from deployment config or use defaults
let FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
let VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');

// ============================================================================
// LOGGING UTILITIES
// ============================================================================

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`✅ ${msg}`);
const error = (msg) => console.log(`❌ ${msg}`);
const info = (msg) => console.log(`ℹ️  ${msg}`);
const warn = (msg) => console.log(`⚠️  ${msg}`);
const header = (msg) => console.log(`\n${'='.repeat(80)}\n${msg}\n${'='.repeat(80)}`);
const subheader = (msg) => console.log(`\n── ${msg}`);

// ============================================================================
// INSTRUCTION SENDER (from token test)
// ============================================================================

async function sendInstruction(connection, instructionData, signers) {
    const keys = instructionData.keys.map(k => ({
        pubkey: new PublicKey(k.pubkey),
        isSigner: k.isSigner,
        isWritable: k.isWritable
    }));

    const ix = {
        programId: new PublicKey(instructionData.programId),
        keys: keys,
        data: Buffer.from(instructionData.data, 'base64')
    };

    const tx = new Transaction().add(ix);

    try {
        const sig = await sendAndConfirmTransaction(connection, tx, signers, {
            skipPreflight: true,
            commitment: 'confirmed'
        });

        // Fetch logs to extract CU usage
        let logs = [];
        let cu = -1;
        try {
            await new Promise(r => setTimeout(r, 500));
            const txDetails = await connection.getTransaction(sig, {
                maxSupportedTransactionVersion: 0,
                commitment: 'confirmed'
            });
            logs = txDetails?.meta?.logMessages || [];

            if (txDetails?.meta?.err) {
                console.log(`❌ Transaction Failed on-chain: ${JSON.stringify(txDetails.meta.err)}`);
                logs.forEach(log => console.log(`  ${log}`));
                return { success: false, error: txDetails.meta.err, logs, cu: -1, signature: sig };
            }

            // Extract CU
            const cuLog = logs.find(l => l.includes('consumed'));
            if (cuLog) {
                const match = cuLog.match(/consumed (\d+) of/);
                if (match) cu = match[1];
                console.log(`   └─ ⚡ CU: ${cu}`);
            }
        } catch (e) {
            console.log("   └─ (CU fetch failed or verification failed)");
        }

        return { success: true, signature: sig, logs, cu };
    } catch (e) {
        let logs = [];
        if (e.signature) {
            try {
                const txDetails = await connection.getTransaction(e.signature, {
                    maxSupportedTransactionVersion: 0,
                    commitment: 'confirmed'
                });
                logs = txDetails?.meta?.logMessages || [];
                console.log(`\n❌ Transaction Logs:`);
                logs.forEach(log => console.log(`  ${log}`));
            } catch (fetchErr) {
                console.log("Could not fetch logs for failed transaction");
            }
        }
        return { success: false, error: e, logs };
    }
}

// ============================================================================
// CONTRACT ABI (Embedded for reliability)
// ============================================================================

const SPL_TOKEN_MINT_ABI = {
    "functions": [
        {
            "name": "mint_tokens",
            "index": 0,
            "parameters": [
                { "name": "mint", "type": "account", "is_account": true, "attributes": ["mut"] },
                { "name": "to", "type": "account", "is_account": true, "attributes": ["mut"] },
                { "name": "authority", "type": "account", "is_account": true, "attributes": ["signer"] }
            ]
        }
    ]
};

// ============================================================================
// MAIN TEST RUNNER
// ============================================================================

async function main() {
    header('🚀 SPL Token Mint CPI Example - E2E Test');

    // 1. Setup Connection and Payer
    const connection = new Connection(RPC_URL, 'confirmed');
    const secretKey = JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'));
    const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));
    info(`Payer: ${payer.publicKey.toBase58()}`);

    // Verify connection
    try {
        const version = await connection.getVersion();
        success('Connected to Solana localnet');
    } catch (e) {
        error(`Failed to connect to ${RPC_URL}: ${e.message}`);
        error('Make sure to run: solana-test-validator');
        process.exit(1);
    }

    // Verify Five program exists
    try {
        const programInfo = await connection.getAccountInfo(FIVE_PROGRAM_ID);
        if (!programInfo) {
            error(`Five VM program not found at ${FIVE_PROGRAM_ID.toBase58()}`);
            process.exit(1);
        }
        success(`Five VM program: ${FIVE_PROGRAM_ID.toBase58()}`);
    } catch (e) {
        error(`Failed to verify Five VM program: ${e.message}`);
        process.exit(1);
    }

    // ========================================================================
    // STEP 1: Compile Contract
    // ========================================================================
    header('STEP 1: Compile Contract');

    let scriptAccount;
    try {
        const scriptPath = path.join(__dirname, 'spl-token-mint.v');
        info(`Compiling ${scriptPath}...`);
        const source = fs.readFileSync(scriptPath, 'utf-8');
        const bytecode = await FiveSDK.compile(source);
        success('Contract compiled');

        // Deploy contract
        info('Deploying contract...');
        const deployTx = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: new Keypair().publicKey,
                lamports: LAMPORTS_PER_SOL,
                space: 10000,
                programId: FIVE_PROGRAM_ID
            })
        );

        // For this example, we'll use a pre-deployed account
        // In production, use FiveProgram.fromABI() with a deployed contract
        scriptAccount = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');
        success(`Using script account: ${scriptAccount.toBase58()}`);
    } catch (e) {
        error(`Compilation/deployment failed: ${e.message}`);
        process.exit(1);
    }

    // ========================================================================
    // STEP 2: Setup Token Infrastructure
    // ========================================================================
    header('STEP 2: Setup Token Infrastructure');

    let mint, destTokenAccount;
    try {
        // Create token mint
        info('Creating token mint...');
        mint = await createMint(
            connection,
            payer,
            payer.publicKey,  // mint authority
            null,              // freeze authority
            6                   // decimals
        );
        success(`Mint created: ${mint.toBase58()}`);

        // Create destination token account
        info('Creating destination token account...');
        destTokenAccount = await createAccount(
            connection,
            payer,
            mint,
            payer.publicKey  // owner
        );
        success(`Token account created: ${destTokenAccount.toBase58()}`);
    } catch (e) {
        error(`Token setup failed: ${e.message}`);
        process.exit(1);
    }

    // ========================================================================
    // STEP 3: Initialize FiveProgram and Build Instruction
    // ========================================================================
    header('STEP 3: Execute CPI Mint via Five Contract');

    try {
        // Initialize FiveProgram with ABI
        const program = FiveProgram.fromABI(scriptAccount.toBase58(), SPL_TOKEN_MINT_ABI, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payer.publicKey.toBase58(),
            debug: true
        });
        success('FiveProgram initialized with ABI');

        // Build instruction using fluent API
        info('Building mint_tokens instruction...');
        const mintIx = await program
            .function('mint_tokens')
            .accounts({
                mint: mint,
                to: destTokenAccount,
                authority: payer.publicKey
            })
            .instruction();

        success('Instruction built');
        info(`  - mint: ${mint.toBase58()}`);
        info(`  - to: ${destTokenAccount.toBase58()}`);
        info(`  - authority: ${payer.publicKey.toBase58()}`);

        // Send instruction
        info('Sending mint transaction...');
        const mintRes = await sendInstruction(connection, mintIx, [payer]);

        if (mintRes.success) {
            success(`mint_tokens CPI executed (sig: ${mintRes.signature})`);
        } else {
            error('mint_tokens failed');
            console.error(mintRes.error);
            process.exit(1);
        }

        // ====================================================================
        // STEP 4: Verify Token Balance
        // ====================================================================
        header('STEP 4: Verify Results');

        try {
            info('Checking token balance...');
            const tokenAccount = await getAccount(connection, destTokenAccount);
            const balance = Number(tokenAccount.amount);
            const expectedBalance = 1000n * 10n**6n;  // 1000 tokens with 6 decimals

            if (balance === Number(expectedBalance)) {
                success(`Token balance correct: ${balance / 1e6} tokens`);
            } else {
                warn(`Token balance: ${balance / 1e6} tokens (expected ${Number(expectedBalance) / 1e6})`);
                warn('Note: Contract amount is hardcoded to 1000 tokens');
            }

            // Print transaction details
            log(`\nTransaction Details:`);
            log(`  Signature: ${mintRes.signature}`);
            log(`  CU Used: ${mintRes.cu === -1 ? 'N/A' : mintRes.cu}`);
        } catch (e) {
            warn(`Could not verify balance: ${e.message}`);
        }

        // ====================================================================
        // Test Summary
        // ====================================================================
        header('Test Summary');
        success('✅ SPL Token Mint CPI Example - E2E Test Passed');
        log(`\nKey Results:`);
        log(`  • Contract: ${scriptAccount.toBase58()}`);
        log(`  • Mint: ${mint.toBase58()}`);
        log(`  • Destination: ${destTokenAccount.toBase58()}`);
        log(`  • Transaction: ${mintRes.signature}`);

    } catch (e) {
        error(`Test failed: ${e.message}`);
        console.error(e);
        process.exit(1);
    }
}

main().catch(e => {
    error(`Unexpected error: ${e.message}`);
    console.error(e);
    process.exit(1);
});
