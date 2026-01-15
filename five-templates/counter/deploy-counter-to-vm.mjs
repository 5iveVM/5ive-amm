import { FiveSDK } from '../../five-sdk/dist/index.js';
import { Connection, Keypair } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const RPC_URL = 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = 'FyCrH1Zfoo55f3qDuAZ22hma2E8ePFUXSP5JF1QbQdXE';
const VM_STATE_PDA = '9tsp9D3GfDmu3R2P4aXuaKYa6YRLtreZqvpoyirp6UZF';

const connection = new Connection(RPC_URL, 'confirmed');
const keypairPath = process.env.HOME + '/.config/solana/id.json';
const keypairBuffer = fs.readFileSync(keypairPath);
const keypairArray = JSON.parse(keypairBuffer.toString());
const payer = Keypair.fromSecretKey(new Uint8Array(keypairArray));

console.log('Deploying counter...');
console.log('Program ID:', FIVE_PROGRAM_ID);
console.log('VM State:', VM_STATE_PDA);
console.log('Payer:', payer.publicKey.toBase58());

// Load counter bytecode
const counterArtifact = JSON.parse(fs.readFileSync(
  path.join(__dirname, 'build/five-counter-template.five'),
  'utf-8'
));
const bytecodeBase64 = counterArtifact.bytecode;
const bytecode = Buffer.from(bytecodeBase64, 'base64');

console.log('Bytecode size:', bytecode.length, 'bytes');

try {
  const result = await FiveSDK.deployToSolana(
    bytecode,
    connection,
    payer,
    {
      debug: true,
      fiveVMProgramId: FIVE_PROGRAM_ID,
      vmStateAccount: VM_STATE_PDA,
    }
  );

  if (result.success) {
    console.log('\n✅ Deployment successful!');
    console.log('Script account:', result.programId);
    console.log('Transaction:', result.transactionId);
    
    // Update deployment config
    const config = {
      fiveProgramId: FIVE_PROGRAM_ID,
      vmStatePda: VM_STATE_PDA,
      counterScriptAccount: result.programId,
      rpcUrl: RPC_URL,
      timestamp: new Date().toISOString()
    };
    
    fs.writeFileSync(
      path.join(__dirname, 'deployment-config.json'),
      JSON.stringify(config, null, 2)
    );
    console.log('✅ Updated deployment-config.json');
  } else {
    console.log('\n❌ Deployment failed:', result.error);
    process.exit(1);
  }
} catch (error) {
  console.log('\n❌ Error:', error.message);
  process.exit(1);
}
