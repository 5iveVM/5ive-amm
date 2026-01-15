import { FiveSDK } from '../../five-sdk/dist/index.js';
import { FiveProgram } from '../../five-sdk/dist/index.js';
import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, LAMPORTS_PER_SOL } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const RPC_URL = 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = 'Dk1bmmhyYXTiJmZPGX16mfYhd1or4tzZkgUkSSoytThv';
const VM_STATE_PDA = 'ZVM9MibJQ4GeMFm8n1jKdZ4JeZnJsk9GNSCVQSUzu8d';

const connection = new Connection(RPC_URL, 'confirmed');
const keypairPath = process.env.HOME + '/.config/solana/id.json';
const keypairBuffer = fs.readFileSync(keypairPath);
const keypairArray = JSON.parse(keypairBuffer.toString());
const payer = Keypair.fromSecretKey(new Uint8Array(keypairArray));

console.log('=== DEPLOYMENT ===');
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
const abi = counterArtifact.abi;

console.log('Bytecode size:', bytecode.length, 'bytes');

try {
  const result = await FiveSDK.deployToSolana(
    bytecode,
    connection,
    payer,
    {
      debug: false,
      fiveVMProgramId: FIVE_PROGRAM_ID,
      vmStateAccount: VM_STATE_PDA,
    }
  );

  if (result.success) {
    console.log('\n✅ Deployment successful!');
    console.log('Script account:', result.programId);
    console.log('DEPLOY_TX:', result.transactionId);

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

    // Now run tests
    console.log('\n=== TESTS ===');

    const program = FiveProgram.fromABI(result.programId, abi, {
      debug: false,
      fiveVMProgramId: FIVE_PROGRAM_ID,
      vmStateAccount: VM_STATE_PDA,
      feeReceiverAccount: payer.publicKey.toBase58()
    });

    const user1 = Keypair.generate();
    const user2 = Keypair.generate();

    // Fund users
    let sig = await connection.requestAirdrop(user1.publicKey, 1000 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, 'confirmed');
    sig = await connection.requestAirdrop(user2.publicKey, 1000 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig, 'confirmed');

    const [counter1Account] = PublicKey.findProgramAddressSync(
      [Buffer.from('counter'), user1.publicKey.toBuffer()],
      new PublicKey(FIVE_PROGRAM_ID)
    );

    const [counter2Account] = PublicKey.findProgramAddressSync(
      [Buffer.from('counter'), user2.publicKey.toBuffer()],
      new PublicKey(FIVE_PROGRAM_ID)
    );

    const txSignatures = [];

    // Initialize counter1
    const ix1 = await program.function('initialize')
      .accounts({ counter: counter1Account.toBase58(), owner: user1.publicKey.toBase58() })
      .args({})
      .instruction();

    const tx1 = new Transaction().add(new TransactionInstruction({
      programId: new PublicKey(ix1.programId),
      keys: ix1.keys.map(k => ({ pubkey: new PublicKey(k.pubkey), isSigner: k.isSigner, isWritable: k.isWritable })),
      data: Buffer.from(ix1.data, 'base64')
    }));

    sig = await connection.sendTransaction(tx1, [payer, user1], { skipPreflight: true, maxRetries: 3 });
    await connection.confirmTransaction(sig, 'confirmed');
    txSignatures.push({ test: 'initialize1', signature: sig });
    console.log('INIT1_TX:', sig);

    // Initialize counter2
    const ix2 = await program.function('initialize')
      .accounts({ counter: counter2Account.toBase58(), owner: user2.publicKey.toBase58() })
      .args({})
      .instruction();

    const tx2 = new Transaction().add(new TransactionInstruction({
      programId: new PublicKey(ix2.programId),
      keys: ix2.keys.map(k => ({ pubkey: new PublicKey(k.pubkey), isSigner: k.isSigner, isWritable: k.isWritable })),
      data: Buffer.from(ix2.data, 'base64')
    }));

    sig = await connection.sendTransaction(tx2, [payer, user2], { skipPreflight: true, maxRetries: 3 });
    await connection.confirmTransaction(sig, 'confirmed');
    txSignatures.push({ test: 'initialize2', signature: sig });
    console.log('INIT2_TX:', sig);

    // Increment counter1 3 times
    for (let i = 1; i <= 3; i++) {
      const ix = await program.function('increment')
        .accounts({ counter: counter1Account.toBase58(), owner: user1.publicKey.toBase58() })
        .args({})
        .instruction();

      const tx = new Transaction().add(new TransactionInstruction({
        programId: new PublicKey(ix.programId),
        keys: ix.keys.map(k => ({ pubkey: new PublicKey(k.pubkey), isSigner: k.isSigner, isWritable: k.isWritable })),
        data: Buffer.from(ix.data, 'base64')
      }));

      sig = await connection.sendTransaction(tx, [payer, user1], { skipPreflight: true, maxRetries: 3 });
      await connection.confirmTransaction(sig, 'confirmed');
      txSignatures.push({ test: `increment1_${i}`, signature: sig });
      console.log(`INC1_${i}_TX:`, sig);
    }

    // Add 10 to counter1
    const ix_add = await program.function('add_amount')
      .accounts({ counter: counter1Account.toBase58(), owner: user1.publicKey.toBase58() })
      .args({ amount: 10 })
      .instruction();

    const tx_add = new Transaction().add(new TransactionInstruction({
      programId: new PublicKey(ix_add.programId),
      keys: ix_add.keys.map(k => ({ pubkey: new PublicKey(k.pubkey), isSigner: k.isSigner, isWritable: k.isWritable })),
      data: Buffer.from(ix_add.data, 'base64')
    }));

    sig = await connection.sendTransaction(tx_add, [payer, user1], { skipPreflight: true, maxRetries: 3 });
    await connection.confirmTransaction(sig, 'confirmed');
    txSignatures.push({ test: 'add_amount', signature: sig });
    console.log('ADD10_TX:', sig);

    // Decrement counter1
    const ix_dec = await program.function('decrement')
      .accounts({ counter: counter1Account.toBase58(), owner: user1.publicKey.toBase58() })
      .args({})
      .instruction();

    const tx_dec = new Transaction().add(new TransactionInstruction({
      programId: new PublicKey(ix_dec.programId),
      keys: ix_dec.keys.map(k => ({ pubkey: new PublicKey(k.pubkey), isSigner: k.isSigner, isWritable: k.isWritable })),
      data: Buffer.from(ix_dec.data, 'base64')
    }));

    sig = await connection.sendTransaction(tx_dec, [payer, user1], { skipPreflight: true, maxRetries: 3 });
    await connection.confirmTransaction(sig, 'confirmed');
    txSignatures.push({ test: 'decrement', signature: sig });
    console.log('DEC_TX:', sig);

    // Increment counter2 5 times
    for (let i = 1; i <= 5; i++) {
      const ix = await program.function('increment')
        .accounts({ counter: counter2Account.toBase58(), owner: user2.publicKey.toBase58() })
        .args({})
        .instruction();

      const tx = new Transaction().add(new TransactionInstruction({
        programId: new PublicKey(ix.programId),
        keys: ix.keys.map(k => ({ pubkey: new PublicKey(k.pubkey), isSigner: k.isSigner, isWritable: k.isWritable })),
        data: Buffer.from(ix.data, 'base64')
      }));

      sig = await connection.sendTransaction(tx, [payer, user2], { skipPreflight: true, maxRetries: 3 });
      await connection.confirmTransaction(sig, 'confirmed');
      txSignatures.push({ test: `increment2_${i}`, signature: sig });
      console.log(`INC2_${i}_TX:`, sig);
    }

    // Reset counter2
    const ix_reset = await program.function('reset')
      .accounts({ counter: counter2Account.toBase58(), owner: user2.publicKey.toBase58() })
      .args({})
      .instruction();

    const tx_reset = new Transaction().add(new TransactionInstruction({
      programId: new PublicKey(ix_reset.programId),
      keys: ix_reset.keys.map(k => ({ pubkey: new PublicKey(k.pubkey), isSigner: k.isSigner, isWritable: k.isWritable })),
      data: Buffer.from(ix_reset.data, 'base64')
    }));

    sig = await connection.sendTransaction(tx_reset, [payer, user2], { skipPreflight: true, maxRetries: 3 });
    await connection.confirmTransaction(sig, 'confirmed');
    txSignatures.push({ test: 'reset', signature: sig });
    console.log('RESET_TX:', sig);

    console.log('\n=== ALL SIGNATURES ===');
    txSignatures.forEach(tx => console.log(`${tx.test}: ${tx.signature}`));
  } else {
    console.log('\n❌ Deployment failed:', result.error);
    process.exit(1);
  }
} catch (error) {
  console.log('\n❌ Error:', error.message);
  console.error(error);
  process.exit(1);
}
