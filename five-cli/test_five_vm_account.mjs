import { Connection, Keypair, SystemProgram, Transaction, PublicKey } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Load keypair
const keypairPath = process.env.HOME + '/.config/solana/id.json';
const keypairBuffer = fs.readFileSync(keypairPath);
const keypairArray = JSON.parse(keypairBuffer.toString());
const payer = Keypair.fromSecretKey(new Uint8Array(keypairArray));

// The Five VM program deployed earlier
const FIVE_VM_PROGRAM_ID = 'AjEcViqBu32FiBV25pgoPeTC4DQtNwD6tkPfwCWa6NfN';

console.log('Testing account creation owned by Five VM...');
console.log('Payer:', payer.publicKey.toBase58());
console.log('Five VM Program:', FIVE_VM_PROGRAM_ID);

const newAccount = Keypair.generate();
const space = 65536; // 64KB like a script account
const lamports = await connection.getMinimumBalanceForRentExemption(space);

console.log('Creating account:', newAccount.publicKey.toBase58());
console.log('Account size:', space, 'bytes');
console.log('Rent required:', lamports / 1e9, 'SOL');

const tx = new Transaction();
tx.add(
  SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: newAccount.publicKey,
    lamports: lamports,
    space: space,
    programId: new PublicKey(FIVE_VM_PROGRAM_ID),
  })
);

const { blockhash } = await connection.getLatestBlockhash('confirmed');
tx.recentBlockhash = blockhash;
tx.feePayer = payer.publicKey;
tx.sign(payer, newAccount);

console.log('\nTransaction details:');
console.log('  Signatures:', tx.signatures.length);
console.log('  Instructions:', tx.instructions.length);
console.log('  Size:', tx.serialize().length, 'bytes');

const signature = await connection.sendRawTransaction(tx.serialize(), {
  skipPreflight: false,
  maxRetries: 3,
});

console.log('\n✅ Transaction sent:', signature);

// Try to get signature status
const startTime = Date.now();
let status = null;
let found = false;
while (Date.now() - startTime < 10000) {
  status = await connection.getSignatureStatus(signature);
  const elapsed = Date.now() - startTime;
  console.log(`[${elapsed}ms] Signature status:`, status?.value ? 'CONFIRMED' : 'PENDING');
  if (status?.value) {
    console.log('✅ Account creation transaction confirmed!');
    
    // Verify account was created
    const accountInfo = await connection.getAccountInfo(newAccount.publicKey);
    console.log('\nAccount verification:');
    console.log('  Owner:', accountInfo?.owner.toBase58());
    console.log('  Lamports:', accountInfo?.lamports);
    console.log('  Executable:', accountInfo?.executable);
    
    if (accountInfo?.owner.toBase58() === FIVE_VM_PROGRAM_ID) {
      console.log('✅ Account is correctly owned by Five VM program!');
    } else {
      console.log('❌ Account owner mismatch!');
    }
    found = true;
    break;
  }
  await new Promise(resolve => setTimeout(resolve, 500));
}

if (!found) {
  console.log('❌ Transaction not confirmed after 10 seconds');
  process.exit(1);
}
