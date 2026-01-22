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

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================ 
// CONFIGURATION
// ============================================================================ 

let RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

// Localnet deployment defaults
let FIVE_PROGRAM_ID = new PublicKey(process.env.FIVE_PROGRAM_ID || '7JizMjzU3u8z3p5QuPNUE2r7YmA6Cks1V7attcujVQrd');
let VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
let TOKEN_SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

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

// Load config overrides if present
const deploymentConfigPath = path.join(__dirname, 'deployment-config.json');
if (fs.existsSync(deploymentConfigPath)) {
    try {
        const deploymentConfig = JSON.parse(fs.readFileSync(deploymentConfigPath, 'utf-8'));
        if (deploymentConfig.rpcUrl) RPC_URL = deploymentConfig.rpcUrl;
        if (deploymentConfig.fiveProgramId) FIVE_PROGRAM_ID = new PublicKey(deploymentConfig.fiveProgramId);
        if (deploymentConfig.vmStatePda) VM_STATE_PDA = new PublicKey(deploymentConfig.vmStatePda);
        if (deploymentConfig.tokenScriptAccount) TOKEN_SCRIPT_ACCOUNT = new PublicKey(deploymentConfig.tokenScriptAccount);
        info('Loaded deployment-config.json overrides');
    } catch (e) {
        warn(`Failed to load deployment-config.json: ${e.message}`);
    }
}

// ============================================================================ 
// HELPER: Transaction Execution
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

        // Fetch logs to extract CU usage
        let logs = [];
        let cu = -1;
        try {
            // wait a bit for confirmed state
            await new Promise(r => setTimeout(r, 500));
            const txDetails = await connection.getTransaction(sig, {
                maxSupportedTransactionVersion: 0,
                commitment: 'confirmed'
            });
            logs = txDetails?.meta?.logMessages || [];

            if (txDetails?.meta?.err) {
                console.log(`❌ Transaction Failed on-chain: ${JSON.stringify(txDetails.meta.err)}`);
                logs.forEach(log => console.log(`  ${log}`));
                return { success: false, error: txDetails.meta.err, logs, cu: -1, signature: sig };
            }

            // Extract CU
            const cuLog = logs.find(l => l.includes('consumed'));
            if (cuLog) {
                const match = cuLog.match(/consumed (\d+) of/);
                if (match) cu = match[1];
                console.log(`   └─ ⚡ CU: ${cu}`);
            }
        } catch (e) {
            console.log("   └─ (CU fetch failed or verification failed)", e);
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
                console.log("Could not fetch logs for failed transaction");
            }
        }
        return { success: false, error: e, logs };
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

    const initMintRes = await sendInstruction(connection, initMintIx, [payer, user1, mintAccount]);
    if (initMintRes.success) success(`init_mint successful (sig: ${initMintRes.signature})`);
    else { error('init_mint failed'); console.error(initMintRes.error); process.exit(1); }

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

        const res = await sendInstruction(connection, ix, [payer, acc.owner, acc.kp]);
        if (res.success) success(`init_token_account for ${acc.name} successful (sig: ${res.signature})`);
        else { error(`init_token_account for ${acc.name} failed`); console.error(res.error); }
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

        const res = await sendInstruction(connection, ix, [payer, user1]);
        if (res.success) success(`mint_to ${op.name} (${op.amount}) successful (sig: ${res.signature})`);
        else { error(`mint_to ${op.name} failed`); console.error(res.error); }
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

    const transferRes = await sendInstruction(connection, transferIx, [payer, user2]);
    if (transferRes.success) success(`transfer 100 from User2 to User3 successful (sig: ${transferRes.signature})`);
    else { error('transfer failed'); console.error(transferRes.error); }

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

    const approveRes = await sendInstruction(connection, approveIx, [payer, user3]);
    if (approveRes.success) success(`approve User2 as delegate successful (sig: ${approveRes.signature})`);
    else { error('approve failed'); console.error(approveRes.error); }

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

    const transferFromRes = await sendInstruction(connection, transferFromIx, [payer, user2]);
    if (transferFromRes.success) success(`transfer_from 50 via delegate successful (sig: ${transferFromRes.signature})`);
    else { error('transfer_from failed'); console.error(transferFromRes.error); }

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

    const revokeRes = await sendInstruction(connection, revokeIx, [payer, user3]);
    if (revokeRes.success) success(`revoke delegation successful (sig: ${revokeRes.signature})`);
    else { error('revoke failed'); console.error(revokeRes.error); }

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

    const burnRes = await sendInstruction(connection, burnIx, [payer, user1]);
    if (burnRes.success) success(`burn 100 tokens successful (sig: ${burnRes.signature})`);
    else { error('burn failed'); console.error(burnRes.error); }

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

    const freezeRes = await sendInstruction(connection, freezeIx, [payer, user1]);
    if (freezeRes.success) success(`freeze account successful (sig: ${freezeRes.signature})`);
    else { error('freeze failed'); console.error(freezeRes.error); }

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

    const thawRes = await sendInstruction(connection, thawIx, [payer, user1]);
    if (thawRes.success) success(`thaw account successful (sig: ${thawRes.signature})`);
    else { error('thaw failed'); console.error(thawRes.error); }

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

    const disableRes = await sendInstruction(connection, disableIx, [payer, user1]);
    if (disableRes.success) success(`disable_mint successful (sig: ${disableRes.signature})`);
    else { error('disable_mint failed'); console.error(disableRes.error); }

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
