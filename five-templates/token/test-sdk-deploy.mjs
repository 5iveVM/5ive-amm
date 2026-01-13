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

console.log('Deploying token with SDK (large program)...');

// Load token bytecode from artifact
const tokenArtifact = JSON.parse(fs.readFileSync('build/five-token-template.five', 'utf-8'));
const bytecodeBase64 = tokenArtifact.bytecode;
const bytecode = Buffer.from(bytecodeBase64, 'base64');

console.log('Bytecode size:', bytecode.length, 'bytes');

// Deploy using SDK (use optimized large program deployment)
const result = await FiveSDK.deployLargeProgramOptimizedToSolana(
  bytecode,
  connection,
  deployerKeypair,
  {
    fiveVMProgramId: 'AjEcViqBu32FiBV25pgoPeTC4DQtNwD6tkPfwCWa6NfN',
    debug: true,
  }
);

if (result.success) {
  console.log('\n✅ Deployment successful!');
  console.log('\nFull result:', JSON.stringify(result, null, 2));
} else {
  console.log('\n❌ Deployment failed:', result.error);
  process.exit(1);
}
