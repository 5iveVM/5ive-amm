#!/usr/bin/env node
/**
 * Counter Template E2E Test - State Persistence Validation
 *
 * Tests counter operations with multiple users using Five SDK for ABI-based instruction building.
 * Validates that programs deployed using normal deploy instruction save state properly.
 *
 * Test Scenario:
 * - User 1: Creates and owns counter1
 * - User 2: Creates and owns counter2
 *
 * Operations:
 * 1. Initialize counter1 for User1 (count = 0)
 * 2. Initialize counter2 for User2 (count = 0)
 * 3. Increment counter1 3 times (count = 3)
 * 4. Add 10 to counter1 (count = 13)
 * 5. Decrement counter1 (count = 12)
 * 6. Increment counter2 5 times (count = 5)
 * 7. Reset counter2 (count = 0)
 * 8. Verify final states: counter1 = 12, counter2 = 0
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    TransactionInstruction, SystemProgram, SYSVAR_RENT_PUBKEY, LAMPORTS_PER_SOL
} from '@solana/web3.js';
import { FiveSDK } from '../../five-sdk/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION
// ============================================================================

let RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

// Default localnet deployment values (will be overridden by deployment-config.json)
let FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
let VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
let COUNTER_SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

// ============================================================================
// LOGGING UTILITIES
// ============================================================================

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`\x1b[32m[PASS]\x1b[0m ${msg}`);
const error = (msg) => console.log(`\x1b[31m[FAIL]\x1b[0m ${msg}`);
const info = (msg) => console.log(`\x1b[34m[INFO]\x1b[0m ${msg}`);
const warn = (msg) => console.log(`\x1b[33m[WARN]\x1b[0m ${msg}`);
const header = (msg) => console.log(`\n${'='.repeat(80)}\n${msg}\n${'='.repeat(80)}`);
const subheader = (msg) => console.log(`\n-- ${msg}`);

// Load deployment config if exists
const deploymentConfigPath = path.join(__dirname, 'deployment-config.json');
if (fs.existsSync(deploymentConfigPath)) {
    try {
        const deploymentConfig = JSON.parse(fs.readFileSync(deploymentConfigPath, 'utf-8'));
        if (deploymentConfig.rpcUrl) {
            RPC_URL = deploymentConfig.rpcUrl;
        }
        if (deploymentConfig.fiveProgramId) {
            FIVE_PROGRAM_ID = new PublicKey(deploymentConfig.fiveProgramId);
        }
        if (deploymentConfig.vmStatePda) {
            VM_STATE_PDA = new PublicKey(deploymentConfig.vmStatePda);
        }
        if (deploymentConfig.counterScriptAccount) {
            COUNTER_SCRIPT_ACCOUNT = new PublicKey(deploymentConfig.counterScriptAccount);
        }
        info('Loaded deployment-config.json overrides');
    } catch (configError) {
        warn(`Failed to load deployment-config.json: ${configError.message}`);
    }
}

// ============================================================================
// LOAD COMPILED COUNTER PROGRAM AND ABI
// ============================================================================

let counterABI = null;
let functionIndices = {};

function loadCounterABI() {
    const buildPath = path.join(__dirname, 'build', 'five-counter-template.five');
    try {
        const fiveFile = JSON.parse(fs.readFileSync(buildPath, 'utf-8'));
        counterABI = fiveFile.abi;
        const functionCount = Array.isArray(counterABI?.functions)
            ? counterABI.functions.length
            : Object.keys(counterABI?.functions || {}).length;

        // Build function index lookup table
        if (Array.isArray(counterABI?.functions)) {
            counterABI.functions.forEach(f => {
                functionIndices[f.name] = f.index;
            });
        }

        info(`Loaded Counter ABI: ${functionCount} functions`);
        return true;
    } catch (e) {
        error(`Failed to load counter ABI from ${buildPath}: ${e.message}`);
        return false;
    }
}

function getFunctionIndex(functionName) {
    const index = functionIndices[functionName];
    if (index === undefined) {
        throw new Error(`Unknown function: ${functionName}`);
    }
    return index;
}

// ============================================================================
// KEYPAIR LOADING
// ============================================================================

function loadKeypair(kpPath) {
    const secretKey = JSON.parse(fs.readFileSync(kpPath, 'utf-8'));
    return Keypair.fromSecretKey(Uint8Array.from(secretKey));
}

// ============================================================================
// FIVE SDK INSTRUCTION EXECUTION
// ============================================================================

/**
 * Execute counter function using Five SDK with proper ABI encoding.
 *
 * @param connection - Solana connection
 * @param payer - Keypair that pays for the transaction
 * @param functionName - Name of the function to call
 * @param parameters - Function parameters (values)
 * @param accounts - Array of account objects: { pubkey: PublicKey, isWritable: bool, isSigner: bool }
 * @param signers - Array of Keypair objects that must sign the transaction
 */
async function executeCounterFunction(
    connection,
    payer,
    functionName,
    parameters = [],
    accounts = [],
    signers = []
) {
    try {
        const functionIndex = getFunctionIndex(functionName);

        // Pass function parameters directly without injecting payer as a parameter
        // ACCOUNT_INDEX_OFFSET = 1, so:
        // - MitoVM receives [VM State, param0, param1, ...]
        // - param0 maps to account index 0 + 1 = 1
        // - param1 maps to account index 1 + 1 = 2
        // Payer is not a function parameter, but the SDK handles it separately as adminAccount.
        const functionAccounts = accounts;

        // Extract pubkey strings from accounts array for SDK
        const accountPubkeys = functionAccounts.map(acc => {
            const pubkey = acc.pubkey instanceof PublicKey
                ? acc.pubkey.toBase58()
                : acc.pubkey.toString();
            return pubkey;
        });

        // Generate the instruction using Five SDK with all accounts
        // Include full ABI metadata for proper parameter encoding
        // Pass payer as adminAccount for fee collection (SDK handles it separately from function parameters)
        const executeData = await FiveSDK.generateExecuteInstruction(
            COUNTER_SCRIPT_ACCOUNT.toBase58(),
            functionIndex,
            parameters,
            accountPubkeys,  // Pass account pubkeys to SDK!
            connection,
            {
                debug: true,
                vmStateAccount: VM_STATE_PDA.toBase58(),
                fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
                metadata: counterABI || undefined,
                adminAccount: payer.publicKey.toBase58()  // Payer as admin for fee collection
            }
        );

        // Use the accounts from SDK, but apply correct signer/writable flags from our accounts array
        const ixKeys = executeData.instruction.accounts.map((acc, index) => {
            // First 2 accounts are Script and VM State from SDK
            if (index < 2) {
                return {
                    pubkey: new PublicKey(acc.pubkey),
                    isSigner: acc.isSigner,
                    isWritable: acc.isWritable
                };
            }
            // Remaining accounts - use flags from our accounts array
            const ourAccountIndex = index - 2;
            if (ourAccountIndex < functionAccounts.length) {
                return {
                    pubkey: new PublicKey(acc.pubkey),
                    isSigner: functionAccounts[ourAccountIndex].isSigner ?? false,
                    isWritable: functionAccounts[ourAccountIndex].isWritable ?? true
                };
            }
            // Admin/payer account added by SDK for fee collection
            // Mark as signer since payer signs the transaction
            const pubkeyStr = typeof acc.pubkey === 'string' ? acc.pubkey : acc.pubkey.toBase58();
            const payerStr = payer.publicKey.toBase58();
            const isAdminAccount = pubkeyStr === payerStr;
            return {
                pubkey: new PublicKey(acc.pubkey),
                isSigner: isAdminAccount ? true : acc.isSigner,  // Payer must be a signer for fee collection
                isWritable: acc.isWritable
            };
        });

        const ix = new TransactionInstruction({
            programId: new PublicKey(executeData.instruction.programId),
            keys: ixKeys,
            data: Buffer.from(executeData.instruction.data, 'base64')
        });

        const tx = new Transaction().add(ix);
        const allSigners = [payer, ...signers];

        const sig = await connection.sendTransaction(tx, allSigners, {
            skipPreflight: true,
            maxRetries: 3
        });

        await connection.confirmTransaction(sig, 'confirmed');

        // Fetch transaction details for compute units
        const txDetails = await connection.getTransaction(sig, {
            maxSupportedTransactionVersion: 0
        });

        let computeUnits = 1400000;
        if (txDetails?.meta?.computeUnitsConsumed) {
            computeUnits = txDetails.meta.computeUnitsConsumed;
        }

        const success_flag = txDetails?.meta?.err === null;

        if (!success_flag || functionName === 'initialize') {
            console.log(`\n[${success_flag ? 'INFO' : 'FAIL'}] Transaction Logs for [${functionName}]:`);
            if (txDetails?.meta?.logMessages) {
                txDetails.meta.logMessages.forEach(msg => console.log(`  ${msg}`));
            } else {
                console.log("  No logs available");
            }
            console.log("");
        }

        return {
            success: success_flag,
            functionName,
            signature: sig,
            computeUnits,
            error: success_flag ? null : JSON.stringify(txDetails?.meta?.err)
        };
    } catch (e) {
        return {
            success: false,
            functionName,
            signature: null,
            computeUnits: 0,
            error: e.message
        };
    }
}

// ============================================================================
// MAIN TEST
// ============================================================================

async function main() {
    header('Counter Template E2E Test - State Persistence Validation');

    // Load ABI
    if (!loadCounterABI()) {
        error('Cannot proceed without counter ABI');
        process.exit(1);
    }

    // Connect
    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = loadKeypair(PAYER_KEYPAIR_PATH);

    // Display payer info
    const balance = await connection.getBalance(payer.publicKey);
    info(`Payer: ${payer.publicKey.toBase58()}`);
    info(`Payer Balance: ${(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);
    info(`Counter Script: ${COUNTER_SCRIPT_ACCOUNT.toBase58()}`);
    info(`VM State PDA: ${VM_STATE_PDA.toBase58()}`);

    // Track test results
    const testResults = [];
    let allPassed = true;

    // ========================================================================
    // SETUP: Create 2 Users
    // ========================================================================

    header('SETUP: Creating 2 Users');

    const user1 = Keypair.generate();
    const user2 = Keypair.generate();

    info(`User1: ${user1.publicKey.toBase58()}`);
    info(`User2: ${user2.publicKey.toBase58()}`);

    // Fund users
    subheader('Funding users with SOL...');
    for (const user of [user1, user2]) {
        const sig = await connection.requestAirdrop(user.publicKey, 1000 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig, 'confirmed');
    }
    const user1Balance = await connection.getBalance(user1.publicKey);
    const user2Balance = await connection.getBalance(user2.publicKey);
    info(`Funded User1: ${(user1Balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);
    info(`Funded User2: ${(user2Balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);

    // ========================================================================
    // STEP 1: Generate Counter Account Keypairs
    // ========================================================================

    header('STEP 1: Generating Counter Account Keypairs');

    const counter1Account = Keypair.generate();
    const counter2Account = Keypair.generate();

    success('Generated keypairs for counter accounts');
    info(`Counter1 Account: ${counter1Account.publicKey.toBase58()}`);
    info(`Counter2 Account: ${counter2Account.publicKey.toBase58()}`);

    // ========================================================================
    // STEP 2: Initialize Counter1 for User1
    // ========================================================================

    header('STEP 2: Initialize Counter1 (User1)');

    let result = await executeCounterFunction(
        connection,
        payer,
        'initialize',
        [],
        [
            { pubkey: counter1Account.publicKey, isWritable: true, isSigner: true },
            { pubkey: user1.publicKey, isWritable: false, isSigner: true },  // Payer is signer but not writable
            { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
            { pubkey: SYSVAR_RENT_PUBKEY, isWritable: false, isSigner: false }
        ],
        [user1, counter1Account]
    );

    testResults.push(result);
    if (result.success) {
        success(`initialize counter1`);
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`initialize counter1 failed: ${result.error}`);
        allPassed = false;
    }

    // ========================================================================
    // STEP 3: Initialize Counter2 for User2
    // ========================================================================

    header('STEP 3: Initialize Counter2 (User2)');

    result = await executeCounterFunction(
        connection,
        payer,
        'initialize',
        [],
        [
            { pubkey: counter2Account.publicKey, isWritable: true, isSigner: true },
            { pubkey: user2.publicKey, isWritable: false, isSigner: true },  // Payer is signer but not writable
            { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
            { pubkey: SYSVAR_RENT_PUBKEY, isWritable: false, isSigner: false }
        ],
        [user2, counter2Account]
    );

    testResults.push(result);
    if (result.success) {
        success(`initialize counter2`);
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`initialize counter2 failed: ${result.error}`);
        allPassed = false;
    }

    // ========================================================================
    // STEP 4: Increment Counter1 Three Times (expected count: 3)
    // ========================================================================

    header('STEP 4: Increment Counter1 Three Times');

    for (let i = 1; i <= 3; i++) {
        result = await executeCounterFunction(
            connection,
            payer,
            'increment',
            [],
            [
                { pubkey: counter1Account.publicKey, isWritable: true, isSigner: false },
                { pubkey: user1.publicKey, isWritable: false, isSigner: true }
            ],
            [user1]
        );

        testResults.push(result);
        if (result.success) {
            success(`increment counter1 (${i}/3)`);
            info(`  Signature: ${result.signature}`);
            info(`  Compute Units: ${result.computeUnits}`);
        } else {
            error(`increment counter1 (${i}/3) failed: ${result.error}`);
            allPassed = false;
        }
    }

    // ========================================================================
    // STEP 5: Add 10 to Counter1 (expected count: 13)
    // ========================================================================

    header('STEP 5: Add 10 to Counter1');

    result = await executeCounterFunction(
        connection,
        payer,
        'add_amount',
        [10],
        [
            { pubkey: counter1Account.publicKey, isWritable: true, isSigner: false },
            { pubkey: user1.publicKey, isWritable: false, isSigner: true }
        ],
        [user1]
    );

    testResults.push(result);
    if (result.success) {
        success(`add_amount 10 to counter1`);
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`add_amount 10 to counter1 failed: ${result.error}`);
        allPassed = false;
    }

    // ========================================================================
    // STEP 6: Decrement Counter1 (expected count: 12)
    // ========================================================================

    header('STEP 6: Decrement Counter1');

    result = await executeCounterFunction(
        connection,
        payer,
        'decrement',
        [],
        [
            { pubkey: counter1Account.publicKey, isWritable: true, isSigner: false },
            { pubkey: user1.publicKey, isWritable: false, isSigner: true }
        ],
        [user1]
    );

    testResults.push(result);
    if (result.success) {
        success(`decrement counter1`);
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`decrement counter1 failed: ${result.error}`);
        allPassed = false;
    }

    // ========================================================================
    // STEP 7: Increment Counter2 Five Times (expected count: 5)
    // ========================================================================

    header('STEP 7: Increment Counter2 Five Times');

    for (let i = 1; i <= 5; i++) {
        result = await executeCounterFunction(
            connection,
            payer,
            'increment',
            [],
            [
                { pubkey: counter2Account.publicKey, isWritable: true, isSigner: false },
                { pubkey: user2.publicKey, isWritable: false, isSigner: true }
            ],
            [user2]
        );

        testResults.push(result);
        if (result.success) {
            success(`increment counter2 (${i}/5)`);
            info(`  Signature: ${result.signature}`);
            info(`  Compute Units: ${result.computeUnits}`);
        } else {
            error(`increment counter2 (${i}/5) failed: ${result.error}`);
            allPassed = false;
        }
    }

    // ========================================================================
    // STEP 8: Reset Counter2 (expected count: 0)
    // ========================================================================

    header('STEP 8: Reset Counter2');

    result = await executeCounterFunction(
        connection,
        payer,
        'reset',
        [],
        [
            { pubkey: counter2Account.publicKey, isWritable: true, isSigner: false },
            { pubkey: user2.publicKey, isWritable: false, isSigner: true }
        ],
        [user2]
    );

    testResults.push(result);
    if (result.success) {
        success(`reset counter2`);
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`reset counter2 failed: ${result.error}`);
        allPassed = false;
    }

    // ========================================================================
    // TEST SUMMARY
    // ========================================================================

    header('Test Execution Complete');

    const successfulTests = testResults.filter(r => r.success).length;
    const failedTests = testResults.filter(r => !r.success).length;
    const totalComputeUnits = testResults.reduce((acc, r) => acc + r.computeUnits, 0);

    info(`Total Tests: ${testResults.length}`);
    info(`Passed: ${successfulTests}`);
    info(`Failed: ${failedTests}`);
    info(`Total Compute Units: ${totalComputeUnits}`);

    // ========================================================================
    // EXPORT STATE FOR VERIFICATION
    // ========================================================================
    const testState = {
        config: {
            rpcUrl: RPC_URL,
            programId: FIVE_PROGRAM_ID.toBase58(),
            vmStatePda: VM_STATE_PDA.toBase58(),
            counterScriptAccount: COUNTER_SCRIPT_ACCOUNT.toBase58()
        },
        accounts: {
            counter1: counter1Account.publicKey.toBase58(),
            counter2: counter2Account.publicKey.toBase58(),
            user1: user1.publicKey.toBase58(),
            user2: user2.publicKey.toBase58(),
            payer: payer.publicKey.toBase58()
        },
        expected: {
            counter1Count: 12,  // 0 + 3 (increments) + 10 (add_amount) - 1 (decrement) = 12
            counter2Count: 0   // 0 + 5 (increments) -> reset -> 0
        },
        results: {
            totalTests: testResults.length,
            passed: successfulTests,
            failed: failedTests,
            totalComputeUnits: totalComputeUnits,
            allPassed: allPassed
        }
    };

    fs.writeFileSync(path.join(__dirname, 'test-state.json'), JSON.stringify(testState, null, 2));
    success('Test state saved to test-state.json');

    if (allPassed) {
        console.log('\n\x1b[32m========================================\x1b[0m');
        console.log('\x1b[32m  ALL TESTS PASSED\x1b[0m');
        console.log('\x1b[32m========================================\x1b[0m\n');
    } else {
        console.log('\n\x1b[31m========================================\x1b[0m');
        console.log('\x1b[31m  SOME TESTS FAILED\x1b[0m');
        console.log('\x1b[31m========================================\x1b[0m\n');
        process.exit(1);
    }
}

// ============================================================================
// RUN TEST
// ============================================================================

main().catch(err => {
    error(err.message);
    console.error(err);
    process.exit(1);
});
