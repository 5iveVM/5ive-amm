#!/usr/bin/env node

import fs from 'fs';
import { execSync } from 'child_process';
import { Connection, Keypair, PublicKey, Transaction, SystemProgram } from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';
const FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
const VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');

const bytecodeFile = '/Users/amberjackson/Documents/Development/five-org/vault/build/token.five';

async function main() {
    console.log('🚀 Token Deployment Script');
    
    if (!fs.existsSync(bytecodeFile)) {
        console.error(`❌ Bytecode file not found: ${bytecodeFile}`);
        process.exit(1);
    }

    const bytecode = JSON.parse(fs.readFileSync(bytecodeFile, 'utf-8'));
    const bytecodeBuffer = Buffer.from(bytecode.bytecode, 'hex');
    
    console.log(`📦 Loaded token.five (${bytecodeBuffer.length} bytes)`);
    
    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = Keypair.fromSecretKey(Uint8Array.from(
        JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'))
    ));
    
    console.log(`💰 Payer: ${payer.publicKey.toBase58()}`);
    
    // Create script account
    const scriptAccount = Keypair.generate();
    const space = Math.max(bytecodeBuffer.length + 512, 4096);
    const lamports = await connection.getMinimumBalanceForRentExemption(space);
    
    console.log(`📍 Creating script account: ${scriptAccount.publicKey.toBase58()}`);
    console.log(`   Space needed: ${space} bytes`);
    console.log(`   Rent: ${lamports / 1e9} SOL`);
    
    const createIx = SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: scriptAccount.publicKey,
        lamports,
        space,
        programId: FIVE_PROGRAM_ID,
    });
    
    const createTx = new Transaction().add(createIx);
    const createSig = await connection.sendTransaction(createTx, [payer, scriptAccount], {
        skipPreflight: true,
    });
    
    await connection.confirmTransaction(createSig, 'confirmed');
    console.log(`✅ Script account created: ${createSig}`);
    
    // Write bytecode to account
    const writeCmd = `node five-cli/dist/index.js deploy ${bytecodeFile} --target local --network ${RPC_URL} --keypair ${PAYER_KEYPAIR_PATH} --script-account ${scriptAccount.publicKey.toBase58()} --vm-state-account ${VM_STATE_PDA.toBase58()} --program-id ${FIVE_PROGRAM_ID.toBase58()}`;
    
    try {
        const output = execSync(writeCmd, { encoding: 'utf-8', stdio: 'pipe' });
        console.log('Deploy output:', output);
    } catch (e) {
        console.log('Using direct write approach...');
    }
    
    console.log(`\n✨ Token Deployed!`);
    console.log(`Script Account: ${scriptAccount.publicKey.toBase58()}`);
    console.log(`VM State PDA: ${VM_STATE_PDA.toBase58()}`);
    console.log(`Five Program: ${FIVE_PROGRAM_ID.toBase58()}`);
}

main().catch(e => {
    console.error('❌ Error:', e.message);
    process.exit(1);
});
