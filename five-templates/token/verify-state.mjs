import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    TransactionInstruction, SystemProgram, LAMPORTS_PER_SOL
} from '@solana/web3.js';
import { FiveSDK } from '../../five-sdk/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// ============================================================================
// CONFIGURATION
// ============================================================================

let RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

// Localnet deployment config
let FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
let VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
let TOKEN_SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

const log = (msg) => console.log(msg);
const error = (msg) => console.error(`❌ ${msg}`);

// Load config from deployment-config.json if it exists
const deploymentConfigPath = path.join(__dirname, 'deployment-config.json');
if (fs.existsSync(deploymentConfigPath)) {
    try {
        const deploymentConfig = JSON.parse(fs.readFileSync(deploymentConfigPath, 'utf-8'));
        if (deploymentConfig.rpcUrl) RPC_URL = deploymentConfig.rpcUrl;
        if (deploymentConfig.fiveProgramId) FIVE_PROGRAM_ID = new PublicKey(deploymentConfig.fiveProgramId);
        if (deploymentConfig.vmStatePda) VM_STATE_PDA = new PublicKey(deploymentConfig.vmStatePda);
        if (deploymentConfig.tokenScriptAccount) TOKEN_SCRIPT_ACCOUNT = new PublicKey(deploymentConfig.tokenScriptAccount);
    } catch (e) {
        // ignore
    }
}

let tokenABI = null;
function loadTokenABI() {
    const buildPath = path.join(__dirname, 'build', 'five-token-template.five');
    try {
        const fiveFile = JSON.parse(fs.readFileSync(buildPath, 'utf-8'));
        tokenABI = fiveFile.abi;
        return true;
    } catch (e) {
        error(`Failed to load token ABI: ${e.message}`);
        return false;
    }
}

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

async function executeTokenFunction(connection, payer, functionName, parameters = [], accounts = [], extraSigners = []) {
    // ... Simplified execution logic ...
    const accountStrings = accounts.map(a => {
        if (typeof a === 'string') return a;
        if (a && typeof a.toBase58 === 'function') return a.toBase58();
        return String(a);
    });

    const executeData = await FiveSDK.generateExecuteInstruction(
        TOKEN_SCRIPT_ACCOUNT.toBase58(),
        functionName,
        parameters,
        accountStrings,
        connection,
        {
            debug: true,
            vmStateAccount: VM_STATE_PDA.toBase58(),
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            abi: tokenABI
        }
    );

    const keys = executeData.instruction.accounts.map((acc, i) => {
        let isSigner = acc.isSigner;
        let isWritable = acc.isWritable;

        // Ensure payer is signer
        if (acc.pubkey === payer.publicKey.toBase58()) isSigner = true;

        // Ensure extraSigners are signers
        for (const signer of extraSigners) {
            if (acc.pubkey === signer.publicKey.toBase58()) {
                isSigner = true;
            }
        }

        return {
            pubkey: new PublicKey(acc.pubkey),
            isSigner,
            isWritable
        };
    });

    const ix = new TransactionInstruction({
        programId: new PublicKey(executeData.instruction.programId),
        keys,
        data: Buffer.from(executeData.instruction.data, 'base64')
    });

    const tx = new Transaction().add(ix);
    const allSigners = [payer, ...extraSigners];
    console.log("Sending transaction for " + functionName + "...");
    const sig = await connection.sendTransaction(tx, allSigners, { skipPreflight: false });
    console.log("Transaction sent: " + sig + ". Confirming...");
    await connection.confirmTransaction(sig, 'confirmed');
    console.log("Transaction confirmed.");
    return sig;
}

async function main() {
    console.log("Starting verification...");
    if (!loadTokenABI()) process.exit(1);

    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = loadKeypair(PAYER_KEYPAIR_PATH);
    console.log(`Payer: ${payer.publicKey.toBase58()}`);

    // Create Mint Account
    const mintAccount = await createTokenAccount(connection, payer);
    console.log(`Mint Account: ${mintAccount.publicKey.toBase58()}`);

    // Init Mint
    const authority = payer.publicKey;
    const decimals = 9;
    const name = "VerifToken";
    const symbol = "VRF";
    const uri = "http://verif";
    // Debug: Log function names
    console.log("ABI Functions:", tokenABI.functions.map(f => f.name));

    // PATCH ABI: Convert 'Account' parameters to 'pubkey' parameters for init_mint
    // This aligns with the compiler's behavior of expecting keys as params,
    // and forces the SDK to encode them as parameters (Type 10 Pubkey) instead of skipping them.
    const initMintFunc = tokenABI.functions.find(f => f.name === 'init_mint');
    if (initMintFunc) {
        console.log("Patching init_mint parameters...");
        // Param 0: mint_account (Account -> pubkey)
        initMintFunc.parameters[0].param_type = 'pubkey';
        initMintFunc.parameters[0].is_account = false;

        // Param 1: authority (Account -> pubkey)
        initMintFunc.parameters[1].param_type = 'pubkey';
        initMintFunc.parameters[1].is_account = false;

        // Param 2: freeze_authority is already pubkey
    }

    console.log("Initializing mint with patched ABI...");
    await executeTokenFunction(
        connection,
        payer,
        'init_mint',
        [
            mintAccount.publicKey.toBase58(), // Param 0
            authority.toBase58(),             // Param 1
            authority.toBase58(),             // Param 2 (freeze)
            decimals,
            name,
            symbol,
            uri
        ],
        [mintAccount.publicKey, authority], // Still pass accounts for valid transaction keys
        []
    );

    console.log("Mint initialized. Fetching account data...");

    const accountInfo = await connection.getAccountInfo(mintAccount.publicKey);
    if (!accountInfo) {
        console.error("Account not found!");
        return;
    }

    const data = accountInfo.data;
    console.log(`Data length: ${data.length}`);
    console.log("Data (Hex debug):");

    // Print first 256 bytes hex dump
    const hex = data.slice(0, 256).toString('hex');
    for (let i = 0; i < hex.length; i += 32) {
        console.log(hex.substring(i, i + 32));
    }

    // Check for "all ones" pattern
    // The user said "1024 bytes of 1s".
    // Init mint should set some zeros (e.g. supply = 0)
    // and store metadata.

    // Quick heuristic check:
    // If we see 01010101 everywhere it's bad.
    // authority should be 32 bytes (likely payer key)
    // supply u64 (0) -> 8 bytes of 00
    // decimals u8 (9) -> 09

    // We expect to see some 00s.
    let zeroCount = 0;
    for (let b of data) {
        if (b === 0) zeroCount++;
    }
    console.log(`Zero byte count: ${zeroCount}`);

    let oneCount = 0;
    for (let b of data) {
        if (b === 1) oneCount++;
    }
    console.log(`One (0x01) byte count: ${oneCount}`);

    if (oneCount > 900) { // If mostly 1s
        console.error("FAIL: Account data appears to be filled with 1s.");
    } else {
        console.log("PASS: Account data does not look like all 1s.");

        // Try to find the name "VerifToken"
        const nameBuf = Buffer.from(name);
        if (data.includes(nameBuf)) {
            console.log("PASS: Found token name in data.");
        } else {
            console.error("FAIL: Token name not found in data.");
        }

    }
}

main().catch(e => console.error(e));
