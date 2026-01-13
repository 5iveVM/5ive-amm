import { Connection, Keypair, SystemProgram, Transaction, PublicKey, TransactionInstruction } from '@solana/web3.js';
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

console.log('Testing multi-instruction transaction (VM state + Script account)...');
console.log('Payer:', payer.publicKey.toBase58());

// Create VM state keypair
const vmStateKeypair = Keypair.generate();
const vmStateSpace = 1024;
const vmStateRent = await connection.getMinimumBalanceForRentExemption(vmStateSpace);

// Create script keypair
const scriptKeypair = Keypair.generate();
const scriptSpace = 65536;
const scriptRent = await connection.getMinimumBalanceForRentExemption(scriptSpace);

console.log('\nAccounts to create:');
console.log('  VM State:', vmStateKeypair.publicKey.toBase58());
console.log('  Script:', scriptKeypair.publicKey.toBase58());

const tx = new Transaction();

// 1) Create VM state account
tx.add(
  SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: vmStateKeypair.publicKey,
    lamports: vmStateRent,
    space: vmStateSpace,
    programId: new PublicKey(FIVE_VM_PROGRAM_ID),
  })
);
console.log('✓ Added createAccount instruction for VM state');

// 2) Initialize VM state
tx.add(
  new TransactionInstruction({
    keys: [
      {
        pubkey: vmStateKeypair.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: payer.publicKey,
        isSigner: true,
        isWritable: false,
      },
    ],
    programId: new PublicKey(FIVE_VM_PROGRAM_ID),
    data: Buffer.from([0]), // Initialize discriminator
  })
);
console.log('✓ Added initialize instruction for VM state');

// 3) Create script account
tx.add(
  SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: scriptKeypair.publicKey,
    lamports: scriptRent,
    space: scriptSpace,
    programId: new PublicKey(FIVE_VM_PROGRAM_ID),
  })
);
console.log('✓ Added createAccount instruction for script');

// 4) Deploy instruction with mock bytecode
const mockBytecode = Buffer.alloc(1000); // Mock bytecode
mockBytecode[0] = 1; // Discriminator for deploy
tx.add(
  new TransactionInstruction({
    keys: [
      {
        pubkey: scriptKeypair.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: vmStateKeypair.publicKey,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: payer.publicKey,
        isSigner: true,
        isWritable: true,
      },
    ],
    programId: new PublicKey(FIVE_VM_PROGRAM_ID),
    data: mockBytecode,
  })
);
console.log('✓ Added deploy instruction');

const { blockhash } = await connection.getLatestBlockhash('confirmed');
tx.recentBlockhash = blockhash;
tx.feePayer = payer.publicKey;

tx.partialSign(payer);
tx.partialSign(vmStateKeypair);
tx.partialSign(scriptKeypair);

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
  console.log(`[${elapsed}ms] Status:`, status?.value ? 'CONFIRMED' : 'PENDING');
  if (status?.value) {
    console.log('✅ Multi-instruction transaction confirmed!');
    console.log('  Confirmation result:', status.value);
    found = true;
    break;
  }
  await new Promise(resolve => setTimeout(resolve, 500));
}

if (!found) {
  console.log('❌ Transaction not confirmed after 10 seconds');
  console.log('Transaction signature for investigation:', signature);
  process.exit(1);
}
