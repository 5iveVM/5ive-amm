#!/usr/bin/env node
/**
 * Five CPI Integration Test Suite - Localnet
 *
 * Comprehensive on-chain integration tests for CPI functionality.
 * Tests CPI calls to real SPL Token program on Solana localnet.
 *
 * Requirements:
 * - solana-test-validator running
 * - Five VM program deployed
 * - npm dependencies installed
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    TransactionInstruction, SystemProgram, LAMPORTS_PER_SOL,
    sendAndConfirmTransaction
} from '@solana/web3.js';
import {
    TOKEN_PROGRAM_ID, createMint, createAccount,
    mintTo, burn, getMint, getAccount
} from '@solana/spl-token';
import { FiveProgram } from '../../five-sdk/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION & LOGGING
// ============================================================================

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

let FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
let VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');

// ============================================================================
// CONTRACT ABIs
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

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`✅ ${msg}`);
const error = (msg) => console.log(`❌ ${msg}`);
const info = (msg) => console.log(`ℹ️  ${msg}`);
const warn = (msg) => console.log(`⚠️  ${msg}`);
const header = (msg) => console.log(`\n${'='.repeat(80)}\n${msg}\n${'='.repeat(80)}`);

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
// TEST SETUP
// ============================================================================

async function setupTest() {
    header('Five CPI Integration Test Suite - Localnet');

    // Load payer
    const payerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8')))
    );

    // Connect
    const connection = new Connection(RPC_URL, 'confirmed');

    // Verify connection
    try {
        const version = await connection.getVersion();
        success(`Connected to localnet (Solana ${version['solana-core']})`);
    } catch (e) {
        error(`Failed to connect to ${RPC_URL}`);
        error('Start localnet with: solana-test-validator');
        process.exit(1);
    }

    // Verify Five program
    try {
        const programInfo = await connection.getAccountInfo(FIVE_PROGRAM_ID);
        if (!programInfo) {
            error(`Five VM program not found at ${FIVE_PROGRAM_ID.toBase58()}`);
            process.exit(1);
        }
        success(`Five VM program: ${FIVE_PROGRAM_ID.toBase58()}`);
    } catch (e) {
        error(`Error checking Five program: ${e.message}`);
        process.exit(1);
    }

    return { connection, payerKeypair };
}

// ============================================================================
// TEST 1: SPL TOKEN MINT VIA CPI
// ============================================================================

async function testSPLTokenMint(connection, payerKeypair) {
    header('Test 1: SPL Token Mint via CPI');

    try {
        // Create token mint
        info('Creating token mint...');
        const mint = await createMint(
            connection,
            payerKeypair,
            payerKeypair.publicKey,  // mint authority
            null,                      // freeze authority
            6                           // decimals
        );
        success(`Mint created: ${mint.toBase58()}`);

        // Create destination token account
        info('Creating destination token account...');
        const destTokenAccount = await createAccount(
            connection,
            payerKeypair,
            mint,
            payerKeypair.publicKey  // owner
        );
        success(`Token account created: ${destTokenAccount.toBase58()}`);

        // Load and compile test contract
        info('Compiling mint test contract...');
        const contractPath = path.join(__dirname, 'test-spl-token-mint.v');
        const source = fs.readFileSync(contractPath, 'utf-8');

        const { FiveSDK } = await import('../../five-sdk/dist/index.js');
        const bytecode = await FiveSDK.compile(source);
        success('Contract compiled');

        // Deploy contract
        info('Deploying contract...');
        const program = new FiveProgram(connection, FIVE_PROGRAM_ID, payerKeypair);
        const deployment = await program.deployScript(bytecode, {
            vmStateAccount: VM_STATE_PDA
        });
        const scriptAccount = deployment.scriptAccount;
        success(`Contract deployed: ${scriptAccount.toBase58()}`);

        // Initialize FiveProgram with ABI
        info('Building mint instruction with FiveProgram API...');
        const program = FiveProgram.fromABI(scriptAccount.toBase58(), SPL_TOKEN_MINT_ABI, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payerKeypair.publicKey.toBase58(),
            debug: false
        });

        // Build instruction using fluent API
        const mintIx = await program
            .function('mint_tokens')
            .accounts({
                mint: mint,
                to: destTokenAccount,
                authority: payerKeypair.publicKey
            })
            .instruction();

        success('Instruction built');

        // Send instruction
        info('Sending mint transaction...');
        const mintRes = await sendInstruction(connection, mintIx, [payerKeypair]);

        if (!mintRes.success) {
            error('Mint transaction failed');
            return false;
        }
        success(`Transaction: ${mintRes.signature}`);

        // Verify results
        info('Verifying token balance...');
        const tokenAccount = await getAccount(connection, destTokenAccount);
        const balance = Number(tokenAccount.amount);

        if (balance === 1000000000) {  // 1000 tokens with 6 decimals
            success(`Token balance correct: ${balance / 1e6} tokens`);
            return true;
        } else {
            error(`Token balance incorrect: expected 1000000000, got ${balance}`);
            return false;
        }

    } catch (e) {
        error(`Test failed: ${e.message}`);
        console.error(e);
        return false;
    }
}

// ============================================================================
// TEST 2: SPL TOKEN BURN VIA INVOKE_SIGNED
// ============================================================================

async function testSPLTokenBurnPDA(connection, payerKeypair) {
    header('Test 2: SPL Token Burn via INVOKE_SIGNED');

    try {
        // Create token mint
        info('Creating token mint...');
        const mint = await createMint(
            connection,
            payerKeypair,
            payerKeypair.publicKey,  // mint authority
            null,                      // freeze authority
            6                           // decimals
        );
        success(`Mint created: ${mint.toBase58()}`);

        // Derive PDA for burn authority
        info('Deriving PDA for burn authority...');
        // In real usage: Pubkey.findProgramAddress(["treasury"], fiveProgramId)
        // For testing, use a derived address
        const [pdaAuth] = PublicKey.findProgramAddressSync(
            [Buffer.from("treasury")],
            FIVE_PROGRAM_ID
        );
        success(`PDA derived: ${pdaAuth.toBase58()}`);

        // Create token account owned by PDA
        info('Creating PDA-owned token account...');
        // This would require special setup in production
        // For now, create account and transfer authority
        const pdaTokenAccount = await createAccount(
            connection,
            payerKeypair,
            mint,
            pdaAuth  // owner is PDA
        );
        success(`PDA token account: ${pdaTokenAccount.toBase58()}`);

        // Mint tokens to PDA account
        info('Minting 10000 tokens to PDA account...');
        const mintTx = await mintTo(
            connection,
            payerKeypair,
            mint,
            pdaTokenAccount,
            payerKeypair.publicKey,
            10000n * 10n**6n  // 10000 tokens with 6 decimals
        );
        success(`Mint tx: ${mintTx}`);

        // Load and compile test contract
        info('Compiling burn test contract...');
        const contractPath = path.join(__dirname, 'test-pda-burn.v');
        const source = fs.readFileSync(contractPath, 'utf-8');

        const { FiveSDK } = await import('../../five-sdk/dist/index.js');
        const bytecode = await FiveSDK.compile(source);
        success('Contract compiled');

        // Deploy contract
        info('Deploying contract...');
        const program = new FiveProgram(connection, FIVE_PROGRAM_ID, payerKeypair);
        const deployment = await program.deployScript(bytecode, {
            vmStateAccount: VM_STATE_PDA
        });
        const scriptAccount = deployment.scriptAccount;
        success(`Contract deployed: ${scriptAccount.toBase58()}`);

        // Initialize FiveProgram with ABI
        info('Building burn instruction with FiveProgram API...');
        const program = FiveProgram.fromABI(scriptAccount.toBase58(), PDA_BURN_ABI, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payerKeypair.publicKey.toBase58(),
            debug: false
        });

        // Build instruction using fluent API
        const burnIx = await program
            .function('burn_from_pda')
            .accounts({
                token_account: pdaTokenAccount,
                mint: mint,
                pda_authority: pdaAuth
            })
            .instruction();

        success('Instruction built');

        // Send instruction
        info('Sending burn transaction...');
        const burnRes = await sendInstruction(connection, burnIx, [payerKeypair]);

        if (!burnRes.success) {
            error('Burn transaction failed');
            return false;
        }
        success(`Transaction: ${burnRes.signature}`);

        // Verify results
        info('Verifying token balance after burn...');
        const tokenAccount = await getAccount(connection, pdaTokenAccount);
        const balance = Number(tokenAccount.amount);
        const expected = 9000n * 10n**6n;  // 10000 - 1000 = 9000 tokens

        if (balance === Number(expected)) {
            success(`Token balance correct: ${balance / 1e6} tokens`);
            return true;
        } else {
            error(`Token balance incorrect: expected ${expected}, got ${balance}`);
            return false;
        }

    } catch (e) {
        error(`Test failed: ${e.message}`);
        console.error(e);
        return false;
    }
}

// ============================================================================
// MAIN TEST RUNNER
// ============================================================================

async function main() {
    try {
        const { connection, payerKeypair } = await setupTest();

        const results = {
            test1: false,
            test2: false
        };

        // Run tests
        results.test1 = await testSPLTokenMint(connection, payerKeypair);
        results.test2 = await testSPLTokenBurnPDA(connection, payerKeypair);

        // Summary
        header('Test Summary');

        log(`Test 1 (SPL Token Mint):  ${results.test1 ? '✅ PASS' : '❌ FAIL'}`);
        log(`Test 2 (SPL Token Burn):  ${results.test2 ? '✅ PASS' : '❌ FAIL'}`);

        const passed = Object.values(results).filter(r => r).length;
        const total = Object.values(results).length;

        log(`\nTotal: ${passed}/${total} passed`);

        if (passed === total) {
            success('\n✅ All CPI Integration Tests Passed');
            process.exit(0);
        } else {
            error('\n❌ Some tests failed');
            process.exit(1);
        }

    } catch (e) {
        error(`Fatal error: ${e.message}`);
        console.error(e);
        process.exit(1);
    }
}

main().catch(e => {
    error(`Unexpected error: ${e.message}`);
    console.error(e);
    process.exit(1);
});
