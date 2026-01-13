import { readFileSync } from 'fs';

// Check counter bytecode size
try {
  const counterBytecode = readFileSync('/Users/amberjackson/Documents/Development/five-org/five-mono/five-templates/counter/.five/artifacts/counter.five', 'utf-8');
  const parsed = JSON.parse(counterBytecode);
  if (parsed.bytecode) {
    const bytecodeBuffer = Buffer.from(parsed.bytecode, 'hex');
    console.log('Counter bytecode size:', bytecodeBuffer.length, 'bytes');
    console.log('Hex string size:', parsed.bytecode.length / 2, 'bytes');
  }
} catch (e) {
  console.log('Counter bytecode not found');
}

// Calculate max bytecode size for a transaction
const MAX_TX_SIZE = 1232;
const ACCOUNT_CREATION_SIZE = 50; // Estimated size for each SystemProgram.createAccount instruction
const INIT_INSTRUCTION_SIZE = 100; // Estimated size for init instruction
const DEPLOY_INSTRUCTION_OVERHEAD = 100; // Instruction header + account metadata
const SIGNATURE_SIZE = 64;
const PUBKEY_SIZE = 32;

// Base transaction structure
let usedSize = 100; // Transaction header and metadata
usedSize += ACCOUNT_CREATION_SIZE; // VM state account creation
usedSize += INIT_INSTRUCTION_SIZE; // VM state init
usedSize += ACCOUNT_CREATION_SIZE; // Script account creation
usedSize += DEPLOY_INSTRUCTION_OVERHEAD;
usedSize += SIGNATURE_SIZE * 3; // 3 signatures (payer, vmstate, script)

const maxBytecodeSize = MAX_TX_SIZE - usedSize;

console.log('\nTransaction size breakdown:');
console.log('  Base overhead:', 100);
console.log('  VM state createAccount:', ACCOUNT_CREATION_SIZE);
console.log('  VM state init:', INIT_INSTRUCTION_SIZE);
console.log('  Script createAccount:', ACCOUNT_CREATION_SIZE);
console.log('  Deploy instruction overhead:', DEPLOY_INSTRUCTION_OVERHEAD);
console.log('  Signatures (3x):', SIGNATURE_SIZE * 3);
console.log('  Total overhead:', usedSize);
console.log('  Max bytecode size:', maxBytecodeSize, 'bytes');
