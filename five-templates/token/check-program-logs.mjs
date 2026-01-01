import { Connection, PublicKey } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Load config
const config = JSON.parse(fs.readFileSync('deployment-config.json', 'utf-8'));

console.log('🔍 Checking for recent program logs\n');

// Get latest slot
const slot = await connection.getSlot('confirmed');
console.log(`Current slot: ${slot}`);

// Try to get logs for the FIVE program
const programId = new PublicKey(config.fiveProgramId);
const signatures = await connection.getSignaturesForAddress(programId, { limit: 5 });

console.log(`\nFound ${signatures.length} recent transactions\n`);

for (const sigInfo of signatures) {
  const tx = await connection.getTransaction(sigInfo.signature, {
    maxSupportedTransactionVersion: 0
  });

  if (tx && tx.meta) {
    const shortSig = sigInfo.signature.substring(0, 20);
    console.log(`\nTx: ${shortSig}...`);
    console.log(`Slot: ${tx.slot}`);
    console.log(`Status: ${tx.meta.err ? 'FAILED' : 'SUCCESS'}`);

    if (tx.meta.logMessages) {
      console.log('Logs:');
      for (const log of tx.meta.logMessages) {
        if (log.includes('ERROR') || log.includes('Execution') || log.includes('STORE') || log.includes('FIELD')) {
          console.log(`  ${log}`);
        }
      }
    }
  }
}
