
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

let RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

// Localnet deployment
let FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
let VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
let TOKEN_SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

const log = (msg) => console.log(msg);
const success = (msg) => console.log(`✅ ${msg}`);
const error = (msg) => console.log(`❌ ${msg}`);
const info = (msg) => console.log(`ℹ️  ${msg}`);

const deploymentConfigPath = path.join(__dirname, 'deployment-config.json');
if (fs.existsSync(deploymentConfigPath)) {
    try {
        const deploymentConfig = JSON.parse(fs.readFileSync(deploymentConfigPath, 'utf-8'));
        if (deploymentConfig.rpcUrl) RPC_URL = deploymentConfig.rpcUrl;
        if (deploymentConfig.fiveProgramId) FIVE_PROGRAM_ID = new PublicKey(deploymentConfig.fiveProgramId);
        if (deploymentConfig.vmStatePda) VM_STATE_PDA = new PublicKey(deploymentConfig.vmStatePda);
        if (deploymentConfig.tokenScriptAccount) TOKEN_SCRIPT_ACCOUNT = new PublicKey(deploymentConfig.tokenScriptAccount);
        info('Loaded deployment-config.json overrides');
    } catch (configError) {
        console.log(`Failed to load deployment-config.json: ${configError.message}`);
    }
}

let tokenABI = null;
let functionIndices = {};

function loadTokenABI() {
    const buildPath = path.join(__dirname, 'build', 'five-token-template.five');
    try {
        const fiveFile = JSON.parse(fs.readFileSync(buildPath, 'utf-8'));
        tokenABI = fiveFile.abi;
        if (Array.isArray(tokenABI?.functions)) {
            tokenABI.functions.forEach(f => {
                functionIndices[f.name] = f.index;
            });
        }
        return true;
    } catch (e) {
        error(`Failed to load token ABI: ${e.message}`);
        return false;
    }
}

function getFunctionIndex(functionName) {
    const index = functionIndices[functionName];
    if (index === undefined) throw new Error(`Unknown function: ${functionName}`);
    return index;
}

function loadKeypair(kpPath) {
    const secretKey = JSON.parse(fs.readFileSync(kpPath, 'utf-8'));
    return Keypair.fromSecretKey(Uint8Array.from(secretKey));
}

async function executeTokenFunction(connection, payer, functionName, parameters = [], accounts = [], signers = []) {
    try {
        const functionIndex = getFunctionIndex(functionName);
        const executeData = await FiveSDK.generateExecuteInstruction(
            TOKEN_SCRIPT_ACCOUNT.toBase58(),
            functionIndex,
            parameters,
            accounts.map(a => a.pubkey.toBase58()),
            connection,
            {
                debug: true,
                vmStateAccount: VM_STATE_PDA.toBase58(),
                fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
                abi: tokenABI
            }
        );

        const ixKeys = executeData.instruction.accounts.map((acc) => ({
            pubkey: new PublicKey(acc.pubkey),
            isSigner: acc.isSigner,
            isWritable: acc.isWritable
        }));

        const ix = new TransactionInstruction({
            programId: new PublicKey(executeData.instruction.programId),
            keys: ixKeys,
            data: Buffer.from(executeData.instruction.data, 'base64')
        });

        console.log("Transaction Keys:");
        const keyLog = ixKeys.map((k, i) => `  [${i}] ${k.pubkey.toBase58()} (Signer: ${k.isSigner})`).join('\n');
        console.log(keyLog);
        fs.writeFileSync('debug_keys.txt', keyLog);


        const tx = new Transaction().add(ix);
        const allSigners = [payer, ...signers];

        const sig = await connection.sendTransaction(tx, allSigners, { skipPreflight: false });
        await connection.confirmTransaction(sig, 'confirmed');

        const txDetails = await connection.getTransaction(sig, { maxSupportedTransactionVersion: 0 });

        if (txDetails?.meta?.err) {
            console.log(`\n❌ Transaction Logs for [${functionName}]:`);
            if (txDetails?.meta?.logMessages) {
                txDetails.meta.logMessages.forEach(msg => console.log(`  ${msg}`));
            }
            return { success: false, error: JSON.stringify(txDetails.meta.err) };
        }

        return { success: true, signature: sig, computeUnits: txDetails.meta.computeUnitsConsumed };
    } catch (e) {
        return { success: false, error: e.message };
    }
}

async function main() {
    if (!loadTokenABI()) process.exit(1);

    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = loadKeypair(PAYER_KEYPAIR_PATH);
    const user1 = Keypair.generate();

    // Fund user1
    const sig = await connection.requestAirdrop(user1.publicKey, 1 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, 'confirmed');

    const balance = await connection.getBalance(user1.publicKey);
    info(`User1 Balance: ${balance / LAMPORTS_PER_SOL} SOL`);

    const mintAccount = Keypair.generate();

    info(`Mint Account: ${mintAccount.publicKey.toBase58()}`);

    const result = await executeTokenFunction(
        connection,
        payer,
        'init_mint',
        [mintAccount.publicKey, user1.publicKey, user1.publicKey, 6, "TestToken", "TEST", "https://example.com"],
        [
            { pubkey: mintAccount.publicKey, isWritable: true, isSigner: true },
            { pubkey: user1.publicKey, isWritable: true, isSigner: true },
            { pubkey: SystemProgram.programId, isWritable: false, isSigner: false },
            { pubkey: SYSVAR_RENT_PUBKEY, isWritable: false, isSigner: false }
        ],
        [user1, mintAccount]
    );

    if (result.success) {
        success('init_mint successful');
    } else {
        error(`init_mint failed: ${result.error}`);
        process.exit(1);
    }
}

main();
