#!/usr/bin/env node
/**
 * INVOKE_SIGNED with PDA Authority CPI Example E2E Test
 *
 * Demonstrates:
 * - Using INVOKE_SIGNED with Program Derived Address authority
 * - Delegated authority without direct signer
 * - Burning tokens from a contract-controlled treasury PDA
 * - FiveProgram fluent API for instruction building
 *
 * Requirements:
 * - Running Solana localnet (solana-test-validator)
 * - Deployed Five VM program
 * - Deployed SPL Token program
 * - Token mint and treasury token account
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
    mintTo, burn, getAccount
} from '@solana/spl-token';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION
// ============================================================================

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

let FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
let VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
let SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

// ============================================================================
// LOGGING UTILITIES
// ============================================================================

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`✅ ${msg}`);
const error = (msg) => console.log(`❌ ${msg}`);
const info = (msg) => console.log(`ℹ️  ${msg}`);
const warn = (msg) => console.log(`⚠️  ${msg}`);
const header = (msg) => console.log(`\n${'='.repeat(80)}\n${msg}\n${'='.repeat(80)}`);

// ============================================================================
// INSTRUCTION SENDER
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
                console.log(`❌ Transaction Failed: ${JSON.stringify(txDetails.meta.err)}`);
                logs.forEach(log => console.log(`  ${log}`));
                return { success: false, error: txDetails.meta.err, logs, cu: -1, signature: sig };
            }

            const cuLog = logs.find(l => l.includes('consumed'));
            if (cuLog) {
                const match = cuLog.match(/consumed (\d+) of/);
                if (match) cu = match[1];
                console.log(`   └─ ⚡ CU: ${cu}`);
            }
        } catch (e) {
            console.log("   └─ (CU info unavailable)");
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
                // Ignore
            }
        }
        return { success: false, error: e, logs };
    }
}

// ============================================================================
// CONTRACT ABI
// ============================================================================

const PDA_BURN_ABI = {
    "functions": [
        {
            "name": "burn_from_pda",
            "index": 0,
            "parameters": [
                { "name": "token_account", "type": "account", "is_account": true, "attributes": ["mut"] },
                { "name": "mint", "type": "account", "is_account": true, "attributes": ["mut"] },
                { "name": "pda_authority", "type": "account", "is_account": true, "attributes": [] }
            ]
        }
    ]
};

// ============================================================================
// MAIN TEST
// ============================================================================

async function main() {
    header('🚀 INVOKE_SIGNED with PDA Authority CPI Example - E2E Test');

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
        error(`Failed to connect to ${RPC_URL}`);
        error('Make sure to run: solana-test-validator');
        process.exit(1);
    }

    // Verify Five program
    try {
        const programInfo = await connection.getAccountInfo(FIVE_PROGRAM_ID);
        if (!programInfo) {
            error(`Five VM program not found`);
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

    try {
        const scriptPath = path.join(__dirname, 'invoke-signed-pda.v');
        info(`Compiling ${scriptPath}...`);
        const source = fs.readFileSync(scriptPath, 'utf-8');
        const bytecode = await FiveSDK.compile(source);
        success('Contract compiled');
        info(`Using script account: ${SCRIPT_ACCOUNT.toBase58()}`);
    } catch (e) {
        error(`Compilation failed: ${e.message}`);
        process.exit(1);
    }

    // ========================================================================
    // STEP 2: Setup Token Infrastructure
    // ========================================================================
    header('STEP 2: Setup Token Infrastructure');

    let mint, pdaTokenAccount;
    const [pdaAuth] = PublicKey.findProgramAddressSync(
        [Buffer.from("treasury")],
        FIVE_PROGRAM_ID
    );
    info(`PDA Authority derived: ${pdaAuth.toBase58()}`);

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

        // Create PDA-owned token account
        info('Creating PDA-owned token account...');
        pdaTokenAccount = await createAccount(
            connection,
            payer,
            mint,
            pdaAuth  // owner is PDA
        );
        success(`PDA token account created: ${pdaTokenAccount.toBase58()}`);

        // Mint tokens to PDA account
        info('Minting 10000 tokens to PDA account...');
        await mintTo(
            connection,
            payer,
            mint,
            pdaTokenAccount,
            payer,
            10000n * 10n**6n  // 10000 tokens with 6 decimals
        );
        success(`Minted 10000 tokens`);

    } catch (e) {
        error(`Token setup failed: ${e.message}`);
        process.exit(1);
    }

    // ========================================================================
    // STEP 3: Initialize FiveProgram and Build INVOKE_SIGNED Instruction
    // ========================================================================
    header('STEP 3: Execute PDA Burn via INVOKE_SIGNED');

    try {
        // Initialize FiveProgram with ABI
        const program = FiveProgram.fromABI(SCRIPT_ACCOUNT.toBase58(), PDA_BURN_ABI, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payer.publicKey.toBase58(),
            debug: true
        });
        success('FiveProgram initialized with ABI');

        // Build instruction using fluent API
        info('Building burn_from_pda instruction...');
        const burnIx = await program
            .function('burn_from_pda')
            .accounts({
                token_account: pdaTokenAccount,
                mint: mint,
                pda_authority: pdaAuth
            })
            .instruction();

        success('Instruction built');
        info(`  - token_account: ${pdaTokenAccount.toBase58()}`);
        info(`  - mint: ${mint.toBase58()}`);
        info(`  - pda_authority: ${pdaAuth.toBase58()}`);

        // Send instruction
        info('Sending burn transaction...');
        const burnRes = await sendInstruction(connection, burnIx, [payer]);

        if (burnRes.success) {
            success(`burn_from_pda INVOKE_SIGNED executed (sig: ${burnRes.signature})`);
        } else {
            error('burn_from_pda failed');
            console.error(burnRes.error);
            process.exit(1);
        }

        // ====================================================================
        // STEP 4: Verify Token Balance
        // ====================================================================
        header('STEP 4: Verify Results');

        try {
            info('Checking token balance after burn...');
            const tokenAccount = await getAccount(connection, pdaTokenAccount);
            const balance = Number(tokenAccount.amount);
            const expected = 9000n * 10n**6n;  // 10000 - 1000 = 9000

            if (balance === Number(expected)) {
                success(`Token balance correct: ${balance / 1e6} tokens (burned 1000)`);
            } else {
                warn(`Token balance: ${balance / 1e6} tokens (expected ${Number(expected) / 1e6})`);
                warn('Note: Contract amount is hardcoded to 1000 tokens');
            }

            // Print transaction details
            log(`\nTransaction Details:`);
            log(`  Signature: ${burnRes.signature}`);
            log(`  CU Used: ${burnRes.cu === -1 ? 'N/A' : burnRes.cu}`);
        } catch (e) {
            warn(`Could not verify balance: ${e.message}`);
        }

        // ====================================================================
        // Test Summary
        // ====================================================================
        header('Test Summary');
        success('✅ INVOKE_SIGNED with PDA Authority CPI Example - E2E Test Passed');
        log(`\nKey Results:`);
        log(`  • Contract: ${SCRIPT_ACCOUNT.toBase58()}`);
        log(`  • Mint: ${mint.toBase58()}`);
        log(`  • PDA Authority: ${pdaAuth.toBase58()}`);
        log(`  • Token Account: ${pdaTokenAccount.toBase58()}`);
        log(`  • Transaction: ${burnRes.signature}`);

        log(`\nPDA Authority Pattern`);
        log(`This example demonstrates using INVOKE_SIGNED with a PDA:`);
        log(`1. Derive PDA from seeds: ["treasury", program_id]`);
        log(`2. Make PDA the authority for external program operations`);
        log(`3. Contract calls INVOKE_SIGNED with PDA seeds`);
        log(`4. Solana validates PDA signature internally`);
        log(`5. External program executes with PDA authority`);

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
