#!/usr/bin/env node

import { Connection, PublicKey } from '@solana/web3.js';
import * as fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = 'http://127.0.0.1:8899';

async function debugOwnership() {
    const connection = new Connection(RPC_URL, 'confirmed');

    console.log('\n═══════════════════════════════════════════════════════════');
    console.log('Debug: Account Ownership Analysis');
    console.log('═══════════════════════════════════════════════════════════\n');

    // Load deployment config
    const configPath = path.join(__dirname, 'deployment-config.json');
    if (!fs.existsSync(configPath)) {
        console.error('❌ deployment-config.json not found');
        console.error('   Please run: npm run deploy');
        process.exit(1);
    }

    const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
    const PROGRAM_ID = new PublicKey(config.fiveProgramId);

    console.log('Checking account ownership...\n');

    // Check script account
    const scriptAccount = await connection.getAccountInfo(
        new PublicKey(config.tokenScriptAccount)
    );

    console.log(`Script Account: ${config.tokenScriptAccount}`);
    if (!scriptAccount) {
        console.log(`  ❌ NOT FOUND on-chain!`);
    } else {
        console.log(`  Owner: ${scriptAccount.owner.toBase58()}`);
        console.log(`  Expected: ${PROGRAM_ID.toBase58()}`);
        console.log(`  Match: ${scriptAccount.owner.equals(PROGRAM_ID) ? '✅' : '❌'}`);
    }
    console.log();

    // Check VM state PDA
    const vmState = await connection.getAccountInfo(
        new PublicKey(config.vmStatePda)
    );

    console.log(`VM State PDA: ${config.vmStatePda}`);
    if (!vmState) {
        console.log(`  ❌ NOT FOUND on-chain!`);
    } else {
        console.log(`  Owner: ${vmState.owner.toBase58()}`);
        console.log(`  Expected: ${PROGRAM_ID.toBase58()}`);
        console.log(`  Match: ${vmState.owner.equals(PROGRAM_ID) ? '✅' : '❌'}`);
    }
    console.log();

    // Identify the issue
    let hasIssues = false;

    if (!scriptAccount) {
        console.error('❌ ISSUE FOUND: Script account does not exist on-chain!');
        console.error('   This prevents any transactions from being executed.\n');
        hasIssues = true;
    } else if (!scriptAccount.owner.equals(PROGRAM_ID)) {
        console.error('❌ ISSUE FOUND: Script account is not owned by Five VM program!');
        console.error('   This causes "Provided owner is not allowed" / "IllegalOwner" error\n');
        console.error('   FIX: Redeploy script account with correct owner:\n');
        console.error('   npm run deploy\n');
        hasIssues = true;
    }

    if (!vmState) {
        console.error('❌ ISSUE FOUND: VM state PDA does not exist on-chain!');
        console.error('   This prevents VM initialization.\n');
        hasIssues = true;
    } else if (!vmState.owner.equals(PROGRAM_ID)) {
        console.error('❌ ISSUE FOUND: VM state PDA is not owned by Five VM program!');
        console.error('   This may cause execution failures\n');
        hasIssues = true;
    }

    if (!hasIssues) {
        console.log('✅ All account ownership checks passed\n');
    }

    process.exit(hasIssues ? 1 : 0);
}

debugOwnership().catch(error => {
    console.error('Debug failed:', error);
    process.exit(1);
});
