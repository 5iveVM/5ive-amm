import { Connection, PublicKey } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = 'http://127.0.0.1:8899';

async function fetchVMState() {
  const connection = new Connection(RPC_URL, 'confirmed');

  // Load config
  const config = JSON.parse(fs.readFileSync('deployment-config.json', 'utf-8'));

  console.log('🔍 VM State & Script Account Deep Dive\n');
  console.log('═══════════════════════════════════════════════════════════\n');

  // 1. Check VM State Account
  console.log('1️⃣ VM STATE ACCOUNT\n');
  const vmStateKey = new PublicKey(config.vmStatePda);
  const vmStateInfo = await connection.getAccountInfo(vmStateKey);

  if (vmStateInfo) {
    console.log(`Address: ${config.vmStatePda}`);
    console.log(`Owner: ${vmStateInfo.owner.toBase58()}`);
    console.log(`Data Size: ${vmStateInfo.data.length} bytes`);
    console.log(`Lamports: ${vmStateInfo.lamports}`);
    console.log(`\nData (hex):`);
    console.log(vmStateInfo.data.toString('hex'));
    console.log(`\nData (ASCII attempt):`);
    try {
      console.log(vmStateInfo.data.toString('utf8').replace(/[^\x20-\x7e]/g, '.'));
    } catch (e) {
      console.log('(binary data)');
    }
  }

  // 2. Check Script Account
  console.log('\n\n2️⃣ SCRIPT ACCOUNT (Token Bytecode)\n');
  const scriptKey = new PublicKey(config.tokenScriptAccount);
  const scriptInfo = await connection.getAccountInfo(scriptKey);

  if (scriptInfo) {
    console.log(`Address: ${config.tokenScriptAccount}`);
    console.log(`Owner: ${scriptInfo.owner.toBase58()}`);
    console.log(`Data Size: ${scriptInfo.data.length} bytes`);
    console.log(`Lamports: ${scriptInfo.lamports}`);

    // Analyze structure
    const dataHex = scriptInfo.data.toString('hex');

    console.log(`\nFirst 256 bytes (hex):`);
    console.log(dataHex.substring(0, 512));

    // Try to parse header
    const view = new DataView(scriptInfo.data.buffer, scriptInfo.data.byteOffset);

    console.log(`\nPotential Header Fields:`);
    console.log(`  Bytes 0-3 (magic?): 0x${dataHex.substring(0, 8)}`);
    console.log(`  Bytes 4-7 (size?): 0x${dataHex.substring(8, 16)}`);

    // Count non-zero
    let nonZero = 0;
    for (let byte of scriptInfo.data) {
      if (byte !== 0) nonZero++;
    }
    console.log(`\nData Statistics:`);
    console.log(`  Total bytes: ${scriptInfo.data.length}`);
    console.log(`  Non-zero bytes: ${nonZero}`);
    console.log(`  ${nonZero > 0 ? '✓ Bytecode appended' : '✗ No bytecode data'}`);
  }

  // 3. Check if state might be in a PDA
  console.log('\n\n3️⃣ CHECKING FOR STATE PDAs\n');

  // Try to find any accounts related to the mint
  const mintKey = new PublicKey('CWb6RUW6Qmh2xneByzVs7KDGCUUYHRaytf6euJTkYDQa');
  const mintInfo = await connection.getAccountInfo(mintKey);

  console.log(`Mint Account: ${mintKey.toBase58()}`);
  console.log(`  Owner: ${mintInfo?.owner.toBase58()}`);
  console.log(`  Data size: ${mintInfo?.data.length}`);
  console.log(`  All zeros: ${mintInfo?.data.every(b => b === 0)}`);

  // Summary
  console.log('\n\n═══════════════════════════════════════════════════════════');
  console.log('\n📊 State Storage Analysis\n');
  console.log('Observations:');
  console.log('  1. Token accounts created but contain all zeros');
  console.log('  2. Script account contains FIVE bytecode');
  console.log('  3. VM state account exists and is initialized');
  console.log('  4. Mint operations report success but no account state written');
  console.log('\nPossibilities:');
  console.log('  A) State is stored in VM state account (transient)');
  console.log('  B) State is stored in heap/memory during execution only');
  console.log('  C) Token accounts need different account type/size');
  console.log('  D) State persistence to accounts not fully implemented');
}

fetchVMState().catch(e => console.error('Error:', e.message));
