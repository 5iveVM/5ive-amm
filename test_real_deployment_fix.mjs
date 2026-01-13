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

console.log('Testing deployment with real counter bytecode...');
console.log('Payer:', payer.publicKey.toBase58());

// Load counter bytecode
const counterArtifact = JSON.parse(fs.readFileSync(
  '/Users/amberjackson/Documents/Development/five-org/five-mono/five-templates/counter/build/five-counter-template.five',
  'utf-8'
));
const bytecodeHex = counterArtifact.bytecode;
const bytecode = Buffer.from(bytecodeHex, 'hex');
console.log('Bytecode size:', bytecode.length, 'bytes');

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
    data: Buffer.from([0]),
  })
);

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

// 4) Deploy instruction with real bytecode - DISCRIMINATOR 8
const deployData = Buffer.concat([
  Buffer.from([8]), // Discriminator 8 for Deploy
  Buffer.from([bytecode.length, 0, 0, 0]), // u32 LE length
  Buffer.from([0]), // permissions = 0
  bytecode  // actual bytecode
]);
console.log('\nDeploy instruction:');
console.log('  Discriminator:', deployData[0]);
console.log('  Bytecode length:', bytecode.length);
console.log('  Total instruction data:', deployData.length, 'bytes');

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
    data: deployData,
  })
);

const { blockhash } = await connection.getLatestBlockhash('confirmed');
tx.recentBlockhash = blockhash;
tx.feePayer = payer.publicKey;

tx.partialSign(payer);
tx.partialSign(vmStateKeypair);
tx.partialSign(scriptKeypair);

console.log('\nTransaction details:');
console.log('  Signatures:', tx.signatures.length);
console.log('  Instructions:', tx.instructions.length);

try {
  const serialized = tx.serialize();
  console.log('  Size:', serialized.length, 'bytes');
  console.log('  Within limit:', serialized.length <= 1232 ? '✅ YES' : '❌ NO');
  
  const signature = await connection.sendRawTransaction(serialized, {
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
      console.log('✅ Deployment transaction confirmed!');
      if (status.value.err) {
        console.log('  Error:', status.value.err);
      } else {
        console.log('  Script deployed successfully!');
        console.log('  Script account:', scriptKeypair.publicKey.toBase58());
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
} catch (e) {
  console.log('❌ Error:', e.message);
  process.exit(1);
}
