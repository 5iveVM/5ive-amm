import fs from 'fs';
import path from 'path';
import { Connection, PublicKey } from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Account addresses from the test
const accounts = {
  mint: 'uUJNxPr9WgiejEyRuQZ5yZKb2NxMun2hPv7yDo13PfC',
  user1: '4zBmTtu2y51f2iKQEJvcvMN3vpER87Jm6nwtZd7D7tFL',
  user2: '5dWA5xnuaZaSN6mQtmBikQWFJaWAULeajKbc1LfGAzv2',
  user3: 'GJvykjgq6jkH8xEGXyM4W6LFS2XK6CjY1ntByWVQ7SX9'
};

async function getAccountState(pubkey, label) {
  try {
    const account = await connection.getAccountInfo(new PublicKey(pubkey));
    if (!account) {
      console.log(`\n❌ ${label}: NOT FOUND`);
      return null;
    }

    console.log(`\n✅ ${label}`);
    console.log(`   Address: ${pubkey}`);
    console.log(`   Owner: ${account.owner.toBase58()}`);
    console.log(`   Lamports: ${account.lamports}`);
    console.log(`   Data length: ${account.data.length} bytes`);
    console.log(`   Executable: ${account.executable}`);

    // Try to parse as a simple token account structure
    if (account.data.length >= 72) {
      const dataView = account.data;
      
      const owner = new PublicKey(dataView.subarray(0, 32)).toBase58();
      const mint = new PublicKey(dataView.subarray(32, 64)).toBase58();
      const balance = Number(dataView.readBigUInt64LE(64));
      
      console.log(`   [Field 0] Owner:   ${owner}`);
      console.log(`   [Field 1] Mint:    ${mint}`);
      console.log(`   [Field 2] Balance: ${balance}`);
    }

    return account;
  } catch (e) {
    console.log(`\n❌ ${label}: Error - ${e.message}`);
    return null;
  }
}

async function main() {
  console.log('═══════════════════════════════════════════════════════════');
  console.log('Token Account State Verification');
  console.log('═══════════════════════════════════════════════════════════');
  console.log(`RPC: ${RPC_URL}`);

  const mintAccount = await getAccountState(accounts.mint, 'MINT Account');
  const user1Account = await getAccountState(accounts.user1, 'USER1 Token Account');
  const user2Account = await getAccountState(accounts.user2, 'USER2 Token Account');
  const user3Account = await getAccountState(accounts.user3, 'USER3 Token Account');

  console.log('\n═══════════════════════════════════════════════════════════');
  console.log('State Summary');
  console.log('═══════════════════════════════════════════════════════════');

  const allAccountsExist = [mintAccount, user1Account, user2Account, user3Account].every(a => a !== null);

  if (allAccountsExist) {
    console.log('✅ All token accounts exist on-chain');
    console.log('✅ State has been persisted to accounts');
  } else {
    console.log('⚠️  Some accounts are missing');
  }

  // Try to detect token supply
  if (mintAccount && mintAccount.data.length > 0) {
    // Look for non-zero values that could indicate supply
    let hasData = false;
    for (let i = 0; i < Math.min(mintAccount.data.length, 256); i++) {
      if (mintAccount.data[i] !== 0) {
        hasData = true;
        break;
      }
    }
    if (hasData) {
      console.log('✅ Mint account contains state data');
    }
  }
}

main().catch(console.error);
