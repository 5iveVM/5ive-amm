#!/usr/bin/env node
/**
 * Token Template E2E Test - 3 User Story using Five SDK
 *
 * Tests core token operations with 3 wallets using Five SDK for proper ABI-based instruction building:
 * - User 1: Token authority (creates mint, mints tokens)
 * - User 2: Regular user (receives tokens, transfers)
 * - User 3: Regular user (receives tokens, participates in delegation)
 *
 * Operations:
 * 1. Initialize mint with User1 as authority
 * 2. Create token accounts for all 3 users
 * 3. Mint tokens to each user
 * 4. Transfer tokens between users
 * 5. Approve delegation and transfer_from
 * 6. Revoke delegation
 * 7. Burn tokens
 * 8. Freeze/thaw accounts
 * 9. Disable mint/freeze authorities
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

// Localnet deployment (updated 2025-12-28)
let FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
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
        if (deploymentConfig.tokenScriptAccount) {
            TOKEN_SCRIPT_ACCOUNT = new PublicKey(deploymentConfig.tokenScriptAccount);
        }
        info('Loaded deployment-config.json overrides');
    } catch (configError) {
        warn(`Failed to load deployment-config.json: ${configError.message}`);
    }
}

// ============================================================================
// LOAD COMPILED TOKEN PROGRAM AND ABI
// ============================================================================

let tokenABI = null;
let functionIndices = {};  // Map function names to indices

function loadTokenABI() {
    const buildPath = path.join(__dirname, 'build', 'five-token-template.five');
    try {
        const fiveFile = JSON.parse(fs.readFileSync(buildPath, 'utf-8'));
        tokenABI = fiveFile.abi;
        const functionCount = Array.isArray(tokenABI?.functions)
            ? tokenABI.functions.length
            : Object.keys(tokenABI?.functions || {}).length;

        // Build function index lookup table
        // This works around an SDK bug where function index defaults to 0 on metadata errors
        if (Array.isArray(tokenABI?.functions)) {
            tokenABI.functions.forEach(f => {
                functionIndices[f.name] = f.index;
            });
        }

        info(`Loaded Token ABI: ${functionCount} functions`);
        return true;
    } catch (e) {
        error(`Failed to load token ABI from ${buildPath}: ${e.message}`);
        return false;
    }
}

// Get function index from name (workaround for SDK bug)
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
// NOTE: Accounts are NOT pre-created here.
// The @init constraint in Five DSL handles account creation via CPI.
// Just generate keypairs and pass them as signers.
// ============================================================================

// ============================================================================
// FIVE SDK INSTRUCTION EXECUTION
// ============================================================================

/**
 * Execute token function using Five SDK with proper ABI encoding.
 *
 * @param connection - Solana connection
 * @param payer - Keypair that pays for the transaction
 * @param functionName - Name of the function to call
 * @param parameters - Function parameters (values)
 * @param accounts - Array of account objects: { pubkey: PublicKey, isWritable: bool, isSigner: bool }
 * @param signers - Array of Keypair objects that must sign the transaction
 */
async function executeTokenFunction(
    connection,
    payer,
    functionName,
    parameters = [],
    accounts = [],
    signers = []
) {
    try {
        // Get function index from ABI (workaround for SDK bug that defaults to index 0)
        const functionIndex = getFunctionIndex(functionName);

        // Generate the base instruction using Five SDK (without accounts - we add them manually)
        const executeData = await FiveSDK.generateExecuteInstruction(
            TOKEN_SCRIPT_ACCOUNT.toBase58(),
            functionIndex,  // Pass index instead of name to avoid SDK bug
            parameters,
            [],  // Empty - we append accounts manually like the DEX test
            connection,
            {
                debug: true,
                vmStateAccount: VM_STATE_PDA.toBase58(),
                fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
                abi: tokenABI  // Pass ABI for proper parameter type encoding
            }
        );

        // Start with the base instruction keys (Script Account, VM State)
        const ixKeys = executeData.instruction.accounts.map((acc) => ({
            pubkey: new PublicKey(acc.pubkey),
            isSigner: acc.isSigner,
            isWritable: acc.isWritable
        }));

        // Append function-specific accounts with explicit flags
        // This follows the DEX test pattern where accounts are objects with pubkey/isSigner/isWritable
        accounts.forEach(acc => {
            const pubkey = acc.pubkey instanceof PublicKey
                ? acc.pubkey
                : new PublicKey(acc.pubkey);
            ixKeys.push({
                pubkey: pubkey,
                isSigner: acc.isSigner ?? false,
                isWritable: acc.isWritable ?? true
            });
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

        if (!success_flag) {
            console.log(`\n❌ Transaction Logs for [${functionName}]:`);
            if (txDetails?.meta?.logMessages) {
                txDetails.meta.logMessages.forEach(msg => console.log(`  ${msg}`));
            } else {
                console.log("  No logs available");
            }
            console.log(""); // Newline
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
    header('🎭 Token Template E2E Test - 3 User Story with Five SDK');

    // Load ABI
    if (!loadTokenABI()) {
        error('Cannot proceed without token ABI');
        process.exit(1);
    }

    // Connect
    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = loadKeypair(PAYER_KEYPAIR_PATH);

    // Display payer info
    const balance = await connection.getBalance(payer.publicKey);
    info(`Payer: ${payer.publicKey.toBase58()}`);
    info(`Payer Balance: ${(balance / LAMPORTS_PER_SOL).toFixed(2)} SOL`);

    // ========================================================================
    // SETUP: Create 3 Users
    // ========================================================================

    header('SETUP: Creating 3 Users');

    const user1 = Keypair.generate(); // Authority
    const user2 = Keypair.generate(); // Holder
    const user3 = Keypair.generate(); // Holder

    info(`User1 (Authority): ${user1.publicKey.toBase58()}`);
    info(`User2 (Holder):    ${user2.publicKey.toBase58()}`);
    info(`User3 (Holder):    ${user3.publicKey.toBase58()}`);

    // Fund users
    subheader('Funding users with SOL...');
    for (const user of [user1, user2, user3]) {
        const sig = await connection.requestAirdrop(user.publicKey, 10 * LAMPORTS_PER_SOL);
        await connection.confirmTransaction(sig, 'confirmed');
    }
    info('Funded User1');
    info('Funded User2');
    info('Funded User3');

    // ========================================================================
    // STEP 1: Generate Account Keypairs (NOT pre-created - @init handles creation)
    // ========================================================================

    header('STEP 1: Generating Account Keypairs');

    // Just generate keypairs - the @init constraint will create accounts via CPI
    const mintAccount = Keypair.generate();
    const user1TokenAccount = Keypair.generate();
    const user2TokenAccount = Keypair.generate();
    const user3TokenAccount = Keypair.generate();

    success('Generated keypairs for token accounts');
    info(`Mint Account:  ${mintAccount.publicKey.toBase58()}`);
    info(`User1 Account: ${user1TokenAccount.publicKey.toBase58()}`);
    info(`User2 Account: ${user2TokenAccount.publicKey.toBase58()}`);
    info(`User3 Account: ${user3TokenAccount.publicKey.toBase58()}`);

    // ========================================================================
    // STEP 2: Initialize Mint
    // ========================================================================

    header('STEP 2: Initialize Mint (init_mint)');

    // init_mint expects: mint_account (@init @mut @signer), authority (@signer)
    // Parameters: freeze_authority (pubkey), decimals, name, symbol, uri
    // The @init constraint will create the mint account via CPI
    let result = await executeTokenFunction(
        connection,
        payer,  // Payer funds the account creation
        'init_mint',
        [user1.publicKey.toBase58(), 6, "TestToken", "TEST", "https://example.com/token"],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: mintAccount.publicKey, isWritable: true, isSigner: true },
            { pubkey: user1.publicKey, isWritable: true, isSigner: true },
            { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },  // Required for @init CPI
            { pubkey: SYSVAR_RENT_PUBKEY, isWritable: false, isSigner: false }  // Rent sysvar for account initialization
        ],
        [user1, mintAccount]  // Both must sign: user1 as authority, mintAccount for creation
    );

    if (result.success) {
        success(`init_mint`);
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`init_mint failed: ${result.error}`);
    }

    // ========================================================================
    // STEP 3: Initialize Token Accounts
    // ========================================================================

    header('STEP 3: Initialize Token Accounts (init_token_account)');

    // init_token_account expects: token_account (@init @mut @signer), owner (@signer)
    // Parameters: mint pubkey
    // The @init constraint will create the token account via CPI
    const tokenAccounts = [
        { account: user1TokenAccount, user: user1, name: 'User1' },
        { account: user2TokenAccount, user: user2, name: 'User2' },
        { account: user3TokenAccount, user: user3, name: 'User3' }
    ];

    for (const { account, user, name } of tokenAccounts) {
        // init_token_account expects:
        //   Accounts: token_account (@init @mut @signer), owner (@signer)
        //   Params: mint (pubkey)
        result = await executeTokenFunction(
            connection,
            payer,  // Payer funds the account creation
            'init_token_account',
            [mintAccount.publicKey.toBase58()],  // mint (Param 2 -> Param 0 in VLE)
            [
                { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
                { pubkey: account.publicKey, isWritable: true, isSigner: true },
                { pubkey: user.publicKey, isWritable: true, isSigner: true }, // Owner (and Payer for funding)
                { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },  // Required for @init CPI
                { pubkey: SYSVAR_RENT_PUBKEY, isWritable: false, isSigner: false }  // Rent sysvar for account initialization
            ],
            [account, user]  // Both token account and owner must sign (owner is @signer in DSL)
        );

        if (result.success) {
            success(`init_token_account (${name})`);
            info(`  Signature: ${result.signature}`);
            info(`  Compute Units: ${result.computeUnits}`);
        } else {
            error(`init_token_account (${name}) failed: ${result.error}`);
        }
    }

    // ========================================================================
    // STEP 4: Mint Tokens
    // ========================================================================

    header('STEP 4: Mint Tokens (mint_to)');

    // mint_to expects: mint (@mut), dest_account (@mut), authority (@signer)
    const mintOps = [
        { account: user1TokenAccount, user: user1, name: 'User1', amount: 1000 },
        { account: user2TokenAccount, user: user2, name: 'User2', amount: 500 },
        { account: user3TokenAccount, user: user3, name: 'User3', amount: 500 }
    ];

    for (const op of mintOps) {
        result = await executeTokenFunction(
            connection,
            payer,
            'mint_to',
            [op.amount],
            [
                { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
                { pubkey: mintAccount.publicKey, isWritable: true, isSigner: false },
                { pubkey: op.account.publicKey, isWritable: true, isSigner: false },
                { pubkey: user1.publicKey, isWritable: false, isSigner: true }
            ],
            [user1]  // Authority signs
        );

        if (result.success) {
            success(`mint_to ${op.name} (${op.amount})`);
            info(`  Signature: ${result.signature}`);
            info(`  Compute Units: ${result.computeUnits}`);
        } else {
            error(`mint_to ${op.name} (${op.amount}) failed: ${result.error}`);
        }
    }

    // ========================================================================
    // STEP 5: Transfer Tokens
    // ========================================================================

    header('STEP 5: Transfer Tokens (transfer)');

    // transfer expects: source (@mut), dest (@mut), owner (@signer)
    result = await executeTokenFunction(
        connection,
        payer,
        'transfer',
        [100],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: user2TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user3TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user2.publicKey, isWritable: false, isSigner: true }
        ],
        [user2]  // Owner of source account signs
    );

    if (result.success) {
        success('transfer 100 from User2 to User3');
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`transfer 100 from User2 to User3 failed: ${result.error}`);
    }

    // ========================================================================
    // STEP 6: Approve Delegation and Transfer From
    // ========================================================================

    header('STEP 6: Approve Delegate (approve)');

    // approve expects: token_account (@mut), owner (@signer)
    // Parameters: delegate pubkey, amount
    result = await executeTokenFunction(
        connection,
        payer,
        'approve',
        [user2.publicKey.toBase58(), 150],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: user3TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user3.publicKey, isWritable: false, isSigner: true }
        ],
        [user3]  // Account owner signs
    );

    if (result.success) {
        success('approve User2 as delegate for 150 tokens');
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`approve User2 as delegate for 150 tokens failed: ${result.error}`);
    }

    subheader('Transfer as Delegate (transfer_from)');

    // transfer_from expects: source (@mut), dest (@mut), delegate (@signer)
    result = await executeTokenFunction(
        connection,
        payer,
        'transfer_from',
        [50],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: user3TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user1TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user2.publicKey, isWritable: false, isSigner: true }
        ],
        [user2]  // Delegate signs
    );

    if (result.success) {
        success('transfer_from 50 from User3 to User1 via delegation');
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`transfer_from 50 from User3 to User1 via delegation failed: ${result.error}`);
    }

    // ========================================================================
    // STEP 7: Revoke Delegation
    // ========================================================================

    header('STEP 7: Revoke Delegation (revoke)');

    // revoke expects: token_account (@mut), owner (@signer)
    result = await executeTokenFunction(
        connection,
        payer,
        'revoke',
        [],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: user3TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user3.publicKey, isWritable: false, isSigner: true }
        ],
        [user3]  // Account owner signs
    );

    if (result.success) {
        success('revoke User2 delegation');
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`revoke User2 delegation failed: ${result.error}`);
    }

    // ========================================================================
    // STEP 8: Burn Tokens
    // ========================================================================

    header('STEP 8: Burn Tokens (burn)');

    // burn expects: mint (@mut), token_account (@mut), owner (@signer)
    result = await executeTokenFunction(
        connection,
        payer,
        'burn',
        [100],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: mintAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user1TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user1.publicKey, isWritable: false, isSigner: true }
        ],
        [user1]  // Account owner signs
    );

    if (result.success) {
        success('burn 100 tokens');
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`burn 100 tokens failed: ${result.error}`);
    }

    // ========================================================================
    // STEP 9: Freeze and Thaw Accounts
    // ========================================================================

    header('STEP 9: Freeze Account (freeze_account)');

    // freeze_account expects: mint, token_account (@mut), freeze_authority (@signer)
    result = await executeTokenFunction(
        connection,
        payer,
        'freeze_account',
        [],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: mintAccount.publicKey, isWritable: false, isSigner: false },
            { pubkey: user2TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user1.publicKey, isWritable: false, isSigner: true }
        ],
        [user1]  // Freeze authority signs
    );

    if (result.success) {
        success('freeze User2 account');
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`freeze User2 account failed: ${result.error}`);
    }

    subheader('Thaw Account (thaw_account)');

    // thaw_account expects: mint, token_account (@mut), freeze_authority (@signer)
    result = await executeTokenFunction(
        connection,
        payer,
        'thaw_account',
        [],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: mintAccount.publicKey, isWritable: false, isSigner: false },
            { pubkey: user2TokenAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user1.publicKey, isWritable: false, isSigner: true }
        ],
        [user1]  // Freeze authority signs
    );

    if (result.success) {
        success('thaw User2 account');
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`thaw User2 account failed: ${result.error}`);
    }

    // ========================================================================
    // STEP 10: Disable Authorities
    // ========================================================================

    header('STEP 10: Disable Authorities');

    // disable_mint expects: mint (@mut), authority (@signer)
    result = await executeTokenFunction(
        connection,
        payer,
        'disable_mint',
        [],
        [
            { pubkey: payer.publicKey, isWritable: true, isSigner: true }, // Fee Payer & Admin
            { pubkey: mintAccount.publicKey, isWritable: true, isSigner: false },
            { pubkey: user1.publicKey, isWritable: false, isSigner: true }
        ],
        [user1]  // Authority signs
    );

    if (result.success) {
        success('disable_mint - permanently disable minting');
        info(`  Signature: ${result.signature}`);
        info(`  Compute Units: ${result.computeUnits}`);
    } else {
        error(`disable_mint failed: ${result.error}`);
    }

    // ========================================================================
    // TEST SUMMARY
    // ========================================================================

    header('📊 Test Execution Complete');
    info('All token operations executed with Five SDK ABI encoding');

    // ========================================================================
    // EXPORT STATE FOR VERIFICATION
    // ========================================================================
    const testState = {
        config: {
            rpcUrl: RPC_URL,
            programId: FIVE_PROGRAM_ID.toBase58(),
            vmStatePda: VM_STATE_PDA.toBase58()
        },
        accounts: {
            mint: mintAccount.publicKey.toBase58(),
            user1: user1.publicKey.toBase58(),
            user2: user2.publicKey.toBase58(),
            user3: user3.publicKey.toBase58(),
            payer: payer.publicKey.toBase58(),
            user1TokenAccount: user1TokenAccount.publicKey.toBase58(),
            user2TokenAccount: user2TokenAccount.publicKey.toBase58(),
            user3TokenAccount: user3TokenAccount.publicKey.toBase58()
        },
        expected: {
            mintSupply: 1000 + 500 + 500 - 100, // 1900
            user1Balance: 1000 + 50 - 100,      // 950 (minted 1000, received 50 from user3 via user2, burned 100)
            user2Balance: 500 - 100,            // 400 (minted 500, sent 100 to user3)
            user3Balance: 500 + 100 - 50,       // 550 (minted 500, received 100 from user2, sent 50 to user1 via delegate)
            tokenName: "TestToken",
            tokenSymbol: "TEST"
        }
    };

    fs.writeFileSync(path.join(__dirname, 'test-state.json'), JSON.stringify(testState, null, 2));
    success('Test state saved to test-state.json');
}

// ============================================================================
// RUN TEST
// ============================================================================

main().catch(err => {
    error(err.message);
    process.exit(1);
});
