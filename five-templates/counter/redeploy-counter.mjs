#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    SystemProgram, SYSVAR_RENT_PUBKEY, TransactionInstruction
} from '@solana/web3.js';
import { FiveSDK } from '../../five-sdk/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

const deploymentConfigPath = path.join(__dirname, 'deployment-config.json');
let config = {};
if (fs.existsSync(deploymentConfigPath)) {
    config = JSON.parse(fs.readFileSync(deploymentConfigPath, 'utf-8'));
}

// Load bytecode
const bytecodeFile = path.join(__dirname, 'build', 'five-counter-template.five');
const fiveFile = JSON.parse(fs.readFileSync(bytecodeFile, 'utf-8'));
const bytecodeBase64 = fiveFile.bytecode;
const bytecode = Buffer.from(bytecodeBase64, 'base64');

console.log(`Redeploying counter script...`);
console.log(`Bytecode size: ${bytecode.length} bytes`);

const connection = new Connection(RPC_URL, 'confirmed');
const payer = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8')))
);

const scriptAccount = Keypair.generate();
console.log(`New script account: ${scriptAccount.publicKey.toBase58()}`);

async function deploy() {
    try {
        // Create the script account
        const lamportsRequired = 5000000; // Adjust based on bytecode size
        const createIx = SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: scriptAccount.publicKey,
            lamports: lamportsRequired,
            space: bytecode.length + 1024, // Add buffer for metadata
            programId: new PublicKey(config.fiveProgramId || 'HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg')
        });

        // Create deployment instruction using FiveSDK
        const deployIx = await FiveSDK.generateDeployInstruction(
            bytecode,
            scriptAccount.publicKey.toBase58(),
            connection,
            {
                fiveVMProgramId: config.fiveProgramId || 'HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg',
                vmStateAccount: config.vmStatePda || 'ErAR2V7HiASpZonjFpLK36dNt2Akk2zJoWveRxFbS3xX'
            }
        );

        // Fetch recent blockhash
        const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash('confirmed');

        // Send transaction
        const tx = new Transaction({
            recentBlockhash: blockhash,
            feePayer: payer.publicKey
        })
            .add(createIx);

        if (deployIx.instruction) {
            try {
                tx.add(deployIx.instruction);
            } catch (e) {
                console.error('Error adding deploy instruction:', e.message);
                throw e;
            }
        }

        console.log(`Sending transaction with ${tx.instructions.length} instructions...`);
        console.log(`Transaction size: ~${tx.serialize({ requireAllSignatures: false }).length} bytes`);

        const sig = await connection.sendTransaction(tx, [payer, scriptAccount], {
            skipPreflight: true
        });

        console.log(`✅ Transaction sent: ${sig}`);

        await connection.confirmTransaction(sig, 'confirmed');
        console.log(`✅ Counter deployed successfully!`);
        console.log(`Transaction signature: ${sig}`);

        // Update deployment config
        config.counterScriptAccount = scriptAccount.publicKey.toBase58();
        config.timestamp = new Date().toISOString();
        fs.writeFileSync(deploymentConfigPath, JSON.stringify(config, null, 2));
        console.log(`✅ Updated deployment-config.json`);

        process.exit(0);
    } catch (error) {
        console.error(`❌ Deployment failed:`, error.message);
        process.exit(1);
    }
}

deploy();
