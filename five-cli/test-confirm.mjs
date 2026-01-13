import { Connection, Keypair, Transaction, SystemProgram } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Load keypair
const keypairBuffer = fs.readFileSync(process.env.HOME + '/.config/solana/id.json');
const keypairArray = JSON.parse(keypairBuffer.toString());
const payer = Keypair.fromSecretKey(new Uint8Array(keypairArray));

console.log('Testing transaction confirmation latency...');
console.log('Payer:', payer.publicKey.toBase58());

const start = Date.now();

try {
  // Get latest blockhash
  const { blockhash } = await connection.getLatestBlockhash('confirmed');
  console.log(`Blockhash retrieved: ${blockhash}`);

  // Create a simple transfer transaction
  const recipient = Keypair.generate();
  const tx = new Transaction();
  tx.add(
    SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: recipient.publicKey,
      lamports: 1000000, // 0.001 SOL
    })
  );
  tx.recentBlockhash = blockhash;
  tx.feePayer = payer.publicKey;
  tx.sign(payer);

  const sendStart = Date.now();
  const signature = await connection.sendRawTransaction(tx.serialize(), {
    skipPreflight: true,
    maxRetries: 3,
  });
  const sendTime = Date.now() - sendStart;
  console.log(`Transaction sent in ${sendTime}ms: ${signature}`);

  // Confirm with timeout tracking
  const confirmStart = Date.now();
  const confirmation = await Promise.race([
    connection.confirmTransaction(signature, 'confirmed'),
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Confirmation timeout at 45s')), 45000)
    ),
  ]);
  const confirmTime = Date.now() - confirmStart;

  const totalTime = Date.now() - start;
  console.log(`\n✅ Transaction confirmed!`);
  console.log(`  - Send time: ${sendTime}ms`);
  console.log(`  - Confirm time: ${confirmTime}ms`);
  console.log(`  - Total time: ${totalTime}ms`);
  console.log(`  - Confirmation result:`, confirmation);
} catch (error) {
  const totalTime = Date.now() - start;
  console.error(`\n❌ Error after ${totalTime}ms:`, error.message);
  process.exit(1);
}
