#!/usr/bin/env node
/**
 * Token Template E2E Test - Manual Instruction Building
 *
 * This version bypasses Five SDK and builds instructions manually
 * to correctly handle the ABI parameter types.
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    TransactionInstruction, SystemProgram, LAMPORTS_PER_SOL
} from '@solana/web3.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION
// ============================================================================

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

const FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
const VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
const TOKEN_SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

// ============================================================================
// LOGGING
// ============================================================================

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`✅ ${msg}`);
const error = (msg) => console.log(`❌ ${msg}`);
const info = (msg) => console.log(`ℹ️  ${msg}`);
const header = (msg) => console.log(`\n${'='.repeat(80)}\n${msg}\n${'='.repeat(80)}`);

// ============================================================================
// VLE ENCODING
// ============================================================================

function encodeVLE(value) {
    const buffer = [];
    let val = BigInt(value);
    if (val === 0n) return Buffer.from([0]);
    while (val > 0n) {
        let byte = Number(val & 0x7Fn);
        val >>= 7n;
        if (val > 0n) byte |= 0x80;
        buffer.push(byte);
    }
    return Buffer.from(buffer);
}

function encodeString(str) {
    const strBytes = Buffer.from(str, 'utf-8');
    return Buffer.concat([encodeVLE(strBytes.length), strBytes]);
}

function encodePubkey(pubkeyStr) {
    // Convert base58 pubkey to 32 bytes using PublicKey
    const pk = typeof pubkeyStr === 'string' ? new PublicKey(pubkeyStr) : pubkeyStr;
    return Buffer.from(pk.toBytes());
}

// ============================================================================
// LOAD ABI
// ============================================================================

let tokenABI = null;

function loadABI() {
    const fiveFile = JSON.parse(fs.readFileSync(
        path.join(__dirname, 'build', 'five-token-template.five'), 'utf-8'
    ));

    // Build function index lookup
    const functions = {};
    for (const fn of fiveFile.abi.functions) {
        functions[fn.name] = fn;
    }
    tokenABI = functions;
    info(`Loaded ABI with ${Object.keys(functions).length} functions`);
}

// ============================================================================
// MANUAL INSTRUCTION BUILDER
// ============================================================================

function buildExecuteInstruction(
    functionName,
    params,        // Object with parameter values keyed by name
    accounts       // Array of {pubkey, isSigner, isWritable}
) {
    const fn = tokenABI[functionName];
    if (!fn) throw new Error(`Unknown function: ${functionName}`);

    const discriminator = Buffer.from([9]); // ExecuteFunction
    const functionIndex = encodeVLE(fn.index);

    // Encode non-account parameters
    const dataParams = [];
    for (const param of fn.parameters) {
        if (param.is_account) continue; // Skip account parameters

        const value = params[param.name];
        if (value === undefined) {
            throw new Error(`Missing parameter: ${param.name}`);
        }

        switch (param.param_type) {
            case 'u8':
                dataParams.push(Buffer.from([value]));
                break;
            case 'u64':
                dataParams.push(encodeVLE(value));
                break;
            case 'pubkey':
                if (typeof value === 'string') {
                    dataParams.push(encodePubkey(value));
                } else {
                    dataParams.push(encodePubkey(value.toBase58()));
                }
                break;
            case 'string':
                dataParams.push(encodeString(value));
                break;
            case 'bool':
                dataParams.push(Buffer.from([value ? 1 : 0]));
                break;
            default:
                throw new Error(`Unknown param type: ${param.param_type}`);
        }
    }

    // Combine: discriminator + function_index + param_count + params
    const paramCount = encodeVLE(dataParams.length);
    const instructionData = Buffer.concat([
        discriminator,
        functionIndex,
        paramCount,
        ...dataParams
    ]);

    // Build account keys: script, vm_state, then function accounts
    const keys = [
        { pubkey: TOKEN_SCRIPT_ACCOUNT, isSigner: false, isWritable: true },
        { pubkey: VM_STATE_PDA, isSigner: false, isWritable: true },
        ...accounts
    ];

    return new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys,
        data: instructionData
    });
}

// ============================================================================
// HELPERS
// ============================================================================

function loadKeypair(kpPath) {
    const secretKey = JSON.parse(fs.readFileSync(kpPath, 'utf-8'));
    return Keypair.fromSecretKey(Uint8Array.from(secretKey));
}

async function createTokenAccount(connection, payer) {
    const account = Keypair.generate();
    const space = 1024;
    const lamports = await connection.getMinimumBalanceForRentExemption(space);

    const ix = SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: account.publicKey,
        lamports,
        space,
        programId: FIVE_PROGRAM_ID,
    });

    const tx = new Transaction().add(ix);
    const sig = await connection.sendTransaction(tx, [payer, account], { skipPreflight: true });
    await connection.confirmTransaction(sig, 'confirmed');
    return account;
}

async function executeFunction(connection, functionName, params, accounts, signers) {
    try {
        const ix = buildExecuteInstruction(functionName, params, accounts);

        console.log(`[${functionName}] Instruction data (${ix.data.length} bytes): ${ix.data.toString('hex').slice(0, 60)}...`);
        console.log(`[${functionName}] Accounts: ${ix.keys.length}`);

        const tx = new Transaction().add(ix);
        const sig = await connection.sendTransaction(tx, signers, { skipPreflight: true });
        await connection.confirmTransaction(sig, 'confirmed');

        const txDetails = await connection.getTransaction(sig, { maxSupportedTransactionVersion: 0 });
        const err = txDetails?.meta?.err;

        if (err) {
            return { success: false, error: JSON.stringify(err), signature: sig };
        }

        return {
            success: true,
            signature: sig,
            computeUnits: txDetails?.meta?.computeUnitsConsumed || 0
        };
    } catch (e) {
        return { success: false, error: e.message };
    }
}

// ============================================================================
// MAIN
// ============================================================================

async function main() {
    header('🎭 Token Template E2E Test - Manual Instruction Building');

    loadABI();

    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = loadKeypair(PAYER_KEYPAIR_PATH);

    const balance = await connection.getBalance(payer.publicKey);
    info(`Payer: ${payer.publicKey.toBase58()}`);
    info(`Balance: ${(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);

    // Create users
    header('SETUP: Creating Users');
    const user1 = Keypair.generate();
    const user2 = Keypair.generate();

    info(`User1 (Authority): ${user1.publicKey.toBase58()}`);
    info(`User2 (Holder):    ${user2.publicKey.toBase58()}`);

    // Fund users
    for (const user of [user1, user2]) {
        const sig = await connection.requestAirdrop(user.publicKey, 10 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig, 'confirmed');
    }
    info('Funded users');

    // Create accounts
    header('STEP 1: Creating Token Accounts');
    const mintAccount = await createTokenAccount(connection, payer);
    const user1TokenAccount = await createTokenAccount(connection, payer);
    const user2TokenAccount = await createTokenAccount(connection, payer);

    success('Created all token accounts');
    info(`Mint Account:  ${mintAccount.publicKey.toBase58()}`);
    info(`User1 Account: ${user1TokenAccount.publicKey.toBase58()}`);
    info(`User2 Account: ${user2TokenAccount.publicKey.toBase58()}`);

    // Test init_mint
    header('STEP 2: Initialize Mint (init_mint)');

    let result = await executeFunction(
        connection,
        'init_mint',
        {
            freeze_authority: user1.publicKey.toBase58(),
            decimals: 6,
            name: "TestToken",
            symbol: "TEST",
            uri: "https://example.com"
        },
        [
            { pubkey: mintAccount.publicKey, isSigner: false, isWritable: true },
            { pubkey: user1.publicKey, isSigner: true, isWritable: false }
        ],
        [payer, user1]
    );

    if (result.success) {
        success(`init_mint - Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`init_mint failed: ${result.error}`);
    }

    // Test init_token_account
    header('STEP 3: Initialize Token Account (init_token_account)');

    result = await executeFunction(
        connection,
        'init_token_account',
        {
            owner: user1.publicKey.toBase58(),
            mint: mintAccount.publicKey.toBase58()
        },
        [
            { pubkey: user1TokenAccount.publicKey, isSigner: false, isWritable: true }
        ],
        [payer, user1]
    );

    if (result.success) {
        success(`init_token_account - Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`init_token_account failed: ${result.error}`);
    }

    // Test mint_to
    header('STEP 4: Mint Tokens (mint_to)');

    result = await executeFunction(
        connection,
        'mint_to',
        {
            amount: 1000
        },
        [
            { pubkey: mintAccount.publicKey, isSigner: false, isWritable: true },
            { pubkey: user1TokenAccount.publicKey, isSigner: false, isWritable: true },
            { pubkey: user1.publicKey, isSigner: true, isWritable: false }
        ],
        [payer, user1]
    );

    if (result.success) {
        success(`mint_to - Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`mint_to failed: ${result.error}`);
    }

    header('📊 Test Complete');
}

main().catch(err => {
    error(err.message);
    console.error(err.stack);
    process.exit(1);
});
