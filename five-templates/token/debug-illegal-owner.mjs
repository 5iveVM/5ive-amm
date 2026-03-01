#!/usr/bin/env node

import { Connection, PublicKey } from '@solana/web3.js';

const RPC_URL = process.env.FIVE_RPC_URL || '';
const PROGRAM_ID_RAW = process.env.FIVE_PROGRAM_ID || '';
const VM_STATE_PDA_RAW = process.env.VM_STATE_PDA || '';
const TOKEN_SCRIPT_ACCOUNT_RAW = process.env.FIVE_TOKEN_SCRIPT_ACCOUNT || process.env.TOKEN_SCRIPT_ACCOUNT || '';

if (!RPC_URL || !PROGRAM_ID_RAW || !VM_STATE_PDA_RAW || !TOKEN_SCRIPT_ACCOUNT_RAW) {
    console.error('❌ Missing explicit configuration.');
    console.error('   Required env vars: FIVE_RPC_URL, FIVE_PROGRAM_ID, VM_STATE_PDA, FIVE_TOKEN_SCRIPT_ACCOUNT');
    process.exit(1);
}

async function debugOwnership() {
    const connection = new Connection(RPC_URL, 'confirmed');
    const PROGRAM_ID = new PublicKey(PROGRAM_ID_RAW);
    const VM_STATE_PDA = new PublicKey(VM_STATE_PDA_RAW);
    const TOKEN_SCRIPT_ACCOUNT = new PublicKey(TOKEN_SCRIPT_ACCOUNT_RAW);

    console.log('\n═══════════════════════════════════════════════════════════');
    console.log('Debug: Account Ownership Analysis');
    console.log('═══════════════════════════════════════════════════════════\n');

    console.log('Checking account ownership...\n');

    // Check script account
    const scriptAccount = await connection.getAccountInfo(TOKEN_SCRIPT_ACCOUNT);

    console.log(`RPC URL: ${RPC_URL}`);
    console.log(`Program ID: ${PROGRAM_ID.toBase58()}`);
    console.log(`Script Account: ${TOKEN_SCRIPT_ACCOUNT.toBase58()}`);
    if (!scriptAccount) {
        console.log(`  ❌ NOT FOUND on-chain!`);
    } else {
        console.log(`  Owner: ${scriptAccount.owner.toBase58()}`);
        console.log(`  Expected: ${PROGRAM_ID.toBase58()}`);
        console.log(`  Match: ${scriptAccount.owner.equals(PROGRAM_ID) ? '✅' : '❌'}`);
    }
    console.log();

    // Check VM state PDA
    const vmState = await connection.getAccountInfo(VM_STATE_PDA);

    console.log(`VM State PDA: ${VM_STATE_PDA.toBase58()}`);
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
        console.error('   FIX: Redeploy the script against this explicit cluster/program, then rerun with the new script account.\n');
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
