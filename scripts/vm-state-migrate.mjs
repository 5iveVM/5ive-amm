#!/usr/bin/env node
import { readFile } from 'node:fs/promises';
import web3 from '../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';
import { loadClusterConfig, deriveVmAddresses, resolveClusterFromEnvOrDefault } from './lib/vm-cluster-config.mjs';

const {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} = web3;

function readArg(name, fallback = undefined) {
  const flag = `--${name}`;
  const idx = process.argv.indexOf(flag);
  if (idx >= 0 && idx + 1 < process.argv.length) return process.argv[idx + 1];
  return fallback;
}

const rpcUrl = readArg('rpc-url', 'http://127.0.0.1:8899');
const cluster = readArg('cluster', resolveClusterFromEnvOrDefault());
const profile = loadClusterConfig({ cluster });
const derived = deriveVmAddresses(profile);
const programIdRaw = readArg('program-id', profile.programId);
const vmStateRaw = readArg('vm-state', derived.vmStatePda);
const keypairPath = readArg('keypair');

if (!keypairPath) {
  console.error(
    'usage: node scripts/vm-state-migrate.mjs ' +
      '--keypair <path> ' +
      '[--cluster localnet|devnet|mainnet] [--rpc-url ...] [--program-id ...] [--vm-state ...]'
  );
  process.exit(2);
}

const secret = JSON.parse(await readFile(keypairPath, 'utf8'));
const signer = Keypair.fromSecretKey(Uint8Array.from(secret));
const connection = new Connection(rpcUrl, 'confirmed');
const programId = new PublicKey(programIdRaw);
const vmState = new PublicKey(vmStateRaw);

const before = await connection.getAccountInfo(vmState, 'confirmed');
if (!before) {
  console.error(`vm_state account not found: ${vmState.toBase58()}`);
  process.exit(1);
}

const data = Buffer.from([15]); // MigrateVmState discriminator
const ix = new TransactionInstruction({
  programId,
  keys: [
    { pubkey: vmState, isSigner: false, isWritable: true },
    { pubkey: signer.publicKey, isSigner: true, isWritable: false }, // authority
    { pubkey: signer.publicKey, isSigner: true, isWritable: true },  // payer
  ],
  data,
});

const tx = new Transaction().add(ix);
const sig = await sendAndConfirmTransaction(connection, tx, [signer], { commitment: 'confirmed' });

const after = await connection.getAccountInfo(vmState, 'confirmed');

console.log('VM_STATE_MIGRATE_OK');
console.log(`  signature: ${sig}`);
console.log(`  vm_state: ${vmState.toBase58()}`);
console.log(`  authority: ${signer.publicKey.toBase58()}`);
console.log(`  size_before: ${before.data.length}`);
console.log(`  size_after: ${after?.data.length ?? 0}`);
