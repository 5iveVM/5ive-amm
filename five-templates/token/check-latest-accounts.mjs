import { Connection, PublicKey } from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Latest accounts from the test run
const accounts = {
  'Mint': 'HRNfnQY94FeMfNZ3h8YxwPjVLJ1GSySqZXG5kWKqdupz',
  'User1': 'G2otALGkyoRXWhnxx5vjHGWvxj3T4fzh21FdDQP9phBj',
  'User2': 'A5LeamrwZvqikCyakpkhWNSZrH2kytwk8zuaYjSAn4HP',
  'User3': '4hhkD9xdqPo52sqxN9qUwTN6T78gupA27tw9TfomtGWe',
};

console.log('🔍 Latest Token Account State\n');
console.log('═══════════════════════════════════════════════════════════\n');

for (const [name, address] of Object.entries(accounts)) {
  const pubkey = new PublicKey(address);
  const accountInfo = await connection.getAccountInfo(pubkey);

  if (!accountInfo) {
    console.log(`❌ ${name}: NOT FOUND\n`);
    continue;
  }

  let nonZero = 0;
  for (let byte of accountInfo.data) {
    if (byte !== 0) nonZero++;
  }

  console.log(`${name} (${address})`);
  console.log(`  Data: ${nonZero}/${accountInfo.data.length} non-zero bytes`);

  if (nonZero > 0) {
    console.log(`  ✅ STATE WRITTEN`);
    const dataHex = accountInfo.data.toString('hex');
    console.log(`  Hex (first 64 chars): ${dataHex.substring(0, 64)}`);
  } else {
    console.log(`  ❌ ALL ZEROS`);
  }
  console.log();
}
