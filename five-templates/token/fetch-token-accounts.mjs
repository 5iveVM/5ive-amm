import { Connection, PublicKey } from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';

// From the latest E2E test run
const accounts = {
  mintAccount: 'CWb6RUW6Qmh2xneByzVs7KDGCUUYHRaytf6euJTkYDQa',
  user1Account: 'DSvpC2B58iHf6iuEbYNF5zobN44xXsVtWWVEH7XXX8S2',
  user2Account: '5pay7FhSsW6CxaV99MVahRS4RXZCMGB6xsRBdEkME5iK',
  user3Account: 'H7Ud9gYKCgAe8PLEuqBvtQirPrz4S4oe5R924VF9Zpd2',
};

async function fetchAndDisplayAccounts() {
  const connection = new Connection(RPC_URL, 'confirmed');

  console.log('📦 Token Account Data Fetch\n');
  console.log('═══════════════════════════════════════════════════════════\n');

  for (const [name, address] of Object.entries(accounts)) {
    try {
      const pubkey = new PublicKey(address);
      const accountInfo = await connection.getAccountInfo(pubkey);

      if (!accountInfo) {
        console.log(`❌ ${name} (${address}): NOT FOUND\n`);
        continue;
      }

      console.log(`✅ ${name}`);
      console.log(`   Address: ${address}`);
      console.log(`   Owner: ${accountInfo.owner.toBase58()}`);
      console.log(`   Lamports: ${accountInfo.lamports}`);
      console.log(`   Data Size: ${accountInfo.data.length} bytes`);
      console.log(`   Executable: ${accountInfo.executable}`);

      // Display hex dump
      const dataHex = accountInfo.data.toString('hex');
      console.log(`\n   Data (hex):`);

      // Print in 32-byte chunks
      for (let i = 0; i < Math.min(dataHex.length, 512); i += 64) {
        const chunk = dataHex.substring(i, i + 64);
        console.log(`   ${chunk}`);
      }

      if (dataHex.length > 512) {
        console.log(`   ... (${Math.ceil(dataHex.length / 2) - 256} more bytes)`);
      }

      // Try to parse some fields
      console.log(`\n   Data Analysis:`);

      // Check if there's meaningful data
      let nonZeroCount = 0;
      for (let byte of accountInfo.data) {
        if (byte !== 0) nonZeroCount++;
      }

      console.log(`   - Non-zero bytes: ${nonZeroCount}/${accountInfo.data.length}`);

      // Try to interpret first few bytes as u64 (balance?)
      if (accountInfo.data.length >= 8) {
        const view = new DataView(accountInfo.data.buffer, accountInfo.data.byteOffset, 8);
        const balance = view.getBigUint64(0, true); // little endian
        console.log(`   - First 8 bytes as u64: ${balance}`);
      }

      // Look for common patterns
      if (nonZeroCount > 0) {
        console.log(`   ✓ Account has been initialized with data`);
      } else {
        console.log(`   ⚠️ Account appears to be all zeros`);
      }

      console.log();
    } catch (error) {
      console.log(`❌ ${name}: Error - ${error.message}\n`);
    }
  }

  console.log('═══════════════════════════════════════════════════════════');
  console.log('\n📊 Summary\n');
  console.log('Account Data Status:');
  console.log('  - Mint Account: Stores token metadata (authority, supply, decimals)');
  console.log('  - User Accounts: Store balances, delegation, frozen status');
  console.log('  - All accounts owned by FIVE Program for state management');
}

fetchAndDisplayAccounts().catch(e => console.error('Error:', e.message));
