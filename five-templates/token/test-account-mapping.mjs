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
console.log('ACCOUNT MAPPING TEST');
console.log('='.repeat(80));

// Create test keypairs
const mintAccount = Keypair.generate();
const authority = Keypair.generate();
const freezeAuthority = new PublicKey('11111111111111111111111111111111');
const scriptAccount = new PublicKey('7uU9TNrgwojKiM2NBttzQt7zijS1D9EwDJfpNfKTVuv4');

// Create accounts array as it would be passed to generateExecuteInstruction
const accountsArray = [
  mintAccount.publicKey.toBase58(),
  authority.publicKey.toBase58()
];

console.log('\n📋 Test Setup:');
console.log(`  Script Account: ${scriptAccount.toBase58()}`);
console.log(`  Accounts Array:`);
accountsArray.forEach((a, i) => console.log(`    [${i}] ${a.slice(0, 8)}...`));

console.log('\n📝 Parameters (for init_mint):');
const parameters = [
  mintAccount.publicKey.toBase58(),
  authority.publicKey.toBase58(),
  freezeAuthority.toBase58(),
  6,
  "TestToken",
  "TEST",
  "https://example.com/token"
];

parameters.forEach((p, i) => {
  if (typeof p === 'string' && p.length === 44) {
    console.log(`  [${i}] PublicKey(${p.slice(0, 8)}...)`);
  } else {
    console.log(`  [${i}] ${p}`);
  }
});

// Get function definition
const initMintFunc = tokenFiveFile.abi.functions.find(f => f.name === 'init_mint');
console.log(`\n🔍 Function Definition (init_mint):`);
console.log(`  Index: ${initMintFunc.index}`);
console.log(`  Parameters:`);
initMintFunc.parameters.forEach((p, i) => {
  const isAccount = p.is_account || p.isAccount;
  const type = p.param_type || p.type;
  console.log(`    [${i}] ${p.name}: ${type}${isAccount ? ' (ACCOUNT)' : ''}`);
});

// Test SDK's account mapping with debug=true
console.log('\n\n📊 Testing SDK Account Mapping (with debug=true):');
console.log('='.repeat(80));

try {
  const result = await FiveSDK.generateExecuteInstruction(
    scriptAccount.toBase58(),
    'init_mint',
    parameters,
    accountsArray,
    undefined,
    {
      debug: true,
      abi: tokenFiveFile.abi
    }
  );

  console.log('\n✅ SUCCESS: Instruction generation succeeded!');
  console.log(`\nGenerated Instruction Data (first 50 bytes):`);
  const data = Buffer.from(result.instruction.data, 'base64');
  console.log(`  Length: ${data.length} bytes`);
  console.log(`  Hex: ${data.slice(0, 50).toString('hex')}`);

} catch (error) {
  console.log('\n❌ ERROR: Account mapping failed');
  console.log(`\nError Message:\n${error.message}`);
  console.log(`\nFull Error:\n${error.stack}`);
}

console.log('\n' + '='.repeat(80));
console.log('TEST COMPLETE');
console.log('='.repeat(80));
