#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';
import {
    Connection, Keypair, PublicKey, Transaction,
    SystemProgram, TransactionInstruction
} from '@solana/web3.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

async function deploy() {
    console.log('Deploying updated counter bytecode...');

    const deploymentConfigPath = path.join(__dirname, 'deployment-config.json');
    const config = JSON.parse(fs.readFileSync(deploymentConfigPath, 'utf-8'));

    // Read compiled bytecode
    const bytecodeFile = path.join(__dirname, 'build', 'five-counter-template.five');
    const fiveFile = JSON.parse(fs.readFileSync(bytecodeFile, 'utf-8'));
    const bytecodeBase64 = fiveFile.bytecode;
    const bytecode = Buffer.from(bytecodeBase64, 'base64');

    console.log(`Bytecode size: ${bytecode.length} bytes`);

    // Create a new script account
    const newScriptAccount = Keypair.generate();
    console.log(`New script account: ${newScriptAccount.publicKey.toBase58()}`);

    // Load payer
    const payer = Keypair.fromSecretKey(
        Uint8Array.from(JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8')))
    );

    const connection = new Connection(RPC_URL, 'confirmed');

    // Use solana CLI to deploy the bytecode
    console.log(`Using Solana CLI to deploy...`);

    try {
        // Create the script account
        const createCmd = `solana create-account ${newScriptAccount.publicKey.toBase58()} ${bytecode.length + 512} ${config.fiveProgramId} --url ${RPC_URL}`;
        console.log(`Creating account...`);
        execSync(createCmd, { stdio: 'pipe' });
        console.log(`✅ Account created`);

        // Write bytecode to the account (this would require a special write command)
        // For now, let's try using the five CLI deploy command if available

        console.log(`\nNote: Full deployment requires the five CLI or custom RPC calls`);
        console.log(`Manual next step: Deploy using five-surfpool or custom script`);

    } catch (error) {
        console.error(`Deployment error: ${error.message}`);
        process.exit(1);
    }
}

deploy();
