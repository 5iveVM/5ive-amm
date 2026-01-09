#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { FiveSDK } from '../../five-sdk/dist/index.js';
import { Keypair, PublicKey } from '@solana/web3.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load token ABI
const tokenFiveFile = JSON.parse(fs.readFileSync(path.join(__dirname, 'build', 'five-token-template.five'), 'utf-8'));

console.log('='.repeat(80));
console.log('INSTRUCTION DATA GENERATION DEBUG');
console.log('='.repeat(80));

// Simulate instruction generation
const scriptAccount = new PublicKey('7uU9TNrgwojKiM2NBttzQt7zijS1D9EwDJfpNfKTVuv4');
const mintAccount = Keypair.generate();
const authority = Keypair.generate();
const freezeAuthority = new PublicKey('11111111111111111111111111111111');

const parameters = [
    mintAccount.publicKey,
    authority.publicKey,
    freezeAuthority,
    6,
    "TestToken",
    "TEST",
    "https://example.com/token"
];

console.log('\n📝 Parameters:');
parameters.forEach((p, i) => {
    console.log(`  [${i}] ${typeof p === 'object' ? p.toString() : p}`);
});

console.log('\n📊 Building execution instruction...');

try {
    // This should call the SDK's instruction generation
    const executeInstr = FiveSDK.buildExecuteInstruction(
        0,  // function index
        parameters,
        tokenFiveFile.abi,
        7   // param count
    );

    console.log(`\n✅ Instruction built successfully`);
    console.log(`   Total bytes: ${executeInstr.length}`);
    console.log(`   Hex: ${Buffer.from(executeInstr).toString('hex')}`);

    // Analyze the bytes
    console.log('\n🔍 Byte Analysis:');
    const bytes = Buffer.from(executeInstr);

    // First byte should be discriminator 9
    console.log(`   [0] Discriminator: 0x${bytes[0].toString(16)} (should be 0x9)`);

    // Next should be function index (VLE encoded)
    console.log(`   [1] Function Index byte: 0x${bytes[1].toString(16)}`);

    // Check for sentinel 128
    console.log(`   [2-3] Next bytes: 0x${bytes[2].toString(16)} 0x${bytes[3].toString(16)}`);
    console.log(`   Checking if these form VLE(128): ${bytes[2].toString(16)} ${bytes[3].toString(16)}`);

    // Print first 20 bytes
    console.log(`\n   First 20 bytes:`);
    console.log(`   ${Array.from(bytes.slice(0, 20)).map(b => '0x' + b.toString(16).padStart(2, '0')).join(' ')}`);

} catch (error) {
    console.error('\n❌ Error building instruction:');
    console.error(error.message);
    if (error.stack) console.error(error.stack);
}

// Also test counter for comparison
console.log('\n' + '='.repeat(80));
console.log('COUNTER COMPARISON');
console.log('='.repeat(80));

const counterFiveFile = JSON.parse(fs.readFileSync(path.join(__dirname, '../counter/build/five-counter-template.five'), 'utf-8'));

const counterAccount = Keypair.generate();
const counterOwner = Keypair.generate();

const counterParams = [counterAccount.publicKey, counterOwner.publicKey];

console.log('\n📝 Counter Parameters:');
counterParams.forEach((p, i) => {
    console.log(`  [${i}] ${p.toString()}`);
});

try {
    const counterInstr = FiveSDK.buildExecuteInstruction(
        0,  // function index
        counterParams,
        counterFiveFile.abi,
        2   // param count
    );

    console.log(`\n✅ Counter instruction built`);
    console.log(`   Total bytes: ${counterInstr.length}`);
    console.log(`   Hex: ${Buffer.from(counterInstr).toString('hex')}`);

    const counterBytes = Buffer.from(counterInstr);
    console.log(`\n   [0] Discriminator: 0x${counterBytes[0].toString(16)}`);
    console.log(`   [1] Function Index: 0x${counterBytes[1].toString(16)}`);
    console.log(`   [2-3] Sentinel/Count: 0x${counterBytes[2].toString(16)} 0x${counterBytes[3].toString(16)}`);
    console.log(`\n   First 20 bytes:`);
    console.log(`   ${Array.from(counterBytes.slice(0, 20)).map(b => '0x' + b.toString(16).padStart(2, '0')).join(' ')}`);

} catch (error) {
    console.error('\n❌ Error with counter:');
    console.error(error.message);
}
