const fs = require('fs');
const path = require('path');

const modulePaths = [
  process.cwd(),
  path.join(process.cwd(), 'node_modules'),
  path.join(process.cwd(), 'five-cli', 'node_modules'),
];

const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} = require(require.resolve('@solana/web3.js', { paths: modulePaths }));

async function main() {
  const [rpcUrl, programIdRaw, payerPath, vmStatePath, vmStateSizeRaw] = process.argv.slice(2);
  console.log(`RPC: ${rpcUrl}`);
  console.log(`Program: ${programIdRaw}`);
  console.log(`Payer: ${payerPath}`);
  console.log(`VM State: ${vmStatePath}`);

  const programId = new PublicKey(programIdRaw);
  const vmStateSize = parseInt(vmStateSizeRaw, 10);

  const payer = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(payerPath, 'utf8')))
  );
  const vmState = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(vmStatePath, 'utf8')))
  );

  const connection = new Connection(rpcUrl, 'confirmed');
  const rentExempt = await connection.getMinimumBalanceForRentExemption(vmStateSize);
  const accountInfo = await connection.getAccountInfo(vmState.publicKey);

  const tx = new Transaction();
  const signers = [payer];

  if (!accountInfo) {
    tx.add(
      SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: vmState.publicKey,
        lamports: rentExempt,
        space: vmStateSize,
        programId,
      })
    );
    signers.push(vmState);
    console.log(`📦 Created VM state account ${vmState.publicKey.toBase58()}`);
  } else {
    console.log(`Account info found for ${vmState.publicKey.toBase58()}:`, accountInfo);
    if (accountInfo.owner.toBase58() !== programId.toBase58()) {
      throw new Error(`Existing VM state account is not owned by the FIVE program. Current owner: ${accountInfo.owner.toBase58()}`);
    }
    if (accountInfo.data.length < vmStateSize) {
      throw new Error('Existing VM state account is too small');
    }
    if (accountInfo.lamports < rentExempt) {
      tx.add(
        SystemProgram.transfer({
          fromPubkey: payer.publicKey,
          toPubkey: vmState.publicKey,
          lamports: rentExempt - accountInfo.lamports,
        })
      );
      console.log('🔄 Topped up VM state account to rent exemption');
    }
    if (accountInfo.data.length >= 56 && accountInfo.data[52] === 1 && tx.instructions.length === 0) {
      console.log(`✅ VM state already initialized: ${vmState.publicKey.toBase58()}`);
      console.log(`READY:${vmState.publicKey.toBase58()}`);
      return;
    }
  }

  tx.add(
    new TransactionInstruction({
      keys: [
        { pubkey: vmState.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      ],
      programId,
      data: Buffer.from([0]), // Initialize discriminator
    })
  );

  const sig = await connection.sendTransaction(tx, signers, { skipPreflight: false });
  await connection.confirmTransaction(sig, 'confirmed');
  console.log(`✅ VM state initialized via tx: ${sig}`);
  console.log(`READY:${vmState.publicKey.toBase58()}`);
}

main().catch((err) => {
  console.error(`VM init error:`);
  console.error(err);
  process.exit(1);
});
