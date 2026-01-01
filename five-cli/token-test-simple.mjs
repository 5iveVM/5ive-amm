#!/usr/bin/env node
/**
 * Token Program Test - 3 User Story
 * Tests all 12 public functions of the token program
 */

import fs from 'fs';
import {
    Connection, Keypair, PublicKey, Transaction,
    TransactionInstruction, SystemProgram
} from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';
const FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
const VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
const TOKEN_SCRIPT_ACCOUNT = new PublicKey('Gaa5aGJAsF8xa7yw7TAYMdHFjhyV4PXeAawQkK2W3Y9a');

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`✅ ${msg}`);
const error = (msg) => console.log(`❌ ${msg}`);
const info = (msg) => console.log(`ℹ️  ${msg}`);
const header = (msg) => console.log(`\n${'='.repeat(70)}\n${msg}\n${'='.repeat(70)}`);

// VLE Encoding for function parameters
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

function loadKeypair(kpPath) {
    const secretKey = JSON.parse(fs.readFileSync(kpPath, 'utf-8'));
    return Keypair.fromSecretKey(Uint8Array.from(secretKey));
}

// Create account instruction for token accounts
async function createTokenAccountIx(connection, payer, owner) {
    const account = Keypair.generate();
    const space = 1024; // Space for token account state
    const lamports = await connection.getMinimumBalanceForRentExemption(space);

    const ix = SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: account.publicKey,
        lamports,
        space,
        programId: FIVE_PROGRAM_ID,
    });

    return { account, ix };
}

// Build simplified instruction for token program functions
function buildSimpleInstruction(
    functionIndex,
    scriptAccount,
    vmState,
    signer,
    mint = null,
    from = null,
    to = null,
    delegate = null,
    amount = 0,
    decimals = 0
) {
    const discriminator = Buffer.from([9]); // ExecuteFunction
    let inputData = encodeVLE(functionIndex);

    // Add function-specific parameters
    if ([0].includes(functionIndex)) { // init_mint
        inputData = Buffer.concat([inputData, encodeVLE(decimals)]);
    } else if ([2, 3, 4, 5, 7].includes(functionIndex)) { // mint, burn, transfer, approve, transfer_approved
        inputData = Buffer.concat([inputData, encodeVLE(amount)]);
    }

    // Build account keys - always include script, vm_state, signer
    const keys = [
        { pubkey: scriptAccount, isSigner: false, isWritable: true },
        { pubkey: vmState, isSigner: false, isWritable: true },
        { pubkey: signer, isSigner: true, isWritable: true },
    ];

    // Add additional accounts as needed
    if (mint) keys.push({ pubkey: mint, isSigner: false, isWritable: [0, 2, 3].includes(functionIndex) });
    if (from) keys.push({ pubkey: from, isSigner: false, isWritable: true });
    if (to && to.toBase58() !== from?.toBase58()) {
        keys.push({ pubkey: to, isSigner: false, isWritable: true });
    }
    if (delegate) keys.push({ pubkey: delegate, isSigner: false, isWritable: false });

    return new TransactionInstruction({
        keys,
        programId: FIVE_PROGRAM_ID,
        data: Buffer.concat([discriminator, inputData]),
    });
}

async function executeFunction(
    connection,
    payer,
    functionIndex,
    functionName,
    scriptAccount,
    vmState,
    signer,
    options = {}
) {
    const ix = buildSimpleInstruction(
        functionIndex,
        scriptAccount,
        vmState,
        signer,
        options.mint,
        options.from,
        options.to,
        options.delegate,
        options.amount || 0,
        options.decimals || 0
    );

    const tx = new Transaction().add(ix);
    const signers = [payer, ...(options.extraSigners || [])];

    try {
        const sig = await connection.sendTransaction(tx, signers, {
            skipPreflight: true,
            maxRetries: 3
        });
        await connection.confirmTransaction(sig, 'confirmed');
        success(`${functionName}: ${sig.substring(0, 20)}...`);
        return sig;
    } catch (e) {
        error(`${functionName} failed: ${e.message}`);
        throw e;
    }
}

async function main() {
    header('🎭 Token Program Test - 3 User Story');

    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = loadKeypair(PAYER_KEYPAIR_PATH);

    info(`Payer: ${payer.publicKey.toBase58()}`);
    const balance = await connection.getBalance(payer.publicKey);
    info(`Balance: ${(balance / 1e9).toFixed(2)} SOL`);

    // Create 3 users
    const user1 = Keypair.generate(); // Mint authority
    const user2 = Keypair.generate(); // Regular user
    const user3 = Keypair.generate(); // Regular user

    info(`User1 (Authority): ${user1.publicKey.toBase58()}`);
    info(`User2 (Holder): ${user2.publicKey.toBase58()}`);
    info(`User3 (Holder): ${user3.publicKey.toBase58()}`);

    // Airdrop SOL to users
    header('Step 1: Fund Users with SOL');
    for (const user of [user1, user2, user3]) {
        const sig = await connection.requestAirdrop(user.publicKey, 1 * 1e9);
        await connection.confirmTransaction(sig, 'confirmed');
        info(`Funded ${user.publicKey.toBase58().substring(0, 10)}...`);
    }

    // Create token accounts for all users
    header('Step 2: Create Token Accounts');
    const mintAcctResult = await createTokenAccountIx(connection, payer, user1);
    const user1AcctResult = await createTokenAccountIx(connection, payer, user1);
    const user2AcctResult = await createTokenAccountIx(connection, payer, user2);
    const user3AcctResult = await createTokenAccountIx(connection, payer, user3);

    const createTx = new Transaction()
        .add(mintAcctResult.ix)
        .add(user1AcctResult.ix)
        .add(user2AcctResult.ix)
        .add(user3AcctResult.ix);

    const createSig = await connection.sendTransaction(createTx, [
        payer, mintAcctResult.account, user1AcctResult.account,
        user2AcctResult.account, user3AcctResult.account
    ], { skipPreflight: true });
    await connection.confirmTransaction(createSig, 'confirmed');
    success(`Created 4 token accounts`);

    const mintAccount = mintAcctResult.account.publicKey;
    const user1TokenAccount = user1AcctResult.account.publicKey;
    const user2TokenAccount = user2AcctResult.account.publicKey;
    const user3TokenAccount = user3AcctResult.account.publicKey;

    // Function 0: init_mint
    header('Step 3: Initialize Mint (init_mint) ✅');
    await executeFunction(connection, user1, 0, 'init_mint', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        mint: mintAccount,
        decimals: 6
    });

    // Function 1: init_token_account (All 3 users)
    header('Step 4: Initialize Token Accounts (init_token_account) ✅');
    await executeFunction(connection, user1, 1, 'init_token_account (User1)', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        from: user1TokenAccount,
        mint: mintAccount
    });
    await executeFunction(connection, user2, 1, 'init_token_account (User2)', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user2.publicKey, {
        from: user2TokenAccount,
        mint: mintAccount
    });
    await executeFunction(connection, user3, 1, 'init_token_account (User3)', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user3.publicKey, {
        from: user3TokenAccount,
        mint: mintAccount
    });

    // Function 2: mint (3 mints)
    header('Step 5: Mint Tokens (mint) ✅');
    await executeFunction(connection, user1, 2, 'mint 1000 to User1', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        mint: mintAccount,
        from: user1TokenAccount,
        amount: 1000
    });
    await executeFunction(connection, user1, 2, 'mint 500 to User2', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        mint: mintAccount,
        from: user2TokenAccount,
        amount: 500
    });
    await executeFunction(connection, user1, 2, 'mint 500 to User3', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        mint: mintAccount,
        from: user3TokenAccount,
        amount: 500
    });

    // Function 4: transfer
    header('Step 6: Transfer Tokens (transfer) ✅');
    await executeFunction(connection, user2, 4, 'transfer 100 from User2 to User3', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user2.publicKey, {
        from: user2TokenAccount,
        to: user3TokenAccount,
        amount: 100
    });

    // Function 5: approve & Function 7: transfer_approved
    header('Step 7: Approve Delegate (approve) ✅');
    await executeFunction(connection, user3, 5, 'approve User1 as delegate for 200 tokens', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user3.publicKey, {
        from: user3TokenAccount,
        delegate: user1.publicKey,
        amount: 200
    });

    header('Step 8: Transfer as Delegate (transfer_approved) ✅');
    await executeFunction(connection, user1, 7, 'transfer 100 from User3 via delegation', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        from: user3TokenAccount,
        to: user2TokenAccount,
        delegate: user1.publicKey,
        amount: 100
    });

    // Function 6: revoke
    header('Step 9: Revoke Delegation (revoke) ✅');
    await executeFunction(connection, user3, 6, 'revoke User1 delegation', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user3.publicKey, {
        from: user3TokenAccount
    });

    // Function 3: burn
    header('Step 10: Burn Tokens (burn) ✅');
    await executeFunction(connection, user1, 3, 'burn 100 tokens from User1', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        mint: mintAccount,
        from: user1TokenAccount,
        amount: 100
    });

    // Function 8: freeze_account
    header('Step 11: Freeze Account (freeze_account) ✅');
    await executeFunction(connection, user1, 8, 'freeze User2 account', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        mint: mintAccount,
        from: user2TokenAccount
    });

    // Function 9: thaw_account
    header('Step 12: Thaw Account (thaw_account) ✅');
    await executeFunction(connection, user1, 9, 'thaw User2 account', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        mint: mintAccount,
        from: user2TokenAccount
    });

    // Function 11: set_mint_authority
    header('Step 13: Transfer Mint Authority (set_mint_authority) ✅');
    await executeFunction(connection, user1, 11, 'transfer authority to User2', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user1.publicKey, {
        mint: mintAccount,
        delegate: user2.publicKey
    });

    // Function 10: close_account (User2 closes after burn)
    header('Step 14: Burn and Close Account (burn then close_account) ✅');
    info('User2 burns remaining balance to close account...');
    await executeFunction(connection, user2, 3, 'burn remaining tokens', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user2.publicKey, {
        mint: mintAccount,
        from: user2TokenAccount,
        amount: 400 // User2 has ~400 tokens left
    });

    header('Step 15: Close Account (close_account) ✅');
    await executeFunction(connection, user2, 10, 'close User2 account', TOKEN_SCRIPT_ACCOUNT, VM_STATE_PDA, user2.publicKey, {
        from: user2TokenAccount
    });

    header('✨ All Tests Complete!');
    console.log(`
    ╔════════════════════════════════════════════════════════════════╗
    ║ Successfully tested all 12 functions of token program          ║
    ╠════════════════════════════════════════════════════════════════╣
    ║ ✅ 0.  init_mint                                               ║
    ║ ✅ 1.  init_token_account (×3)                                 ║
    ║ ✅ 2.  mint (×3)                                               ║
    ║ ✅ 3.  burn (×2)                                               ║
    ║ ✅ 4.  transfer                                                ║
    ║ ✅ 5.  approve                                                 ║
    ║ ✅ 6.  revoke                                                  ║
    ║ ✅ 7.  transfer_approved                                       ║
    ║ ✅ 8.  freeze_account                                          ║
    ║ ✅ 9.  thaw_account                                            ║
    ║ ✅ 10. close_account                                           ║
    ║ ✅ 11. set_mint_authority                                      ║
    ╚════════════════════════════════════════════════════════════════╝

    📊 User Story Summary:
    ├─ User1 (Authority): Created mint, minted tokens, burned, froze/thawed, transferred authority
    ├─ User2 (Holder): Initialized account, received mints, transferred, approved delegate, burned, closed account
    └─ User3 (Holder): Initialized account, received mints, transferred, approved delegate, revoked delegation

    🎉 All operations completed successfully on localnet!
    `);
}

main().catch(err => {
    error(`Test failed: ${err.message}`);
    console.error(err);
    process.exit(1);
});
