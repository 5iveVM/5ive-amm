#!/usr/bin/env node
/**
 * Anchor Program Call CPI Example E2E Test
 *
 * Demonstrates:
 * - Calling an Anchor program with 8-byte discriminators
 * - Mixed account and data parameters
 * - Borsh serialization for Anchor compatibility
 * - FiveProgram fluent API for instruction building
 *
 * Requirements:
 * - Running Solana localnet (solana-test-validator)
 * - Deployed Five VM program
 * - Deployed Anchor counter program
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    SystemProgram, LAMPORTS_PER_SOL, sendAndConfirmTransaction
} from '@solana/web3.js';
import { FiveSDK, FiveProgram } from '../../five-sdk/dist/index.js';
import { loadSdkValidatorConfig } from '../../scripts/lib/sdk-validator-config.mjs';
import { emitStepEvent } from '../../scripts/lib/sdk-validator-reporter.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION
// ============================================================================

const CFG = loadSdkValidatorConfig({
    network: process.env.FIVE_NETWORK || 'localnet',
});
const RPC_URL = CFG.rpcUrl;
const PAYER_KEYPAIR_PATH = CFG.keypairPath;
const FIVE_PROGRAM_ID = new PublicKey(CFG.programId);
const VM_STATE_PDA = CFG.vmStatePda
    ? new PublicKey(CFG.vmStatePda)
    : PublicKey.findProgramAddressSync([Buffer.from('vm_state')], FIVE_PROGRAM_ID)[0];

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

async function sendInstruction(connection, instructionData, signers, step = 'execute_instruction') {
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
                emitStepEvent({
                    step,
                    status: 'FAIL',
                    signature: sig,
                    computeUnits: null,
                    missingCuReason: 'transaction meta.err present',
                    error: JSON.stringify(txDetails.meta.err),
                });
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

        emitStepEvent({
            step,
            status: 'PASS',
            signature: sig,
            computeUnits: Number.isFinite(Number(cu)) && Number(cu) >= 0 ? Number(cu) : null,
            missingCuReason: Number.isFinite(Number(cu)) && Number(cu) >= 0 ? null : 'compute units unavailable in transaction metadata/logs',
        });
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
        emitStepEvent({
            step,
            status: 'FAIL',
            signature: e.signature || null,
            computeUnits: null,
            missingCuReason: 'transaction submission failed',
            error: e.message || String(e),
        });
        return { success: false, error: e, logs };
    }
}

// ============================================================================
// CONTRACT ABI (Anchor Counter Example)
// ============================================================================

const ANCHOR_COUNTER_ABI = {
    "functions": [
        {
            "name": "increment_remote",
            "index": 0,
            "parameters": [
                { "name": "counter", "type": "account", "is_account": true, "attributes": ["mut"] },
                { "name": "user", "type": "account", "is_account": true, "attributes": ["signer"] }
            ]
        }
    ]
};

// ============================================================================
// MAIN TEST
// ============================================================================

async function main() {
    header('🚀 Anchor Program Call CPI Example - E2E Test');

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
    let scriptAccount;

    try {
        const scriptPath = path.join(__dirname, 'anchor-program-call.v');
        info(`Compiling ${scriptPath}...`);
        const source = fs.readFileSync(scriptPath, 'utf-8');
        const compilation = await FiveSDK.compile(source);
        const bytecode = compilation?.bytecode;
        if (!bytecode) {
            throw new Error(`Compile failed: ${compilation?.error || 'missing bytecode'}`);
        }
        success('Contract compiled');
        info('Deploying compiled contract...');
        const deployment = await FiveSDK.deployToSolana(bytecode, connection, payer, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            debug: false,
        });
        if (!deployment.success || !deployment.programId) {
            throw new Error(`deployToSolana failed: ${deployment.error || 'unknown error'}`);
        }
        scriptAccount = new PublicKey(deployment.programId);
        success(`Using script account: ${scriptAccount.toBase58()}`);
    } catch (e) {
        error(`Compilation failed: ${e.message}`);
        process.exit(1);
    }

    // ========================================================================
    // STEP 2: Create Test Accounts
    // ========================================================================
    header('STEP 2: Setup Test Accounts');

    const user = payer;
    const counter = Keypair.generate();

    success(`Using payer as user signer: ${user.publicKey.toBase58()}`);
    // Pre-fund counter so downstream CPI transfer does not violate rent-exempt checks.
    try {
        const fundCounterTx = new Transaction().add(
            SystemProgram.transfer({
                fromPubkey: payer.publicKey,
                toPubkey: counter.publicKey,
                lamports: 0.01 * LAMPORTS_PER_SOL
            })
        );
        await sendAndConfirmTransaction(connection, fundCounterTx, [payer]);
        success(`Counter pre-funded: ${counter.publicKey.toBase58()}`);
    } catch (e) {
        warn(`Could not pre-fund counter: ${e.message}`);
    }

    // ========================================================================
    // STEP 3: Initialize FiveProgram and Build Instruction
    // ========================================================================
    header('STEP 3: Execute Anchor CPI via Five Contract');

    try {
        // Initialize FiveProgram with ABI
        const program = FiveProgram.fromABI(scriptAccount.toBase58(), ANCHOR_COUNTER_ABI, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payer.publicKey.toBase58(),
            debug: true
        });
        success('FiveProgram initialized with ABI');

        // Build instruction using fluent API
        info('Building increment_remote instruction...');
        const incrementIx = await program
            .function('increment_remote')
            .accounts({
                counter: counter.publicKey,
                user: user.publicKey
            })
            .instruction();

        success('Instruction built');
        info(`  - counter: ${counter.publicKey.toBase58()}`);
        info(`  - user: ${user.publicKey.toBase58()}`);

        // Send instruction
        info('Sending increment transaction...');
        const incrementRes = await sendInstruction(connection, incrementIx, [payer]);

        if (incrementRes.success) {
            success(`increment_remote CPI executed (sig: ${incrementRes.signature})`);
        } else {
            error('increment_remote failed');
            console.error(incrementRes.error);
            process.exit(1);
        }

        // ====================================================================
        // Test Summary
        // ====================================================================
        header('Test Summary');
        success('✅ Anchor Program Call CPI Example - E2E Test Passed');
        log(`\nKey Results:`);
        log(`  • Contract: ${scriptAccount.toBase58()}`);
        log(`  • Counter Account: ${counter.publicKey.toBase58()}`);
        log(`  • User: ${user.publicKey.toBase58()}`);
        log(`  • Transaction: ${incrementRes.signature}`);

        log(`\nNote: 8-byte Discriminator Format`);
        log(`This example uses Anchor's 8-byte sighash discriminator format:`);
        log(`@discriminator([0xAA, 0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF])`);
        log(`\nTo integrate with a real Anchor program:`);
        log(`1. Get the program's IDL (Interface Definition Language)`);
        log(`2. Find the instruction's sighash in the IDL`);
        log(`3. Update the contract with correct program ID and discriminator`);
        log(`4. Deploy and test against the real program`);

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
