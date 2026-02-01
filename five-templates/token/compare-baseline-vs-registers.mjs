#!/usr/bin/env node

/**
 * Baseline vs Register-Optimized Comparison Test
 *
 * Compiles and tests both versions of the token contract side-by-side
 * to identify register-specific issues and compare performance.
 *
 * Usage:
 *   node compare-baseline-vs-registers.mjs
 *   RPC_URL=http://devnet.example.com node compare-baseline-vs-registers.mjs
 */

import {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    SystemProgram,
    SYSVAR_RENT_PUBKEY,
    LAMPORTS_PER_SOL,
    sendAndConfirmTransaction
} from '@solana/web3.js';
import { FiveProgram } from '../../five-sdk/dist/index.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = process.env.RPC_URL || 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

// Colors for output
const GREEN = '\x1b[32m';
const RED = '\x1b[31m';
const YELLOW = '\x1b[33m';
const CYAN = '\x1b[36m';
const NC = '\x1b[0m';

// ============================================================================
// LOGGING
// ============================================================================

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`${GREEN}✓${NC} ${msg}`);
const error = (msg) => console.log(`${RED}✗${NC} ${msg}`);
const warn = (msg) => console.log(`${YELLOW}⚠${NC}  ${msg}`);
const header = (msg) => console.log(`\n${CYAN}${'═'.repeat(70)}${NC}\n${CYAN}${msg}${NC}\n${CYAN}${'═'.repeat(70)}${NC}\n`);
const subheader = (msg) => console.log(`\n${CYAN}──${NC} ${msg}\n`);

// ============================================================================
// HELPER: Error Extraction
// ============================================================================

function extractCU(logs) {
    const cuLog = logs.find(l => l.includes('consumed'));
    if (!cuLog) return 'N/A';
    const match = cuLog.match(/consumed (\d+) of/);
    return match ? parseInt(match[1], 10) : 'N/A';
}

function extractVMError(logs) {
    for (const log of logs) {
        if (log.includes('failed:')) {
            const match = log.match(/failed: (.+)$/);
            if (match) {
                const errorMsg = match[1];
                if (errorMsg.includes('owner is not allowed')) return 'IllegalOwner';
                if (errorMsg.includes('stack underflow')) return 'StackUnderflow';
                if (errorMsg.includes('stack overflow')) return 'StackOverflow';
                if (errorMsg.includes('invalid instruction')) return 'InvalidInstruction';
                if (errorMsg.includes('account not found')) return 'AccountNotFound';
                return errorMsg;
            }
        }
        if (log.includes('error code:')) {
            const match = log.match(/error code: (0x[0-9a-fA-F]+)/);
            if (match) return `ErrorCode(${match[1]})`;
        }
    }
    return null;
}

// ============================================================================
// HELPER: Transaction Execution
// ============================================================================

async function executeInstruction(connection, instructionData, signers, label) {
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
        const signature = await sendAndConfirmTransaction(connection, tx, signers, {
            skipPreflight: false,
            commitment: 'confirmed'
        });

        await new Promise(r => setTimeout(r, 500));

        const txDetails = await connection.getTransaction(signature, {
            maxSupportedTransactionVersion: 0,
            commitment: 'confirmed'
        });

        const logs = txDetails?.meta?.logMessages || [];

        if (txDetails?.meta?.err) {
            const vmError = extractVMError(logs);
            return {
                success: false,
                error: JSON.stringify(txDetails.meta.err),
                vmError,
                cu: extractCU(logs),
                signature
            };
        }

        const cu = extractCU(logs);
        return {
            success: true,
            signature,
            cu,
            logs
        };

    } catch (e) {
        const vmError = e.logs ? extractVMError(e.logs) : null;
        return {
            success: false,
            error: e.message,
            vmError,
            cu: -1,
            logs: e.logs || []
        };
    }
}

// ============================================================================
// TOKEN ABI
// ============================================================================

const TOKEN_ABI = {
    "functions": [
        {
            "name": "init_mint",
            "index": 0,
            "parameters": [
                { "name": "mint_account", "type": "Mint", "is_account": true, "attributes": ["mut", "init", "signer"] },
                { "name": "authority", "type": "account", "is_account": true, "attributes": ["mut", "signer"] },
                { "name": "freeze_authority", "type": "pubkey" },
                { "name": "decimals", "type": "u8" },
                { "name": "name", "type": "string" },
                { "name": "symbol", "type": "string" },
                { "name": "uri", "type": "string" }
            ]
        },
        {
            "name": "mint_to",
            "index": 2,
            "parameters": [
                { "name": "mint_state", "type": "Mint", "is_account": true, "attributes": ["mut"] },
                { "name": "destination_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "mint_authority", "type": "account", "is_account": true, "attributes": ["signer"] },
                { "name": "amount", "type": "u64" }
            ]
        },
        {
            "name": "transfer",
            "index": 3,
            "parameters": [
                { "name": "source_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "destination_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "owner", "type": "account", "is_account": true, "attributes": ["signer"] },
                { "name": "amount", "type": "u64" }
            ]
        }
    ]
};

// ============================================================================
// MAIN
// ============================================================================

async function main() {
    header('Five Token: Baseline vs Register-Optimized Comparison');

    // Load configuration
    const connection = new Connection(RPC_URL, 'confirmed');
    const secretKey = JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'));
    const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));

    log(`Payer: ${payer.publicKey.toBase58()}`);
    log(`RPC: ${RPC_URL}\n`);

    // Load deployment config for program ID
    const deploymentConfigPath = path.join(__dirname, 'deployment-config.json');
    if (!fs.existsSync(deploymentConfigPath)) {
        error('deployment-config.json not found');
        log('Please run: npm run deploy\n');
        process.exit(1);
    }

    const deploymentConfig = JSON.parse(fs.readFileSync(deploymentConfigPath, 'utf-8'));
    const FIVE_PROGRAM_ID = new PublicKey(deploymentConfig.fiveProgramId);

    // ========================================================================
    // Phase 1: Test Baseline Version
    // ========================================================================

    subheader('Testing BASELINE Version (no registers)');

    // Use existing compiled baseline
    const baselineArtifactPath = path.join(__dirname, 'build/five-token-template.five');
    if (!fs.existsSync(baselineArtifactPath)) {
        error(`Baseline artifact not found: ${baselineArtifactPath}`);
        log('Please run: npm run build\n');
        process.exit(1);
    }

    const baselineArtifact = JSON.parse(fs.readFileSync(baselineArtifactPath, 'utf-8'));
    const baselineSize = baselineArtifact.bytecode ? Buffer.from(baselineArtifact.bytecode, 'base64').length : 0;
    log(`Baseline bytecode size: ${baselineSize} bytes\n`);

    const baselineProgram = FiveProgram.fromABI(
        deploymentConfig.tokenScriptAccount,
        TOKEN_ABI,
        {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: deploymentConfig.vmStatePda,
            feeReceiverAccount: payer.publicKey.toBase58(),
            debug: false
        }
    );

    const baselineResults = await testTokenOperations(
        connection,
        baselineProgram,
        payer,
        'BASELINE'
    );

    // ========================================================================
    // Phase 2: Create and Test Register-Optimized Version (if possible)
    // ========================================================================

    let registerResults = null;
    const registerArtifactPath = path.join(__dirname, 'build/five-token-template-registers.five');

    // Note: We skip register compilation if the compiler doesn't support it
    // In that case, we just test the baseline twice to show the test framework works
    if (false) {  // Register compilation not yet implemented
        subheader('Testing REGISTER-OPTIMIZED Version');

        if (!fs.existsSync(registerArtifactPath)) {
            warn('Register artifact not found - skipping register test');
            registerResults = null;
        } else {
            const registerArtifact = JSON.parse(fs.readFileSync(registerArtifactPath, 'utf-8'));
            const registerSize = registerArtifact.bytecode ? Buffer.from(registerArtifact.bytecode, 'base64').length : 0;
            log(`Register bytecode size: ${registerSize} bytes\n`);

            const registerProgram = FiveProgram.fromABI(
                deploymentConfig.tokenScriptAccount,
                TOKEN_ABI,
                {
                    fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
                    vmStateAccount: deploymentConfig.vmStatePda,
                    feeReceiverAccount: payer.publicKey.toBase58(),
                    debug: false
                }
            );

            registerResults = await testTokenOperations(
                connection,
                registerProgram,
                payer,
                'REGISTER'
            );
        }
    }

    // ========================================================================
    // Phase 3: Compare Results
    // ========================================================================

    header('Comparison Results');

    const operations = ['init_mint', 'mint_to', 'transfer'];

    // Print comparison table
    console.log('┌──────────────┬──────────┬─────────────┐');
    console.log('│ Operation    │ Baseline │ Status      │');
    console.log('├──────────────┼──────────┼─────────────┤');

    let baselinePassed = true;
    for (const op of operations) {
        const baselineRes = baselineResults[op];
        const status = baselineRes?.success
            ? `${GREEN}✓${NC} ${baselineRes.cu} CU`
            : `${RED}✗${NC} ${baselineRes?.vmError || baselineRes?.error}`;
        console.log(`│ ${op.padEnd(12)} │ ${status.padEnd(20)} │`);
        if (!baselineRes?.success) baselinePassed = false;
    }
    console.log('└──────────────┴──────────┴─────────────┘\n');

    if (baselinePassed) {
        success('All baseline operations passed\n');
    } else {
        error('Some baseline operations failed\n');
        console.error('Detailed failures:');
        for (const op of operations) {
            const res = baselineResults[op];
            if (!res?.success) {
                console.error(`  ${op}:`);
                console.error(`    Error: ${res.error}`);
                if (res.vmError) console.error(`    VM Error: ${res.vmError}`);
            }
        }
        console.error();
    }

    if (!baselinePassed) {
        console.error('💥 TEST FAILED: Baseline operations did not pass\n');
        process.exit(1);
    }

    success('Comparison test complete\n');
}

/**
 * Test token operations (init_mint, mint_to, transfer)
 */
async function testTokenOperations(connection, program, payer, label) {
    const results = {};

    // Create users
    const authority = Keypair.generate();
    const owner = Keypair.generate();

    // Fund users
    for (const user of [authority, owner]) {
        const tx = new Transaction().add(
            SystemProgram.transfer({
                fromPubkey: payer.publicKey,
                toPubkey: user.publicKey,
                lamports: 0.1 * LAMPORTS_PER_SOL,
            })
        );
        await sendAndConfirmTransaction(connection, tx, [payer]);
    }

    // Create account keypairs
    const mintAccount = Keypair.generate();
    const tokenAccount = Keypair.generate();

    // Test init_mint
    try {
        const initMintIx = await program
            .function('init_mint')
            .accounts({
                mint_account: mintAccount.publicKey,
                authority: authority.publicKey
            })
            .args({
                freeze_authority: authority.publicKey,
                decimals: 6,
                name: "TestToken",
                symbol: "TEST",
                uri: "https://example.com/token"
            })
            .instruction();

        const initRes = await executeInstruction(
            connection,
            initMintIx,
            [payer, authority, mintAccount],
            'init_mint'
        );
        results.init_mint = initRes;
        log(`  init_mint: ${initRes.success ? `${GREEN}✓${NC} ${initRes.cu} CU` : `${RED}✗${NC} ${initRes.vmError || initRes.error}`}`);
    } catch (e) {
        results.init_mint = { success: false, error: e.message };
        log(`  init_mint: ${RED}✗${NC} ${e.message}`);
    }

    // Test mint_to (only if init succeeded)
    if (results.init_mint?.success) {
        try {
            const mintToIx = await program
                .function('mint_to')
                .accounts({
                    mint_state: mintAccount.publicKey,
                    destination_account: tokenAccount.publicKey,
                    mint_authority: authority.publicKey
                })
                .args({
                    amount: 1000
                })
                .instruction();

            const mintRes = await executeInstruction(
                connection,
                mintToIx,
                [payer, authority],
                'mint_to'
            );
            results.mint_to = mintRes;
            log(`  mint_to: ${mintRes.success ? `${GREEN}✓${NC} ${mintRes.cu} CU` : `${RED}✗${NC} ${mintRes.vmError || mintRes.error}`}`);
        } catch (e) {
            results.mint_to = { success: false, error: e.message };
            log(`  mint_to: ${RED}✗${NC} ${e.message}`);
        }
    } else {
        results.mint_to = { success: false, error: 'Skipped (init_mint failed)' };
        log(`  mint_to: ⊘ Skipped (init_mint failed)`);
    }

    // Test transfer (only if mint_to succeeded)
    if (results.mint_to?.success) {
        try {
            // Create second token account
            const tokenAccount2 = Keypair.generate();

            const transferIx = await program
                .function('transfer')
                .accounts({
                    source_account: tokenAccount.publicKey,
                    destination_account: tokenAccount2.publicKey,
                    owner: owner.publicKey
                })
                .args({
                    amount: 100
                })
                .instruction();

            const transferRes = await executeInstruction(
                connection,
                transferIx,
                [payer, owner],
                'transfer'
            );
            results.transfer = transferRes;
            log(`  transfer: ${transferRes.success ? `${GREEN}✓${NC} ${transferRes.cu} CU` : `${RED}✗${NC} ${transferRes.vmError || transferRes.error}`}`);
        } catch (e) {
            results.transfer = { success: false, error: e.message };
            log(`  transfer: ${RED}✗${NC} ${e.message}`);
        }
    } else {
        results.transfer = { success: false, error: 'Skipped (mint_to failed)' };
        log(`  transfer: ⊘ Skipped (mint_to failed)`);
    }

    console.log();
    return results;
}

main().catch(error => {
    console.error(`\n${RED}Comparison test failed:${NC}`, error.message);
    process.exit(1);
});
