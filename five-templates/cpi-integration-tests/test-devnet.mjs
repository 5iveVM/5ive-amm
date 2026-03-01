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
    getOrCreateAssociatedTokenAccount, mintTo, burn, getAccount
} from '@solana/spl-token';
import { FiveProgram, FiveSDK } from '../../five-sdk/dist/index.js';
import { loadSdkValidatorConfig } from '../../scripts/lib/sdk-validator-config.mjs';
import { emitStepEvent } from '../../scripts/lib/sdk-validator-reporter.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION
// ============================================================================

const CFG = loadSdkValidatorConfig({
    network: process.env.FIVE_NETWORK || 'devnet',
});
const RPC_URL = CFG.rpcUrl;
const PAYER_KEYPAIR_PATH = CFG.keypairPath;
const FIVE_PROGRAM_ID = new PublicKey(CFG.programId);
const VM_STATE_PDA = CFG.vmStatePda
    ? new PublicKey(CFG.vmStatePda)
    : PublicKey.findProgramAddressSync([Buffer.from('vm_state')], FIVE_PROGRAM_ID)[0];

// ============================================================================
// LOGGING
// ============================================================================

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`✅ ${msg}`);
const error = (msg) => console.log(`❌ ${msg}`);
const info = (msg) => console.log(`ℹ️  ${msg}`);
const warn = (msg) => console.log(`⚠️  ${msg}`);
const header = (msg) => console.log(`\n${'='.repeat(80)}\n${msg}\n${'='.repeat(80)}`);

function extractFunctionParamAttributes(source) {
    const attrsByFunction = {};
    const functionRegex = /pub\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([\s\S]*?)\)\s*(?:->\s*[^{]+)?\s*\{/g;
    let fnMatch;
    while ((fnMatch = functionRegex.exec(source)) !== null) {
        const fnName = fnMatch[1];
        const rawParams = fnMatch[2].trim();
        const paramAttrs = {};
        if (rawParams.length > 0) {
            const paramList = rawParams.split(',').map((p) => p.trim()).filter(Boolean);
            for (const rawParam of paramList) {
                const colonIdx = rawParam.indexOf(':');
                if (colonIdx === -1) continue;
                const paramName = rawParam.slice(0, colonIdx).trim();
                const attrMatches = [...rawParam.matchAll(/@([a-zA-Z_][a-zA-Z0-9_]*)/g)];
                paramAttrs[paramName] = attrMatches.map((m) => m[1]);
            }
        }
        attrsByFunction[fnName] = paramAttrs;
    }
    return attrsByFunction;
}

function normalizeAbiForFiveProgram(abi, source) {
    if (!abi || !Array.isArray(abi.functions)) {
        return abi;
    }
    const attrsByFunction = extractFunctionParamAttributes(source);
    return {
        ...abi,
        functions: abi.functions.map((fn) => ({
            ...fn,
            parameters: (fn.parameters || []).map((p) => ({
                ...p,
                is_account: p.is_account ?? p.isAccount ?? false,
                isAccount: p.isAccount ?? p.is_account ?? false,
                type: (() => {
                    const raw = p.type || p.param_type;
                    if (raw === 'Account') return 'account';
                    return raw;
                })(),
                attributes: Array.isArray(p.attributes) && p.attributes.length > 0
                    ? [...p.attributes]
                    : [...(attrsByFunction[fn.name]?.[p.name] || [])]
            }))
        }))
    };
}

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

        const compilation = await FiveSDK.compile(source);
        const bytecode = compilation?.bytecode;
        const runtimeAbi = normalizeAbiForFiveProgram(compilation?.abi, source);
        if (!bytecode || !runtimeAbi) {
            throw new Error(`compile failed: ${compilation?.error || 'missing bytecode/abi'}`);
        }

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
        const program2 = FiveProgram.fromABI(scriptAccount.toBase58(), runtimeAbi, {
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
                authority: payerKeypair.publicKey,
                token_program: TOKEN_PROGRAM_ID
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

        if (balance === 1000) {
            success(`Balance correct: ${balance / 1e6} tokens`);
            return true;
        } else {
            error(`Balance incorrect: expected 1000, got ${balance}`);
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
        const pdaTokenAccount = (await getOrCreateAssociatedTokenAccount(
            connection,
            payerKeypair,
            mint,
            payerKeypair.publicKey,
            true
        )).address;
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

        const compilation = await FiveSDK.compile(source);
        const bytecode = compilation?.bytecode;
        const runtimeAbi = normalizeAbiForFiveProgram(compilation?.abi, source);
        if (!bytecode || !runtimeAbi) {
            throw new Error(`compile failed: ${compilation?.error || 'missing bytecode/abi'}`);
        }

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
        const program2 = FiveProgram.fromABI(scriptAccount.toBase58(), runtimeAbi, {
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
                pda_authority: payerKeypair.publicKey,
                token_program: TOKEN_PROGRAM_ID
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
        const expected = (10000n * 10n**6n) - 1000n;

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
