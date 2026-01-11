import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { FiveProgram } from '../../five-sdk/dist/index.js';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// Load ABI
const abiPath = path.join(__dirname, 'src', 'counter.abi.json');
const abi = JSON.parse(fs.readFileSync(abiPath, 'utf-8'));

console.log('ABI structure:');
console.log('- name:', abi.name);
console.log('- functions type:', typeof abi.functions);
console.log('- functions is array:', Array.isArray(abi.functions));
console.log('- function count:', abi.functions?.length);
console.log('- first function:', JSON.stringify(abi.functions?.[0], null, 2));

// Test FiveProgram
const COUNTER_SCRIPT = 'GozdrELSNrs2emihAKxVQtcHzvAjz6CZNeDF4vTxfWFm';
const program = FiveProgram.fromABI(COUNTER_SCRIPT, abi);
console.log('\nFiveProgram initialized');
console.log('Functions:', program.getFunctions());

// Try building an instruction
console.log('\nTesting instruction building...');
try {
  const builder = program
    .function('increment')
    .accounts({
      counter: 'HqtHLCBCGNZpFanWXDqkoaGgLyKXi6Ed5a4SmVZwJpFC',
      owner: '3RvpMFnthpu5aVw7qG1RUCWjm8UaKVmqM1GP1ao4jWaV'
    });
  
  console.log('Builder created successfully');
  console.log('Attempting to generate instruction...');
  const ix = await builder.instruction();
  console.log('Instruction generated!');
  console.log('programId:', ix.programId);
  console.log('keys:', ix.keys);
} catch (err) {
  console.error('Error:', err.message);
  console.error('Stack:', err.stack);
}
