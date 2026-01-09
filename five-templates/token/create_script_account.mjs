import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const connection = new Connection('http://127.0.0.1:8899', 'confirmed');
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';
const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'))));
const FIVE_PROGRAM_ID = new PublicKey('FJERHDufQjbHvXjYuoprJQhKY4413cTPKhbbNSSw4tBg');

// Create a new keypair for the script account
const scriptKeypair = Keypair.generate();
const scriptPubkey = scriptKeypair.publicKey;

console.log('New script account:', scriptPubkey.toBase58());

// Create Initialize instruction (discriminator 0x00)
const ix = new TransactionInstruction({
  programId: FIVE_PROGRAM_ID,
  keys: [
    { pubkey: scriptPubkey, isSigner: true, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    { pubkey: payer.publicKey, isSigner: true, isWritable: true }
  ],
  data: Buffer.from([0x00]) // Initialize discriminator
});

const tx = new Transaction().add(ix);
const sig = await connection.sendTransaction(tx, [payer, scriptKeypair], { skipPreflight: false });
console.log('Initialize signature:', sig);

await connection.confirmTransaction(sig, 'confirmed');
console.log('✅ Script account created successfully');

// Update deployment-config.json with new script account
const configPath = path.join(__dirname, 'five-templates/token/deployment-config.json');
const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
config.tokenScriptAccount = scriptPubkey.toBase58();
fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
console.log('Updated deployment-config.json');
