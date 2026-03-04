#!/usr/bin/env node
/**
 * Token Template E2E Test - FiveProgram API Version
 *
 * Tests core token operations using the high-level FiveProgram API.
 * This demonstrates the "Plug & Play" developer experience.
 *
 * Operations:
 * 1. Initialize mint
 * 2. Initialize token accounts
 * 3. Mint tokens
 * 4. Transfer
 * 5. Approve & Transfer From
 * 6. Revoke
 * 7. Burn
 * 8. Freeze/Thaw
 * 9. Disable Authority
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    SystemProgram, SYSVAR_RENT_PUBKEY, LAMPORTS_PER_SOL, sendAndConfirmTransaction
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
let RPC_URL = CFG.rpcUrl;
const PAYER_KEYPAIR_PATH = CFG.keypairPath;
let FIVE_PROGRAM_ID = new PublicKey(CFG.programId);
let VM_STATE_PDA = CFG.vmStatePda
    ? new PublicKey(CFG.vmStatePda)
    : PublicKey.findProgramAddressSync([Buffer.from('vm_state')], FIVE_PROGRAM_ID)[0];
const TOKEN_SCRIPT_ACCOUNT_RAW = process.env.FIVE_TOKEN_SCRIPT_ACCOUNT || process.env.TOKEN_SCRIPT_ACCOUNT || process.env.SCRIPT_ACCOUNT || '';
if (!TOKEN_SCRIPT_ACCOUNT_RAW) {
    throw new Error(
        'Missing FIVE_TOKEN_SCRIPT_ACCOUNT (or TOKEN_SCRIPT_ACCOUNT). ' +
        'Token E2E no longer auto-loads deployment-config.json because hidden network fallbacks are disabled.'
    );
}
let TOKEN_SCRIPT_ACCOUNT = new PublicKey(TOKEN_SCRIPT_ACCOUNT_RAW);
const FEE_VAULT_SEED_PREFIX = Buffer.from([0xff, ...Buffer.from('five_vm_fee_vault_v1')]);
const FEE_VAULT_ACCOUNT = process.env.FEE_VAULT_ACCOUNT
    ? new PublicKey(process.env.FEE_VAULT_ACCOUNT)
    : PublicKey.findProgramAddressSync([FEE_VAULT_SEED_PREFIX, Buffer.from([0])], FIVE_PROGRAM_ID)[0];

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

if (process.env.RPC_URL && !process.env.FIVE_RPC_URL) warn('Deprecated env RPC_URL detected; prefer FIVE_RPC_URL');

// ============================================================================
// HELPER: Error Extraction
// ============================================================================

/**
 * Extract compute units from transaction logs
 */
function extractCU(logs) {
    const cuLog = logs.find(l => l.includes('consumed'));
    if (!cuLog) return 'N/A';
    const match = cuLog.match(/consumed (\d+) of/);
    return match ? parseInt(match[1], 10) : 'N/A';
}

/**
 * Extract Five VM error from transaction logs
 * Returns error name if found (e.g., "IllegalOwner", "StackUnderflow")
 */
function extractVMError(logs) {
    // Look for Five VM program failure logs
    for (const log of logs) {
        // Pattern: "Program failed: <error message>"
        if (log.includes('failed:')) {
            const match = log.match(/failed: (.+)$/);
            if (match) {
                const errorMsg = match[1];

                // Map common Solana errors to VM errors
                if (errorMsg.includes('owner is not allowed')) return 'IllegalOwner';
                if (errorMsg.includes('stack underflow')) return 'StackUnderflow';
                if (errorMsg.includes('stack overflow')) return 'StackOverflow';
                if (errorMsg.includes('invalid instruction')) return 'InvalidInstruction';
                if (errorMsg.includes('account not found')) return 'AccountNotFound';

                return errorMsg;  // Return raw message if not mapped
            }
        }

        // Pattern: Custom error code (e.g., "Program returned error code: 0x1")
        if (log.includes('error code:')) {
            const match = log.match(/error code: (0x[0-9a-fA-F]+)/);
            if (match) return `ErrorCode(${match[1]})`;
        }
    }

    return null;
}

/**
 * Verify transaction result and fail test if not successful
 */
function assertTransactionSuccess(result, operationName) {
    if (!result.success) {
        console.error(`\n💥 TEST FAILED: ${operationName} transaction failed`);
        console.error(`   Signature: ${result.signature || 'N/A'}`);
        console.error(`   Error: ${result.error || 'Unknown'}`);
        if (result.vmError) {
            console.error(`   VM Error: ${result.vmError}`);
        }
        process.exit(1);
    }
}

// ============================================================================
// HELPER: Transaction Execution
// ============================================================================

async function sendInstruction(connection, instructionData, signers, label = '') {
    const keys = instructionData.keys.map(k => ({
        pubkey: new PublicKey(k.pubkey),
        isSigner: k.isSigner,
        isWritable: k.isWritable
    }));

    // Enforce execute tail expected by on-chain runtime, but avoid duplicating
    // when the SDK has already appended canonical [payer, fee_vault, system_program].
    const hasCanonicalTail = (() => {
        if (keys.length < 3) return false;
        const tailSystem = keys[keys.length - 1];
        const tailVault = keys[keys.length - 2];
        const tailPayer = keys[keys.length - 3];
        return (
            tailSystem.pubkey.toBase58() === SystemProgram.programId.toBase58() &&
            !tailSystem.isSigner &&
            !tailSystem.isWritable &&
            !tailVault.isSigner &&
            tailVault.isWritable &&
            tailPayer.isSigner &&
            tailPayer.isWritable
        );
    })();

    if (!hasCanonicalTail) {
        warn('Instruction missing canonical fee tail; applying legacy tail injection');
        const payerSigner = signers[0];
        if (payerSigner?.publicKey) {
            keys.push({
                pubkey: payerSigner.publicKey,
                isSigner: true,
                isWritable: true,
            });
        }
        keys.push({
            pubkey: FEE_VAULT_ACCOUNT,
            isSigner: false,
            isWritable: true,
        });
        keys.push({
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
        });
    }

    const ix = {
        programId: new PublicKey(instructionData.programId),
        keys: keys,
        data: Buffer.from(instructionData.data, 'base64')
    };

    const tx = new Transaction().add(ix);
    let signature = null;

    try {
        // CHANGE: Remove skipPreflight to enable pre-flight simulation
        signature = await sendAndConfirmTransaction(connection, tx, signers, {
            skipPreflight: false,
            commitment: 'confirmed'
        });

        await new Promise(r => setTimeout(r, 500));

        // Fetch transaction details
        const txDetails = await connection.getTransaction(signature, {
            maxSupportedTransactionVersion: 0,
            commitment: 'confirmed'
        });

        const logs = txDetails?.meta?.logMessages || [];

        // CRITICAL: Check for on-chain error FIRST
        if (txDetails?.meta?.err) {
            console.log(`\n❌ ${label} FAILED (on-chain error)`);
            console.log(`   Signature: ${signature}`);
            console.log(`   Error: ${JSON.stringify(txDetails.meta.err)}`);

            // Extract VM error if present
            const vmError = extractVMError(logs);
            if (vmError) {
                console.log(`   VM Error: ${vmError}`);
            }

            // Show relevant logs
            console.log(`   Logs:`);
            logs.forEach(log => {
                if (log.includes('Program') || log.includes('consumed') || log.includes('failed')) {
                    console.log(`     ${log}`);
                }
            });

            emitStepEvent({
                step: label || 'execute_instruction',
                status: 'FAIL',
                signature,
                computeUnits: null,
                missingCuReason: 'transaction meta.err present',
                error: JSON.stringify(txDetails.meta.err),
            });
            return {
                success: false,
                error: txDetails.meta.err,
                vmError,
                logs,
                signature,
                cu: extractCU(logs)  // Still track CU even on failure
            };
        }

        // Extract CU from successful transaction
        const cu = extractCU(logs);
        console.log(`✓ ${label} succeeded`);
        console.log(`   Signature: ${signature}`);
        console.log(`   CU: ${cu}`);
        emitStepEvent({
            step: label || 'execute_instruction',
            status: 'PASS',
            signature,
            computeUnits: Number.isFinite(Number(cu)) ? Number(cu) : null,
            missingCuReason: Number.isFinite(Number(cu)) ? null : 'compute units unavailable in transaction metadata/logs',
        });

        return {
            success: true,
            signature,
            logs,
            cu
        };

    } catch (error) {
        // Handle pre-flight simulation failure or RPC error
        console.log(`\n❌ ${label} FAILED (simulation or RPC error)`);
        console.log(`   Error: ${error.message}`);

        // Try to fetch logs if we have a signature
        if (signature) {
            try {
                const txDetails = await connection.getTransaction(signature, {
                    maxSupportedTransactionVersion: 0
                });
                const logs = txDetails?.meta?.logMessages || [];
                console.log(`   Logs:`);
                logs.forEach(log => console.log(`     ${log}`));
            } catch (e) {
                // Ignore log fetch errors
            }
        }

        // Check if this is a simulation error
        if (error.logs) {
            console.log(`   Simulation Logs:`);
            error.logs.forEach(log => console.log(`     ${log}`));

            const vmError = extractVMError(error.logs);
            if (vmError) {
                console.log(`   VM Error: ${vmError}`);
            }
        }

        emitStepEvent({
            step: label || 'execute_instruction',
            status: 'FAIL',
            signature,
            computeUnits: null,
            missingCuReason: 'transaction submission/simulation failed',
            error: error.message || String(error),
        });
        return {
            success: false,
            error: error.message,
            vmError: error.logs ? extractVMError(error.logs) : null,
            logs: error.logs || [],
            signature,
            cu: -1
        };
    }
}

// ============================================================================
// HELPER: Deployment Ownership Precheck
// ============================================================================

async function assertDeploymentOwnership(connection) {
    subheader('Deployment Ownership Precheck');

    const checks = [
        { label: 'Script Account', pubkey: TOKEN_SCRIPT_ACCOUNT },
        { label: 'VM State PDA', pubkey: VM_STATE_PDA }
    ];

    for (const check of checks) {
        let accountInfo;
        try {
            accountInfo = await connection.getAccountInfo(check.pubkey, 'confirmed');
        } catch (err) {
            throw new Error(
                `${check.label} lookup failed (${check.pubkey.toBase58()}): ${err.message}\n` +
                `RPC: ${RPC_URL}`
            );
        }

        if (!accountInfo) {
            throw new Error(
                `${check.label} not found on-chain: ${check.pubkey.toBase58()}\n` +
                `Run ./e2e-token-test.sh --deploy or five deploy build/five-token-template.five to create/update deployment accounts.`
            );
        }

        if (!accountInfo.owner.equals(FIVE_PROGRAM_ID)) {
            throw new Error(
                `${check.label} owner mismatch for ${check.pubkey.toBase58()}\n` +
                `Expected owner: ${FIVE_PROGRAM_ID.toBase58()}\n` +
                `Actual owner:   ${accountInfo.owner.toBase58()}\n` +
                `Redeploy with ./e2e-token-test.sh --deploy or five deploy build/five-token-template.five and pass the returned script account explicitly.`
            );
        }

        info(`${check.label} owner verified: ${check.pubkey.toBase58()}`);
    }
}

// ============================================================================ 
// TOKEN ABI (Embedded for reliability)
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
            "name": "init_token_account",
            "index": 1,
            "parameters": [
                { "name": "token_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut", "init", "signer"] },
                { "name": "owner", "type": "account", "is_account": true, "attributes": ["mut", "signer"] },
                { "name": "mint", "type": "pubkey" }
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
        },
        {
            "name": "transfer_from",
            "index": 4,
            "parameters": [
                { "name": "source_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "destination_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "authority", "type": "account", "is_account": true, "attributes": ["signer"] },
                { "name": "amount", "type": "u64" }
            ]
        },
        {
            "name": "approve",
            "index": 5,
            "parameters": [
                { "name": "source_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "owner", "type": "account", "is_account": true, "attributes": ["signer"] },
                { "name": "delegate", "type": "pubkey" },
                { "name": "amount", "type": "u64" }
            ]
        },
        {
            "name": "revoke",
            "index": 6,
            "parameters": [
                { "name": "source_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "owner", "type": "account", "is_account": true, "attributes": ["signer"] }
            ]
        },
        {
            "name": "burn",
            "index": 7,
            "parameters": [
                { "name": "mint_state", "type": "Mint", "is_account": true, "attributes": ["mut"] },
                { "name": "source_account", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "owner", "type": "account", "is_account": true, "attributes": ["signer"] },
                { "name": "amount", "type": "u64" }
            ]
        },
        {
            "name": "freeze_account",
            "index": 8,
            "parameters": [
                { "name": "mint_state", "type": "Mint", "is_account": true },
                { "name": "account_to_freeze", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "freeze_authority", "type": "account", "is_account": true, "attributes": ["signer"] }
            ]
        },
        {
            "name": "thaw_account",
            "index": 9,
            "parameters": [
                { "name": "mint_state", "type": "Mint", "is_account": true },
                { "name": "account_to_thaw", "type": "TokenAccount", "is_account": true, "attributes": ["mut"] },
                { "name": "freeze_authority", "type": "account", "is_account": true, "attributes": ["signer"] }
            ]
        },
        {
            "name": "set_mint_authority",
            "index": 10,
            "parameters": [
                { "name": "mint_state", "type": "Mint", "is_account": true, "attributes": ["mut"] },
                { "name": "current_authority", "type": "account", "is_account": true, "attributes": ["signer"] },
                { "name": "new_authority", "type": "pubkey" }
            ]
        },
        {
            "name": "set_freeze_authority",
            "index": 11,
            "parameters": [
                { "name": "mint_state", "type": "Mint", "is_account": true, "attributes": ["mut"] },
                { "name": "current_freeze_authority", "type": "account", "is_account": true, "attributes": ["signer"] },
                { "name": "new_freeze_authority", "type": "pubkey" }
            ]
        },
        {
            "name": "disable_mint",
            "index": 12,
            "parameters": [
                { "name": "mint_state", "type": "Mint", "is_account": true, "attributes": ["mut"] },
                { "name": "current_authority", "type": "account", "is_account": true, "attributes": ["signer"] }
            ]
        },
        {
            "name": "disable_freeze",
            "index": 13,
            "parameters": [
                { "name": "mint_state", "type": "Mint", "is_account": true, "attributes": ["mut"] },
                { "name": "current_freeze_authority", "type": "account", "is_account": true, "attributes": ["signer"] }
            ]
        }
    ]
};

// ============================================================================ 
// MAIN
// ============================================================================ 

async function main() {
    header('🚀 Token E2E Test with FiveProgram API');

    // 1. Setup Connection and Payer
    const connection = new Connection(RPC_URL, 'confirmed');
    const secretKey = JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'));
    const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));
    info(`Payer: ${payer.publicKey.toBase58()}`);

    // 2.5 Validate deployment ownership before building any instruction.
    await assertDeploymentOwnership(connection);

    // 2. Setup FiveProgram
    const program = FiveProgram.fromABI(TOKEN_SCRIPT_ACCOUNT.toBase58(), TOKEN_ABI, {
        fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
        vmStateAccount: VM_STATE_PDA.toBase58(),
        feeReceiverAccount: payer.publicKey.toBase58(),
        debug: true
    });
    success('FiveProgram initialized');

    // 3. Create Users
    const user1 = Keypair.generate(); // Authority
    const user2 = Keypair.generate(); // Holder
    const user3 = Keypair.generate(); // Holder

    // Fund users using transfer from payer (works on devnet, unlike airdrop)
    const fundAmount = 0.05 * LAMPORTS_PER_SOL; // 0.05 SOL each
    for (const user of [user1, user2, user3]) {
        const tx = new Transaction().add(
            SystemProgram.transfer({
                fromPubkey: payer.publicKey,
                toPubkey: user.publicKey,
                lamports: fundAmount,
            })
        );
        const sig = await sendAndConfirmTransaction(connection, tx, [payer]);
        info(`Funded ${user.publicKey.toBase58().slice(0, 8)}... with 0.05 SOL`);
    }
    info('Users created and funded');

    // 4. Generate Account Keypairs
    const mintAccount = Keypair.generate();
    const user1TokenAccount = Keypair.generate();
    const user2TokenAccount = Keypair.generate();
    const user3TokenAccount = Keypair.generate();
    info(`Mint: ${mintAccount.publicKey.toBase58()}`);

    // ======================================================================== 
    // STEP 1: Init Mint
    // ======================================================================== 
    header('STEP 1: Init Mint');

    const initMintIx = await program
        .function('init_mint')
        .accounts({
            mint_account: mintAccount.publicKey,
            authority: user1.publicKey
        })
        .args({
            freeze_authority: user1.publicKey,
            decimals: 6,
            name: "TestToken",
            symbol: "TEST",
            uri: "https://example.com/token"
        })
        .instruction();

    const initMintRes = await sendInstruction(connection, initMintIx, [payer, user1, mintAccount], 'init_mint');
    assertTransactionSuccess(initMintRes, 'init_mint');

    // ======================================================================== 
    // STEP 2: Init Token Accounts
    // ======================================================================== 
    header('STEP 2: Init Token Accounts');

    const accounts = [
        { kp: user1TokenAccount, owner: user1, name: 'User1' },
        { kp: user2TokenAccount, owner: user2, name: 'User2' },
        { kp: user3TokenAccount, owner: user3, name: 'User3' }
    ];

    for (const acc of accounts) {
        const ix = await program
            .function('init_token_account')
            .accounts({
                token_account: acc.kp.publicKey,
                owner: acc.owner.publicKey
            })
            .args({
                mint: mintAccount.publicKey
            })
            .instruction();

        const res = await sendInstruction(connection, ix, [payer, acc.owner, acc.kp], `init_token_account_${acc.name}`);
        assertTransactionSuccess(res, `init_token_account_${acc.name}`);
    }

    // ======================================================================== 
    // STEP 3: Mint To
    // ======================================================================== 
    header('STEP 3: Mint To');

    const mints = [
        { dest: user1TokenAccount, amount: 1000, name: 'User1' },
        { dest: user2TokenAccount, amount: 500, name: 'User2' },
        { dest: user3TokenAccount, amount: 500, name: 'User3' }
    ];

    for (const op of mints) {
        const ix = await program
            .function('mint_to')
            .accounts({
                mint_state: mintAccount.publicKey,
                destination_account: op.dest.publicKey,
                mint_authority: user1.publicKey
            })
            .args({
                amount: op.amount
            })
            .instruction();

        const res = await sendInstruction(connection, ix, [payer, user1], `mint_to_${op.name}`);
        assertTransactionSuccess(res, `mint_to_${op.name}`);
    }

    // ======================================================================== 
    // STEP 4: Transfer
    // ======================================================================== 
    header('STEP 4: Transfer');

    // Transfer 100 from User2 to User3
    const transferIx = await program
        .function('transfer')
        .accounts({
            source_account: user2TokenAccount.publicKey,
            destination_account: user3TokenAccount.publicKey,
            owner: user2.publicKey
        })
        .args({
            amount: 100
        })
        .instruction();

    const transferRes = await sendInstruction(connection, transferIx, [payer, user2], 'transfer');
    assertTransactionSuccess(transferRes, 'transfer');

    // ======================================================================== 
    // STEP 5: Approve & Transfer From
    // ======================================================================== 
    header('STEP 5: Approve & Transfer From');

    // User3 approves User2 to spend 150
    const approveIx = await program
        .function('approve')
        .accounts({
            source_account: user3TokenAccount.publicKey,
            owner: user3.publicKey
        })
        .args({
            delegate: user2.publicKey,
            amount: 150
        })
        .instruction();

    const approveRes = await sendInstruction(connection, approveIx, [payer, user3], 'approve');
    assertTransactionSuccess(approveRes, 'approve');

    // User2 transfers 50 from User3 to User1
    const transferFromIx = await program
        .function('transfer_from')
        .accounts({
            source_account: user3TokenAccount.publicKey,
            destination_account: user1TokenAccount.publicKey,
            authority: user2.publicKey // Delegate
        })
        .args({
            amount: 50
        })
        .instruction();

    const transferFromRes = await sendInstruction(connection, transferFromIx, [payer, user2], 'transfer_from');
    assertTransactionSuccess(transferFromRes, 'transfer_from');

    // ======================================================================== 
    // STEP 6: Revoke
    // ======================================================================== 
    header('STEP 6: Revoke');

    const revokeIx = await program
        .function('revoke')
        .accounts({
            source_account: user3TokenAccount.publicKey,
            owner: user3.publicKey
        })
        .instruction(); // No args for revoke

    const revokeRes = await sendInstruction(connection, revokeIx, [payer, user3], 'revoke');
    assertTransactionSuccess(revokeRes, 'revoke');

    // ======================================================================== 
    // STEP 7: Burn
    // ======================================================================== 
    header('STEP 7: Burn');

    const burnIx = await program
        .function('burn')
        .accounts({
            mint_state: mintAccount.publicKey,
            source_account: user1TokenAccount.publicKey,
            owner: user1.publicKey
        })
        .args({
            amount: 100
        })
        .instruction();

    const burnRes = await sendInstruction(connection, burnIx, [payer, user1], 'burn');
    assertTransactionSuccess(burnRes, 'burn');

    // ======================================================================== 
    // STEP 8: Freeze/Thaw
    // ======================================================================== 
    header('STEP 8: Freeze/Thaw');

    const freezeIx = await program
        .function('freeze_account')
        .accounts({
            mint_state: mintAccount.publicKey,
            account_to_freeze: user2TokenAccount.publicKey,
            freeze_authority: user1.publicKey
        })
        .instruction();

    const freezeRes = await sendInstruction(connection, freezeIx, [payer, user1], 'freeze_account');
    assertTransactionSuccess(freezeRes, 'freeze_account');

    // DEBUG: Inspect accounts before Thaw
    const mintInfo = await connection.getAccountInfo(mintAccount.publicKey);
    const tokenInfo = await connection.getAccountInfo(user2TokenAccount.publicKey);

    console.log("DEBUG STATE BEFORE THAW:");
    if (mintInfo) {
        // Mint: authority(32), freeze_auth(32), supply(8), decimals(1)...
        // Data layout:
        // 0-32: authority
        // 32-64: freeze_authority
        // 64-72: supply (u64)
        // 72: decimals (u8)
        const auth = new PublicKey(mintInfo.data.subarray(0, 32));
        const freezeAuth = new PublicKey(mintInfo.data.subarray(32, 64));
        console.log(`  Mint Authority: ${auth.toBase58()}`);
        console.log(`  Mint Freeze Auth: ${freezeAuth.toBase58()} (Expected: ${user1.publicKey.toBase58()})`);

        // Supply is at offset 64
        const supply = mintInfo.data.readBigUInt64LE(64);
        console.log(`  Mint Supply: ${supply}`);
    }
    if (tokenInfo) {
        // TokenAccount:
        // 0-32: owner
        // 32-64: mint
        // 64-72: balance (u64)
        // 72: is_frozen (bool)
        const owner = new PublicKey(tokenInfo.data.subarray(0, 32));
        const mint = new PublicKey(tokenInfo.data.subarray(32, 64));
        const frozen = tokenInfo.data[72];
        console.log(`  Token Owner: ${owner.toBase58()}`);
        console.log(`  Token Mint: ${mint.toBase58()} (Expected: ${mintAccount.publicKey.toBase58()})`);
        console.log(`  Token Frozen: ${frozen} (Expected: 1)`);
    }

    const thawIx = await program
        .function('thaw_account')
        .accounts({
            mint_state: mintAccount.publicKey,
            account_to_thaw: user2TokenAccount.publicKey,
            freeze_authority: user1.publicKey
        })
        .instruction();

    const thawRes = await sendInstruction(connection, thawIx, [payer, user1], 'thaw_account');
    assertTransactionSuccess(thawRes, 'thaw_account');

    // ======================================================================== 
    // STEP 9: Disable Authority
    // ======================================================================== 
    header('STEP 9: Disable Authority');

    const disableIx = await program
        .function('disable_mint')
        .accounts({
            mint_state: mintAccount.publicKey,
            current_authority: user1.publicKey
        })
        .instruction();

    const disableRes = await sendInstruction(connection, disableIx, [payer, user1], 'disable_mint');
    assertTransactionSuccess(disableRes, 'disable_mint');

    // Export state
    const testState = {
        config: {
            programId: FIVE_PROGRAM_ID.toBase58(),
            script: TOKEN_SCRIPT_ACCOUNT.toBase58()
        },
        accounts: {
            mint: mintAccount.publicKey.toBase58(),
            user1: user1.publicKey.toBase58(),
            user2: user2.publicKey.toBase58(),
            user3: user3.publicKey.toBase58(),
            user1Token: user1TokenAccount.publicKey.toBase58(),
            user2Token: user2TokenAccount.publicKey.toBase58(),
            user3Token: user3TokenAccount.publicKey.toBase58()
        },
        results: {
            initMint: initMintRes.success,
            mintTo: true, // simplified
            transfer: transferRes.success,
            approve: approveRes.success,
            transferFrom: transferFromRes.success,
            revoke: revokeRes.success,
            burn: burnRes.success,
            freeze: freezeRes.success,
            disable: disableRes.success
        }
    };

    fs.writeFileSync(path.join(__dirname, 'test-state-fiveprogram.json'), JSON.stringify(testState, null, 2));
    success('Test state saved to test-state-fiveprogram.json');

    // ======================================================================== 
    // STEP 10: Verify Balances
    // ======================================================================== 
    header('STEP 10: Verify Balances');

    const verifyBalance = async (tokenAccountPubkey, expectedBalance, label) => {
        const account = await connection.getAccountInfo(tokenAccountPubkey);
        if (!account) {
            error(`${label} NOT FOUND`);
            return;
        }

        // Parse balance from offset 64 (after owner[32] and mint[32])
        // using u64 LE format
        const balance = Number(account.data.readBigUInt64LE(64));
        if (balance === expectedBalance) {
            success(`${label} balance verified: ${balance}`);
        } else {
            error(`${label} balance MISMATCH: Expected ${expectedBalance}, Got ${balance}`);
        }
    };

    // Expected balances after all operations:
    // User1: 1000 (mint) + 50 (transfer_from) - 100 (burn) = 950
    // User2: 500 (mint) - 100 (transfer) = 400
    // User3: 500 (mint) + 100 (transfer) - 50 (transfer_from) = 550

    await verifyBalance(user1TokenAccount.publicKey, 950, 'User1 Token Account');
    await verifyBalance(user2TokenAccount.publicKey, 400, 'User2 Token Account');
    await verifyBalance(user3TokenAccount.publicKey, 550, 'User3 Token Account');

    console.log('\n🚀 Token E2E Test Completed Successfully!');
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
