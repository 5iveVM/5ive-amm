import { Connection, PublicKey } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Load config
const config = JSON.parse(fs.readFileSync('/Users/amberjackson/Documents/Development/five-org/five-templates/token/deployment-config.json', 'utf-8'));

console.log('🔎 Searching for All Program-Owned Accounts\n');

// Get all accounts owned by the FIVE program
const programId = new PublicKey(config.fiveProgramId);
const accounts = await connection.getProgramAccounts(programId);

console.log(`Found ${accounts.length} accounts owned by FIVE Program\n`);
console.log('═══════════════════════════════════════════════════════════\n');

for (const { pubkey, account } of accounts) {
  const addr = pubkey.toBase58();
  console.log(`📍 ${addr}`);
  console.log(`   Lamports: ${account.lamports}`);
  console.log(`   Data Size: ${account.data.length} bytes`);

  // Check if it's our known accounts
  const isVMState = addr === config.vmStatePda;
  const isScript = addr === config.tokenScriptAccount;
  const isMint = addr === 'CWb6RUW6Qmh2xneByzVs7KDGCUUYHRaytf6euJTkYDQa';
  const isUser = ['DSvpC2B58iHf6iuEbYNF5zobN44xXsVtWWVEH7XXX8S2', '5pay7FhSsW6CxaV99MVahRS4RXZCMGB6xsRBdEkME5iK', 'H7Ud9gYKCgAe8PLEuqBvtQirPrz4S4oe5R924VF9Zpd2'].includes(addr);

  if (isVMState) console.log(`   ★ VM STATE ACCOUNT`);
  if (isScript) console.log(`   ★ TOKEN SCRIPT ACCOUNT`);
  if (isMint) console.log(`   ★ MINT ACCOUNT (from E2E test)`);
  if (isUser) console.log(`   ★ USER ACCOUNT (from E2E test)`);

  // Count non-zero
  let nonZero = 0;
  for (let byte of account.data) {
    if (byte !== 0) nonZero++;
  }
  console.log(`   Non-zero: ${nonZero}/${account.data.length} bytes`);
  console.log();
}
