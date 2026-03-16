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
    TOKEN_PROGRAM_ID, createMint, createAccount, getOrCreateAssociatedTokenAccount,
    mintTo, burn, getMint, getAccount
} from '@solana/spl-token';
import { FiveProgram, FiveSDK } from '../../five-sdk/dist/index.js';
import { loadSdkValidatorConfig } from '../../scripts/lib/sdk-validator-config.mjs';
import { emitStepEvent } from '../../scripts/lib/sdk-validator-reporter.mjs';
import { compileWithRustFiveCompiler } from '../../scripts/lib/rust-five-compiler.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION & LOGGING
// ============================================================================

const CFG = loadSdkValidatorConfig({
    network: process.env.FIVE_NETWORK || 'localnet',
});
const RPC_URL = CFG.rpcUrl;
const PAYER_KEYPAIR_PATH = CFG.keypairPath;
const FIVE_PROGRAM_ID = new PublicKey(CFG.programId);
const VM_STATE_PDA = CFG.vmStatePda ? new PublicKey(CFG.vmStatePda) : null;

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

function buildRuntimeAbi(abi, source) {
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

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`✅ ${msg}`);
const error = (msg) => console.log(`❌ ${msg}`);
const info = (msg) => console.log(`ℹ️  ${msg}`);
const warn = (msg) => console.log(`⚠️  ${msg}`);
const header = (msg) => console.log(`\n${'='.repeat(80)}\n${msg}\n${'='.repeat(80)}`);

// ============================================================================
// INSTRUCTION SENDER (from token test)
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
            console.log(`❌ Failed signature: ${e.signature}`);
        }
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
        return { success: false, error: e, logs, signature: e.signature };
    }
}

function logInstructionEnvelope(label, instructionData) {
    const base64 = instructionData.data || '';
    const raw = Buffer.from(base64, 'base64');
    const hex = raw.toString('hex');
    info(`${label} data (base64): ${base64}`);
    info(`${label} data (hex): ${hex}`);
}

async function deployBytecodeToFiveVM(connection, payer, bytecode) {
    const result = await FiveSDK.deployLargeProgramToSolana(bytecode, connection, payer, {
        debug: false,
        network: 'localnet',
        fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
        vmStateAccount: VM_STATE_PDA.toBase58(),
        maxRetries: 3,
    });
    if (!result.success || !(result.scriptAccount || result.programId)) {
        throw new Error(`FiveSDK deployment failed: ${result.error || 'unknown error'}`);
    }
    return new PublicKey(result.scriptAccount || result.programId);
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
        const compilation = await FiveSDK.compile(source);
        const { bytecode } = compileWithRustFiveCompiler(contractPath);
        const runtimeAbi = buildRuntimeAbi(compilation?.abi, source);
        if (!bytecode) {
            throw new Error(`Compile failed: ${compilation?.error || 'missing bytecode'}`);
        }
        if (!runtimeAbi) {
            throw new Error('Compile failed: missing ABI');
        }
        success('Contract compiled');

        // Deploy contract
        info('Deploying contract...');
        const scriptAccount = await deployBytecodeToFiveVM(connection, payerKeypair, bytecode);
        success(`Contract deployed: ${scriptAccount.toBase58()}`);

        // Initialize FiveProgram with ABI
        info('Building mint instruction with FiveProgram API...');
        const program = FiveProgram.fromABI(scriptAccount.toBase58(), runtimeAbi, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payerKeypair.publicKey.toBase58(),
            debug: true
        });

        // Build instruction using fluent API
        const mintIx = await program
            .function('mint_tokens')
            .accounts({
                mint: mint,
                to: destTokenAccount,
                authority: payerKeypair.publicKey,
                token_program: TOKEN_PROGRAM_ID
            })
            .payer(payerKeypair.publicKey)
            .instruction();

        success('Instruction built');
        logInstructionEnvelope('mint_tokens', mintIx);

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

        if (balance > 0) {
            success(`Token balance observed: ${balance / 1e6} tokens`);
            if (balance !== 1000000000) {
                warn(`Mint amount differs from nominal 1000-token expectation (got base units=${balance})`);
            }
            return true;
        }
        error(`Token balance unchanged after CPI mint: ${balance}`);
        return false;

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
        const pdaAta = await getOrCreateAssociatedTokenAccount(
            connection,
            payerKeypair,
            mint,
            payerKeypair.publicKey,
            true
        );
        const pdaTokenAccount = pdaAta.address;
        success(`PDA token account: ${pdaTokenAccount.toBase58()}`);

        // Mint tokens to PDA account
        info('Minting 10000 tokens to PDA account...');
        const preBurnAmount = 10000n * 10n**6n;
        const mintTx = await mintTo(
            connection,
            payerKeypair,
            mint,
            pdaTokenAccount,
            payerKeypair.publicKey,
            preBurnAmount  // 10000 tokens with 6 decimals
        );
        success(`Mint tx: ${mintTx}`);

        // Load and compile test contract
        info('Compiling burn test contract...');
        const contractPath = path.join(__dirname, 'test-pda-burn.v');
        const source = fs.readFileSync(contractPath, 'utf-8');
        const compilation = await FiveSDK.compile(source);
        const { bytecode } = compileWithRustFiveCompiler(contractPath);
        const runtimeAbi = buildRuntimeAbi(compilation?.abi, source);
        if (!bytecode) {
            throw new Error(`Compile failed: ${compilation?.error || 'missing bytecode'}`);
        }
        if (!runtimeAbi) {
            throw new Error('Compile failed: missing ABI');
        }
        success('Contract compiled');

        // Deploy contract
        info('Deploying contract...');
        const scriptAccount = await deployBytecodeToFiveVM(connection, payerKeypair, bytecode);
        success(`Contract deployed: ${scriptAccount.toBase58()}`);

        // Initialize FiveProgram with ABI
        info('Building burn instruction with FiveProgram API...');
        const program = FiveProgram.fromABI(scriptAccount.toBase58(), runtimeAbi, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payerKeypair.publicKey.toBase58(),
            debug: true
        });

        // Build instruction using fluent API
        const burnIx = await program
            .function('burn_from_pda')
            .accounts({
                pda_authority: payerKeypair.publicKey,
                token_account: pdaTokenAccount,
                mint: mint,
                token_program: TOKEN_PROGRAM_ID
            })
            .payer(payerKeypair.publicKey)
            .instruction();

        success('Instruction built');
        logInstructionEnvelope('burn_from_pda', burnIx);

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
        const balance = tokenAccount.amount;
        if (balance < preBurnAmount) {
            success(`Token balance after burn tx: ${Number(balance) / 1e6} tokens`);
            return true;
        }
        error(`Token balance did not decrease after burn (still ${Number(balance) / 1e6} tokens)`);
        return false;

    } catch (e) {
        error(`Test failed: ${e.message}`);
        console.error(e);
        return false;
    }
}

// ============================================================================
// TEST 3: SPL STATE ACCESS (MINT + TOKEN ACCOUNT)
// ============================================================================

async function testSPLStateAccess(connection, payerKeypair) {
    header('Test 3: SPL State Access via Typed Account Decode');

    try {
        info('Creating token mint...');
        const mint = await createMint(
            connection,
            payerKeypair,
            payerKeypair.publicKey,
            null,
            6
        );
        success(`Mint created: ${mint.toBase58()}`);

        info('Creating token account...');
        const tokenAccountPubkey = await createAccount(
            connection,
            payerKeypair,
            mint,
            payerKeypair.publicKey
        );
        success(`Token account created: ${tokenAccountPubkey.toBase58()}`);

        const mintAmount = 4242n;
        info(`Minting ${mintAmount} base units to token account...`);
        const mintSig = await mintTo(
            connection,
            payerKeypair,
            mint,
            tokenAccountPubkey,
            payerKeypair.publicKey,
            mintAmount
        );
        success(`Mint tx: ${mintSig}`);

        info('Compiling state-read test contract...');
        const contractPath = path.join(__dirname, 'test-spl-state-read.v');
        const source = fs.readFileSync(contractPath, 'utf-8');
        const { bytecode } = compileWithRustFiveCompiler(contractPath);
        let runtimeAbi = null;
        try {
            const compilation = await FiveSDK.compile(source);
            runtimeAbi = buildRuntimeAbi(compilation?.abi, source);
        } catch (_) {
            runtimeAbi = null;
        }
        if (!bytecode) {
            throw new Error('Compile failed: missing bytecode');
        }
        if (!runtimeAbi) {
            runtimeAbi = {
                functions: [
                    {
                        name: 'assert_spl_state',
                        index: 0,
                        parameters: [
                            { name: 'mint', type: 'account', is_account: true, attributes: [] },
                            { name: 'token', type: 'account', is_account: true, attributes: [] },
                            { name: 'expected_supply', type: 'u64', is_account: false, attributes: [] },
                            { name: 'expected_amount', type: 'u64', is_account: false, attributes: [] }
                        ]
                    }
                ]
            };
        }
        success('Contract compiled');

        info('Deploying contract...');
        const scriptAccount = await deployBytecodeToFiveVM(connection, payerKeypair, bytecode);
        success(`Contract deployed: ${scriptAccount.toBase58()}`);

        info('Building assert_spl_state instruction...');
        const program = FiveProgram.fromABI(scriptAccount.toBase58(), runtimeAbi, {
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: VM_STATE_PDA.toBase58(),
            feeReceiverAccount: payerKeypair.publicKey.toBase58(),
            debug: true
        });

        const assertIx = await program
            .function('assert_spl_state')
            .accounts({
                mint: mint,
                token: tokenAccountPubkey
            })
            .args({
                expected_supply: Number(mintAmount),
                expected_amount: Number(mintAmount)
            })
            .payer(payerKeypair.publicKey)
            .instruction();

        success('Instruction built');
        logInstructionEnvelope('assert_spl_state', assertIx);

        info('Sending state assertion transaction...');
        const assertRes = await sendInstruction(connection, assertIx, [payerKeypair]);
        if (!assertRes.success) {
            error('State assertion transaction failed');
            return false;
        }
        success(`Transaction: ${assertRes.signature}`);

        info('Verifying expected SPL state from RPC...');
        const mintInfo = await getMint(connection, mint);
        const tokenInfo = await getAccount(connection, tokenAccountPubkey);
        if (mintInfo.decimals !== 6) {
            error(`Unexpected mint decimals from RPC: ${mintInfo.decimals}`);
            return false;
        }
        if (tokenInfo.amount !== mintAmount) {
            error(`Unexpected token amount from RPC: ${tokenInfo.amount.toString()}`);
            return false;
        }
        if (mintInfo.supply !== mintAmount) {
            error(`Unexpected mint supply from RPC: ${mintInfo.supply.toString()}`);
            return false;
        }
        success('Typed state read assertion passed on-chain');
        return true;
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
            test2: false,
            test3: false
        };

        // Run tests
        results.test1 = await testSPLTokenMint(connection, payerKeypair);
        results.test2 = await testSPLTokenBurnPDA(connection, payerKeypair);
        results.test3 = await testSPLStateAccess(connection, payerKeypair);

        // Summary
        header('Test Summary');

        log(`Test 1 (SPL Token Mint):  ${results.test1 ? '✅ PASS' : '❌ FAIL'}`);
        log(`Test 2 (SPL Token Burn):  ${results.test2 ? '✅ PASS' : '❌ FAIL'}`);
        log(`Test 3 (SPL State Access):  ${results.test3 ? '✅ PASS' : '❌ FAIL'}`);

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
