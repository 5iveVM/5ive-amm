#!/usr/bin/env node
/**
 * Five CPI Integration Test Suite - Devnet
 *
 * Same tests as localnet, but runs against live devnet.
 * Uses real SPL Token program and persistent on-chain state.
 *
 * Setup:
 * - solana config set -u devnet
 * - solana airdrop 10
 * - Five VM program deployed on devnet
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    SystemProgram, LAMPORTS_PER_SOL, sendAndConfirmTransaction
} from '@solana/web3.js';
import {
    TOKEN_PROGRAM_ID, createMint, createAccount,
    mintTo, burn, getAccount
} from '@solana/spl-token';
import { FiveProgram, FiveSDK } from '../../five-sdk/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION
// ============================================================================

const RPC_URL = process.env.FIVE_RPC_URL || process.env.RPC_URL || 'https://api.devnet.solana.com';
const PAYER_KEYPAIR_PATH = process.env.FIVE_KEYPAIR_PATH || process.env.PAYER_KEYPAIR_PATH || (process.env.HOME + '/.config/solana/id.json');

// Load from environment or use defaults
let FIVE_PROGRAM_ID = new PublicKey(process.env.FIVE_PROGRAM_ID || '9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
let VM_STATE_PDA = new PublicKey(process.env.VM_STATE_PDA || 'DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');

// ============================================================================
// LOGGING
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

// ============================================================================
// SETUP
// ============================================================================

async function setupTest() {
    header('Five CPI Integration Test Suite - Devnet');

    // Load payer
    const payerKeypair = Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8')))
    );
    info(`Payer: ${payerKeypair.publicKey.toBase58()}`);

    // Connect
    const connection = new Connection(RPC_URL, 'confirmed');

    try {
        const version = await connection.getVersion();
        success(`Connected to devnet`);
    } catch (e) {
        error(`Failed to connect to devnet: ${e.message}`);
        error('Try: solana config set -u devnet');
        process.exit(1);
    }

    // Check balance
    try {
        const balance = await connection.getBalance(payerKeypair.publicKey);
        if (balance < 0.5 * 1e9) {
            warn(`Low balance: ${balance / 1e9} SOL`);
            warn('Airdrop: solana airdrop 10 -u devnet');
        } else {
            info(`Balance: ${balance / 1e9} SOL`);
        }
    } catch (e) {
        warn(`Could not check balance: ${e.message}`);
    }

    // Verify Five program
    try {
        const programInfo = await connection.getAccountInfo(FIVE_PROGRAM_ID);
        if (!programInfo) {
            error(`Five VM program not found at ${FIVE_PROGRAM_ID.toBase58()}`);
            error('Deploy with: five deploy <program.so> --url devnet');
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
// TEST 1: SPL TOKEN MINT
// ============================================================================

async function testSPLTokenMint(connection, payerKeypair) {
    header('Test 1: SPL Token Mint via CPI (Devnet)');

    try {
        info('Creating token mint...');
        const mint = await createMint(
            connection,
            payerKeypair,
            payerKeypair.publicKey,
            null,
            6
        );
        success(`Mint: ${mint.toBase58()}`);

        info('Creating destination token account...');
        const destTokenAccount = await createAccount(
            connection,
            payerKeypair,
            mint,
            payerKeypair.publicKey
        );
        success(`Token account: ${destTokenAccount.toBase58()}`);

        info('Compiling and deploying contract...');
        const contractPath = path.join(__dirname, 'test-spl-token-mint.v');
        const source = fs.readFileSync(contractPath, 'utf-8');

        const bytecode = await FiveSDK.compile(source);

        const deployment = await FiveSDK.deployToSolana(bytecode, connection, payerKeypair, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            debug: false,
        });
        if (!deployment.success || !deployment.programId) {
            throw new Error(`deployToSolana failed: ${deployment.error || 'unknown error'}`);
        }
        const scriptAccount = new PublicKey(deployment.programId);
        success(`Contract: ${scriptAccount.toBase58()}`);

        info('Building mint instruction...');
        const program2 = FiveProgram.fromABI(scriptAccount.toBase58(), SPL_TOKEN_MINT_ABI, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payerKeypair.publicKey.toBase58(),
            debug: false
        });

        const mintIx = await program2
            .function('mint_tokens')
            .accounts({
                mint: mint,
                to: destTokenAccount,
                authority: payerKeypair.publicKey
            })
            .instruction();

        info('Executing mint via CPI...');
        const mintRes = await sendInstruction(connection, mintIx, [payerKeypair]);
        if (!mintRes.success) {
            error('Mint failed');
            return false;
        }
        success(`Tx: ${mintRes.signature}`);

        info('Verifying token balance...');
        const tokenAccount = await getAccount(connection, destTokenAccount);
        const balance = Number(tokenAccount.amount);

        if (balance === 1000000000) {
            success(`Balance correct: ${balance / 1e6} tokens`);
            return true;
        } else {
            error(`Balance incorrect: expected 1000000000, got ${balance}`);
            return false;
        }

    } catch (e) {
        error(`Test failed: ${e.message}`);
        console.error(e);
        return false;
    }
}

// ============================================================================
// TEST 2: SPL TOKEN BURN VIA PDA
// ============================================================================

async function testSPLTokenBurnPDA(connection, payerKeypair) {
    header('Test 2: SPL Token Burn via INVOKE_SIGNED (Devnet)');

    try {
        info('Creating token mint...');
        const mint = await createMint(
            connection,
            payerKeypair,
            payerKeypair.publicKey,
            null,
            6
        );
        success(`Mint: ${mint.toBase58()}`);

        info('Deriving PDA...');
        const [pdaAuth] = PublicKey.findProgramAddressSync(
            [Buffer.from("treasury")],
            FIVE_PROGRAM_ID
        );
        success(`PDA: ${pdaAuth.toBase58()}`);

        info('Creating PDA-owned token account...');
        const pdaTokenAccount = await createAccount(
            connection,
            payerKeypair,
            mint,
            pdaAuth
        );
        success(`PDA token account: ${pdaTokenAccount.toBase58()}`);

        info('Minting 10000 tokens...');
        const mintTx = await mintTo(
            connection,
            payerKeypair,
            mint,
            pdaTokenAccount,
            payerKeypair.publicKey,
            10000n * 10n**6n
        );
        success(`Mint tx: ${mintTx}`);

        info('Compiling and deploying contract...');
        const contractPath = path.join(__dirname, 'test-pda-burn.v');
        const source = fs.readFileSync(contractPath, 'utf-8');

        const bytecode = await FiveSDK.compile(source);

        const deployment = await FiveSDK.deployToSolana(bytecode, connection, payerKeypair, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            debug: false,
        });
        if (!deployment.success || !deployment.programId) {
            throw new Error(`deployToSolana failed: ${deployment.error || 'unknown error'}`);
        }
        const scriptAccount = new PublicKey(deployment.programId);
        success(`Contract: ${scriptAccount.toBase58()}`);

        info('Building burn instruction...');
        const program2 = FiveProgram.fromABI(scriptAccount.toBase58(), PDA_BURN_ABI, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payerKeypair.publicKey.toBase58(),
            debug: false
        });

        const burnIx = await program2
            .function('burn_from_pda')
            .accounts({
                token_account: pdaTokenAccount,
                mint: mint,
                pda_authority: pdaAuth
            })
            .instruction();

        info('Executing burn via INVOKE_SIGNED...');
        const burnRes = await sendInstruction(connection, burnIx, [payerKeypair]);
        if (!burnRes.success) {
            error('Burn failed');
            return false;
        }
        success(`Tx: ${burnRes.signature}`);

        info('Verifying token balance...');
        const tokenAccount = await getAccount(connection, pdaTokenAccount);
        const balance = Number(tokenAccount.amount);
        const expected = 9000n * 10n**6n;

        if (balance === Number(expected)) {
            success(`Balance correct: ${balance / 1e6} tokens`);
            return true;
        } else {
            error(`Balance incorrect: expected ${expected}, got ${balance}`);
            return false;
        }

    } catch (e) {
        error(`Test failed: ${e.message}`);
        console.error(e);
        return false;
    }
}

// ============================================================================
// MAIN
// ============================================================================

async function main() {
    try {
        const { connection, payerKeypair } = await setupTest();

        const results = {
            test1: await testSPLTokenMint(connection, payerKeypair),
            test2: await testSPLTokenBurnPDA(connection, payerKeypair)
        };

        header('Test Summary');

        log(`Test 1 (SPL Mint):  ${results.test1 ? '✅ PASS' : '❌ FAIL'}`);
        log(`Test 2 (SPL Burn):  ${results.test2 ? '✅ PASS' : '❌ FAIL'}`);

        const passed = Object.values(results).filter(r => r).length;
        const total = Object.values(results).length;

        log(`\nTotal: ${passed}/${total} passed`);

        if (passed === total) {
            success('\n✅ All Devnet CPI Tests Passed');
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
