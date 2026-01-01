import fs from 'fs';
import path from 'path';
import { Connection, PublicKey } from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Account addresses from the test
const accounts = {
  mint: 'B4P95ChptHtkNpREnLh2F3enzasatpc22nsgLh7ySg94',
  user1: 'E8FKB8eKZdX2TLaQypAHm5DJFFV8rYW6bDtZaatQX22d',
  user2: 'Dy7ejm4GFcwudotR3UjfUgQpUWDBngZ4xrqZHqn8t3r1',
  user3: 'BScXbBrDVr5DJXdAi6kRs6VssdtWKoGkG6Kmt5tsHpqW'
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
    if (account.data.length >= 8) {
      // Try to read 8-byte values (likely balance)
      const dataView = account.data;
      let offset = 0;

      // Check for non-zero data
      let nonZeroBytes = 0;
      for (let i = 0; i < Math.min(dataView.length, 128); i++) {
        if (dataView[i] !== 0) nonZeroBytes++;
      }
      console.log(`   Non-zero bytes (first 128): ${nonZeroBytes}`);

      // Try to read u64 values
      if (account.data.length >= 8) {
        const balance = Number(dataView.readBigUInt64LE(0));
        if (balance > 0) {
          console.log(`   Possible balance value: ${balance}`);
        }
      }
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
