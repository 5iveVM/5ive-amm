import { Connection, PublicKey } from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const statePath = path.join(__dirname, 'test-state.json');
if (!fs.existsSync(statePath)) {
  console.log("Error: test-state.json not found");
  process.exit(1);
}
const state = JSON.parse(fs.readFileSync(statePath, 'utf8'));

// Accounts from the latest test run
const accounts = state.accounts;

async function dumpAccountState(pubkey, label) {
  try {
    const account = await connection.getAccountInfo(new PublicKey(pubkey));
    if (!account) {
      console.log(`\n❌ ${label}: NOT FOUND`);
      return;
    }

    console.log(`\n${'═'.repeat(80)}`);
    console.log(`${label}`);
    console.log(`${'═'.repeat(80)}`);
    console.log(`Address: ${pubkey}`);
    console.log(`Owner: ${account.owner.toBase58()}`);
    console.log(`Lamports: ${account.lamports}`);
    console.log(`Data Length: ${account.data.length} bytes`);
    console.log(`Executable: ${account.executable}`);

    // Show hex dump
    console.log(`\nHex Dump (full ${account.data.length} bytes):`);
    const data = account.data;
    for (let i = 0; i < data.length; i += 32) {
      let hex = '';
      let ascii = '';
      for (let j = 0; j < 32 && i + j < data.length; j++) {
        const byte = data[i + j];
        hex += byte.toString(16).padStart(2, '0') + ' ';
        ascii += (byte >= 32 && byte <= 126) ? String.fromCharCode(byte) : '.';
      }
      console.log(`${i.toString().padStart(4, '0')}: ${hex.padEnd(96)} ${ascii}`);
    }

    // Try to parse as u64 values
    console.log(`\nParsed Values (u64 integers, little-endian):`);
    for (let i = 0; i < Math.min(data.length, 256); i += 8) {
      if (i + 8 <= data.length) {
        const view = data.slice(i, i + 8);
        const value = view.readBigUInt64LE(0);
        console.log(`  Offset ${i.toString().padStart(3, '0')}: ${value.toString().padStart(20)} (0x${value.toString(16)})`);
      }
    }

  } catch (e) {
    console.log(`\n❌ ${label}: Error - ${e.message}`);
  }
}

async function main() {
  console.log('\n');
  console.log('█'.repeat(80));
  console.log('TOKEN ACCOUNT STATE DUMP');
  console.log('█'.repeat(80));
  console.log(`RPC: ${RPC_URL}`);
  console.log(`Timestamp: ${new Date().toISOString()}`);

  await dumpAccountState(accounts.mint, '📌 MINT ACCOUNT');
  await dumpAccountState(accounts.user1TokenAccount, '💰 USER1 TOKEN ACCOUNT');
  await dumpAccountState(accounts.user2TokenAccount, '💰 USER2 TOKEN ACCOUNT');
  await dumpAccountState(accounts.user3TokenAccount, '💰 USER3 TOKEN ACCOUNT');

  console.log('\n' + '█'.repeat(80));
  console.log('END OF STATE DUMP');
  console.log('█'.repeat(80) + '\n');
}

main().catch(console.error);
