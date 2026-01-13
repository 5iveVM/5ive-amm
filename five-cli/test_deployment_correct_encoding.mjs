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

console.log('Testing deployment with correct encoding (writeUInt32LE)...');
console.log('Payer:', payer.publicKey.toBase58());

// Load counter bytecode
const counterArtifact = JSON.parse(fs.readFileSync(
  '../five-templates/counter/build/five-counter-template.five',
  'utf-8'
));
const bytecodeBase64 = counterArtifact.bytecode;
const bytecode = Buffer.from(bytecodeBase64, 'base64');
console.log('Bytecode size:', bytecode.length, 'bytes');

// Create VM state keypair
const vmStateKeypair = Keypair.generate();
const vmStateSpace = 1024;
const vmStateRent = await connection.getMinimumBalanceForRentExemption(vmStateSpace);

// Create script keypair
const scriptKeypair = Keypair.generate();
const scriptSpace = 65536;
const scriptRent = await connection.getMinimumBalanceForRentExemption(scriptSpace);

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

// 4) Deploy instruction - USE writeUInt32LE
const lengthBuffer = Buffer.allocUnsafe(4);
lengthBuffer.writeUInt32LE(bytecode.length, 0);

const deployData = Buffer.concat([
  Buffer.from([8]), // Discriminator 8
  lengthBuffer, // u32 LE length
  Buffer.from([0]), // permissions
  bytecode // actual bytecode
]);
console.log('Deploy instruction data size:', deployData.length, 'bytes');

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

const serialized = tx.serialize();
console.log('Transaction size:', serialized.length, 'bytes');

try {
  const signature = await connection.sendRawTransaction(serialized, {
    skipPreflight: false,
    maxRetries: 3,
  });

  console.log('\n✅ Transaction sent:', signature);
  
  // Poll for confirmation
  const startTime = Date.now();
  let found = false;
  while (Date.now() - startTime < 10000) {
    const status = await connection.getSignatureStatus(signature);
    const elapsed = Date.now() - startTime;
    if (status?.value) {
      console.log(`[${elapsed}ms] ✅ CONFIRMED`);
      if (status.value.err) {
        console.log('Transaction failed:', status.value.err);
        process.exit(1);
      }
      console.log('Script deployed successfully!');
      console.log('Script account:', scriptKeypair.publicKey.toBase58());
      found = true;
      break;
    }
    console.log(`[${elapsed}ms] PENDING`);
    await new Promise(resolve => setTimeout(resolve, 500));
  }

  if (!found) {
    console.log('❌ Timeout after 10 seconds');
    process.exit(1);
  }
} catch (e) {
  console.log('❌ Error:', e.message);
  if (e.logs) {
    console.log('Logs:', e.logs.slice(-10));
  }
  process.exit(1);
}
