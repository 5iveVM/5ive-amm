import { Connection, Keypair, Transaction, TransactionInstruction, SystemProgram, sendAndConfirmRawTransaction } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Load keypair
const keypairBuffer = fs.readFileSync(process.env.HOME + '/.config/solana/id.json');
const keypairArray = JSON.parse(keypairBuffer.toString());
const payer = Keypair.fromSecretKey(new Uint8Array(keypairArray));

console.log('Testing deployment-like transaction...');
console.log('Payer:', payer.publicKey.toBase58());

const FIVE_VM_PROGRAM = 'AjEcViqBu32FiBV25pgoPeTC4DQtNwD6tkPfwCWa6NfN';

try {
  const { PublicKey } = await import('@solana/web3.js');
  
  // Create test keypairs
  const scriptKeypair = Keypair.generate();
  const vmStateKeypair = Keypair.generate();
  
  console.log('Script account:', scriptKeypair.publicKey.toBase58());
  console.log('VM state account:', vmStateKeypair.publicKey.toBase58());
  
  // Get blockhash
  const { blockhash } = await connection.getLatestBlockhash('confirmed');
  
  // Build transaction
  const tx = new Transaction();
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  
  // Add compute budget
  const { ComputeBudgetProgram } = await import('@solana/web3.js');
  tx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }));
  
  // Add test instruction
  const testIx = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true }
    ],
    programId: new PublicKey(FIVE_VM_PROGRAM),
    data: Buffer.from([0])
  });
  tx.add(testIx);
  
  tx.sign(payer);
  
  console.log('Transaction size:', tx.serialize().length, 'bytes');
  console.log('Instructions:', tx.instructions.length);
  
  const sendStart = Date.now();
  const signature = await connection.sendRawTransaction(tx.serialize(), {
    skipPreflight: true,
    maxRetries: 3,
  });
  const sendTime = Date.now() - sendStart;
  console.log('Transaction sent in', sendTime + 'ms:', signature.substring(0, 20) + '...');
  
  // Test confirmation
  console.log('Testing confirmation with 45s timeout...');
  
  const confirmStart = Date.now();
  try {
    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Timeout at 45s')), 45000)
    );
    const confirmation = await Promise.race([
      connection.confirmTransaction(signature, 'confirmed'),
      timeoutPromise
    ]);
    const confirmTime = Date.now() - confirmStart;
    console.log('Confirmed in', confirmTime + 'ms');
    if (confirmation.value.err) {
      console.log('Transaction failed:', JSON.stringify(confirmation.value.err));
    }
  } catch (e) {
    const confirmTime = Date.now() - confirmStart;
    console.log('Confirmation failed after', confirmTime + 'ms:', e.message);
  }
  
} catch (error) {
  console.error('Test failed:', error);
  process.exit(1);
}
