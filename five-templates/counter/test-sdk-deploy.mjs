import { FiveSDK } from '../../five-sdk/dist/index.js';
import { Connection, Keypair } from '@solana/web3.js';
import fs from 'fs';

const RPC_URL = 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

// Load keypair
const keypairPath = process.env.HOME + '/.config/solana/id.json';
const keypairBuffer = fs.readFileSync(keypairPath);
const keypairArray = JSON.parse(keypairBuffer.toString());
const deployerKeypair = Keypair.fromSecretKey(new Uint8Array(keypairArray));

console.log('Deploying counter with SDK...');
console.log('Deployer:', deployerKeypair.publicKey.toBase58());

// Load counter bytecode from artifact
const counterArtifact = JSON.parse(fs.readFileSync(
  'build/five-counter-template.five',
  'utf-8'
));
const bytecodeBase64 = counterArtifact.bytecode;
const bytecode = Buffer.from(bytecodeBase64, 'base64');

console.log('Bytecode size:', bytecode.length, 'bytes');

// Deploy using SDK
try {
  const result = await FiveSDK.deployToSolana(
    bytecode,
    connection,
    deployerKeypair,
    {
      debug: true,
      fiveVMProgramId: 'AjEcViqBu32FiBV25pgoPeTC4DQtNwD6tkPfwCWa6NfN',
    }
  );

  if (result.success) {
    console.log('\n✅ Deployment successful!');
    console.log('Script account:', result.programId);
    console.log('Transaction:', result.transactionId);
  } else {
    console.log('\n❌ Deployment failed:', result.error);
    process.exit(1);
  }
} catch (error) {
  console.log('\n❌ Error:', error.message);
  process.exit(1);
}
