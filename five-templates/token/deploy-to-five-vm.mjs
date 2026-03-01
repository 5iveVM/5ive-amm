import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
  Connection, Keypair, PublicKey, Transaction, TransactionInstruction,
  SystemProgram, LAMPORTS_PER_SOL, ComputeBudgetProgram
} from '@solana/web3.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = process.env.FIVE_RPC_URL || process.env.RPC_URL || '';
if (!RPC_URL) {
  throw new Error('Missing FIVE_RPC_URL (or legacy RPC_URL). Deployment must target an explicit cluster.');
}
const FIVE_PROGRAM_ID_RAW = process.env.FIVE_PROGRAM_ID || process.env.FIVE_VM_PROGRAM_ID || '';
if (!FIVE_PROGRAM_ID_RAW) {
  throw new Error('Missing FIVE_PROGRAM_ID (or legacy FIVE_VM_PROGRAM_ID). Deployment must target an explicit VM program.');
}
const FIVE_PROGRAM_ID = new PublicKey(FIVE_PROGRAM_ID_RAW);
const VM_STATE_PDA = process.env.VM_STATE_PDA || process.env.FIVE_VM_STATE_PDA || '';
if (!VM_STATE_PDA) {
  throw new Error('Missing VM_STATE_PDA (or legacy FIVE_VM_STATE_PDA). Deployment must target an explicit VM state account.');
}
const FEE_VAULT_SEED_PREFIX = Buffer.from([0xff, ...Buffer.from('five_vm_fee_vault_v1')]);
const FEE_VAULT_0 = process.env.FEE_VAULT_ACCOUNT
  ? new PublicKey(process.env.FEE_VAULT_ACCOUNT)
  : PublicKey.findProgramAddressSync([FEE_VAULT_SEED_PREFIX, Buffer.from([0])], FIVE_PROGRAM_ID)[0];

async function deployProgram() {
  const connection = new Connection(RPC_URL, 'confirmed');
  const payerKeyPath = process.env.FIVE_KEYPAIR_PATH || path.join(process.env.HOME, '.config/solana/id.json');
  const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(payerKeyPath, 'utf-8'))));

  const artifactPath = process.env.FIVE_ARTIFACT_PATH || path.join(__dirname, 'build', 'five-token-template.five');
  const parsed = JSON.parse(fs.readFileSync(artifactPath, 'utf-8'));
  const bytecode = new Uint8Array(Buffer.from(parsed.bytecode, 'base64'));

  const balance = await connection.getBalance(payer.publicKey);
  if (balance < 0.1 * LAMPORTS_PER_SOL) throw new Error('Insufficient balance');

  const vmStatePda = new PublicKey(VM_STATE_PDA);
  const vmStateInfo = await connection.getAccountInfo(vmStatePda);
  if (!vmStateInfo || !vmStateInfo.owner.equals(FIVE_PROGRAM_ID)) {
    throw new Error('VM state missing or owned by wrong program');
  }

  const confirmTx = async (signature, description) => {
    const latestBlockhash = await connection.getLatestBlockhash();
    const confirmation = await connection.confirmTransaction({ signature, ...latestBlockhash }, 'confirmed');
    if (confirmation.value.err) throw new Error(`${description} failed: ${JSON.stringify(confirmation.value.err)}`);
  };

  const scriptKeypair = Keypair.generate();
  const SCRIPT_HEADER_SIZE = 64;
  const finalScriptSize = SCRIPT_HEADER_SIZE + bytecode.length;
  const rentRequired = await connection.getMinimumBalanceForRentExemption(finalScriptSize);
  const initialLamports = rentRequired + 0.01 * LAMPORTS_PER_SOL;

  const initTx = new Transaction().add(
    ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: scriptKeypair.publicKey,
      lamports: initialLamports,
      space: finalScriptSize,
      programId: FIVE_PROGRAM_ID,
    }),
    new TransactionInstruction({
      keys: [
        { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: vmStatePda, isSigner: false, isWritable: true },
        { pubkey: FEE_VAULT_0, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: FIVE_PROGRAM_ID,
      data: Buffer.concat([Buffer.from([4]), Buffer.from(new Uint32Array([bytecode.length]).buffer)]),
    })
  );

  const initSig = await connection.sendTransaction(initTx, [payer, scriptKeypair], { skipPreflight: true });
  await confirmTx(initSig, 'Script Account Init');

  const CHUNK_SIZE = 380;
  for (let i = 0; i < bytecode.length; i += CHUNK_SIZE) {
    const chunk = bytecode.slice(i, Math.min(i + CHUNK_SIZE, bytecode.length));
    const appendTx = new Transaction().add(
      ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
      new TransactionInstruction({
        keys: [
          { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
          { pubkey: payer.publicKey, isSigner: true, isWritable: true },
          { pubkey: vmStatePda, isSigner: false, isWritable: true },
          { pubkey: FEE_VAULT_0, isSigner: false, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId: FIVE_PROGRAM_ID,
        data: Buffer.concat([Buffer.from([5]), chunk]),
      })
    );
    const appendSig = await connection.sendTransaction(appendTx, [payer], { skipPreflight: true });
    await confirmTx(appendSig, `Chunk ${Math.floor(i / CHUNK_SIZE) + 1} append`);
  }

  console.log(`tokenScriptAccount=${scriptKeypair.publicKey.toBase58()}`);
  console.log(`fiveProgramId=${FIVE_PROGRAM_ID.toBase58()}`);
  console.log(`vmStatePda=${vmStatePda.toBase58()}`);
  console.log(`rpcUrl=${RPC_URL}`);
}

deployProgram().catch((e) => { console.error(e.message || e); process.exit(1); });
