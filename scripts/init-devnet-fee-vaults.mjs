#!/usr/bin/env node

/**
 * Initialize fee vault shards on localnet/devnet/mainnet.
 * This must be run before deploying scripts to ensure fee collection accounts exist.
 */

import { readFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import web3 from '../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';
import { loadClusterConfig, resolveClusterFromEnvOrDefault } from './lib/vm-cluster-config.mjs';
const { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction } = web3;
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));

function parseArgs(argv) {
  const args = { network: 'devnet' };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--network' && argv[i + 1]) args.network = argv[++i];
    else if (a === '--rpc-url' && argv[i + 1]) args.rpcUrl = argv[++i];
    else if (a === '--program-id' && argv[i + 1]) args.programId = argv[++i];
    else if (a === '--vm-state' && argv[i + 1]) args.vmState = argv[++i];
    else if (a === '--keypair' && argv[i + 1]) args.keypairPath = argv[++i];
    else if (a === '--shards' && argv[i + 1]) args.shards = Number(argv[++i]);
    else if (a === '--strict') args.strict = true;
  }
  return args;
}

function defaultsForNetwork(network) {
  if (network === 'localnet') return { rpcUrl: 'http://127.0.0.1:8899' };
  if (network === 'mainnet' || network === 'mainnet-beta') {
    return { rpcUrl: 'https://api.mainnet-beta.solana.com' };
  }
  return { rpcUrl: 'https://api.devnet.solana.com' };
}

// Fee vault seed matches Rust: b"\xFFfive_vm_fee_vault_v1"
const FEE_VAULT_SEED = Buffer.from([
  0xff, 0x66, 0x69, 0x76, 0x65, 0x5f, 0x76, 0x6d, 0x5f, 0x66, 0x65, 0x65,
  0x5f, 0x76, 0x61, 0x75, 0x6c, 0x74, 0x5f, 0x76, 0x31,
]);

const INIT_FEE_VAULT_INSTRUCTION = 11;

async function deriveFeeVault(programId, shardIndex) {
  const [pda, bump] = PublicKey.findProgramAddressSync(
    [FEE_VAULT_SEED, Buffer.from([shardIndex])],
    programId
  );
  return { address: pda.toBase58(), bump };
}

async function main() {
  const args = parseArgs(process.argv);
  const cluster = args.network === 'localnet' ? 'localnet' : (args.network === 'mainnet' || args.network === 'mainnet-beta' ? 'mainnet' : 'devnet');
  const defaults = defaultsForNetwork(args.network);
  const rpcUrl = args.rpcUrl || process.env.FIVE_RPC_URL || defaults.rpcUrl;
  const clusterConfig = loadClusterConfig({ cluster: process.env.FIVE_VM_CLUSTER || cluster || resolveClusterFromEnvOrDefault() });
  const configProgramId = clusterConfig.programId;
  const configShardCount = clusterConfig.feeVaultShardCount;
  const programIdRaw = args.programId || process.env.FIVE_PROGRAM_ID || configProgramId;
  const programId = new PublicKey(programIdRaw);
  const shardCount = Number.isFinite(args.shards) && args.shards > 0
    ? args.shards
    : configShardCount;

  const connection = new Connection(rpcUrl, 'confirmed');

  // Load payer keypair
  const keypairPath =
    args.keypairPath ||
    process.env.FIVE_KEYPAIR_PATH ||
    path.join(os.homedir(), '.config/solana/id.json');
  if (!fs.existsSync(keypairPath)) {
    throw new Error(`Keypair not found at ${keypairPath}`);
  }

  const keypairData = JSON.parse(await readFile(keypairPath, 'utf-8'));
  const payer = Keypair.fromSecretKey(Uint8Array.from(keypairData));

  const vmStateRaw = args.vmState || process.env.FIVE_VM_STATE;
  const [derivedVmState] = PublicKey.findProgramAddressSync(
    [Buffer.from('vm_state', 'utf-8')],
    programId
  );
  const vmState = vmStateRaw ? new PublicKey(vmStateRaw) : derivedVmState;

  console.log(`\n📋 Initializing Fee Vault Shards on ${args.network.toUpperCase()}`);
  console.log(`   RPC: ${rpcUrl}`);
  console.log(`   Payer: ${payer.publicKey.toBase58()}`);
  console.log(`   Program ID: ${programId.toBase58()}`);
  console.log(`   VM State: ${vmState.toBase58()}`);
  console.log(`   Shards: ${shardCount}\n`);

  // Check VM state account exists
  const vmStateInfo = await connection.getAccountInfo(vmState);
  if (!vmStateInfo) {
    throw new Error(`VM State account ${vmState.toBase58()} not found on ${args.network}`);
  }
  console.log(`✓ VM State account found (${vmStateInfo.data.length} bytes)`);

  if (args.strict) {
    console.log('⏳ Strict parity precheck (compiled constants vs chain)...');
    const result = spawnSync('node', [
      path.join(SCRIPT_DIR, 'check-vm-constants-parity.mjs'),
      '--rpc-url', rpcUrl,
      '--program-id', programId.toBase58(),
      '--vm-state', vmState.toBase58(),
    ], { stdio: 'inherit' });
    if (result.status !== 0) {
      throw new Error('Strict parity precheck failed');
    }
  }

  // Get current balance
  const balance = await connection.getBalance(payer.publicKey);
  console.log(`✓ Payer balance: ${balance / 1e9} SOL\n`);
  // Discover missing shards first so we can estimate exact minimum balance.
  const missingShards = [];
  for (let shardIndex = 0; shardIndex < shardCount; shardIndex++) {
    const vault = await deriveFeeVault(programId, shardIndex);
    const vaultPubkey = new PublicKey(vault.address);
    const vaultInfo = await connection.getAccountInfo(vaultPubkey);
    if (vaultInfo) {
      console.log(`✓ Shard ${shardIndex}: Already initialized (${vault.address})`);
    } else {
      missingShards.push({ shardIndex, vault, vaultPubkey });
    }
  }

  if (missingShards.length === 0) {
    console.log('\n✅ Fee vault initialization complete!');
    console.log('   Total shards initialized: 0');
    console.log(`   Ready for script deployment on ${args.network}\n`);
    console.log('Fee Vault Shard Addresses:');
    for (let i = 0; i < shardCount; i++) {
      const vault = await deriveFeeVault(programId, i);
      console.log(`   Shard ${i}: ${vault.address}`);
    }
    return;
  }

  // Estimate required lamports:
  // each missing shard needs rent exemption for 0-byte system account + one tx fee.
  const rentPerVault = await connection.getMinimumBalanceForRentExemption(0);
  const dummyInstruction = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: vmState, isSigner: false, isWritable: false },
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: missingShards[0].vaultPubkey, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from([
      INIT_FEE_VAULT_INSTRUCTION,
      missingShards[0].shardIndex & 0xff,
      missingShards[0].vault.bump & 0xff,
    ]),
  });
  const dummyTx = new Transaction().add(dummyInstruction);
  const { blockhash: feeBlockhash } = await connection.getLatestBlockhash('confirmed');
  dummyTx.recentBlockhash = feeBlockhash;
  dummyTx.feePayer = payer.publicKey;
  const feePerTx = Number((await connection.getFeeForMessage(dummyTx.compileMessage(), 'confirmed')).value || 0);
  const safetyPerTx = 10_000; // 0.00001 SOL
  const requiredLamports = missingShards.length * (rentPerVault + feePerTx + safetyPerTx);
  console.log(
    `✓ Estimated required: ${(requiredLamports / 1e9).toFixed(9)} SOL for ${missingShards.length} shard(s) ` +
    `(each rent ${(rentPerVault / 1e9).toFixed(9)} + fee ${(feePerTx / 1e9).toFixed(9)} + safety ${(safetyPerTx / 1e9).toFixed(9)})\n`
  );
  if (balance < requiredLamports) {
    const shortfall = requiredLamports - balance;
    throw new Error(
      `Insufficient SOL balance. Need ${(requiredLamports / 1e9).toFixed(9)} SOL ` +
      `(short by ${(shortfall / 1e9).toFixed(9)} SOL).`
    );
  }

  // Initialize only missing fee vault shards.
  const txSignatures = [];
  for (const { shardIndex, vault, vaultPubkey } of missingShards) {

    // Create init fee vault instruction
    const data = Buffer.from([INIT_FEE_VAULT_INSTRUCTION, shardIndex & 0xff, vault.bump & 0xff]);

    const instruction = new TransactionInstruction({
      programId,
      keys: [
        { pubkey: vmState, isSigner: false, isWritable: false },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: vaultPubkey, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });

    // Create and send transaction
    const tx = new Transaction().add(instruction);
    const { blockhash } = await connection.getLatestBlockhash('confirmed');
    tx.recentBlockhash = blockhash;
    tx.feePayer = payer.publicKey;
    tx.partialSign(payer);

    try {
      console.log(`⏳ Shard ${shardIndex}: Sending initialization...`);
      const sig = await connection.sendRawTransaction(tx.serialize(), {
        skipPreflight: false,
        preflightCommitment: 'confirmed',
        maxRetries: 5,
      });

      // Wait for confirmation
      await connection.confirmTransaction(sig, 'confirmed');
      txSignatures.push(sig);
      console.log(`✓ Shard ${shardIndex}: Initialized (${vault.address})`);
      console.log(`   Signature: ${sig}\n`);
    } catch (e) {
      console.error(`✗ Shard ${shardIndex}: Failed - ${e.message}`);
      if (e.logs) {
        console.error('   Program logs:', e.logs);
      }
      throw e;
    }
  }

  console.log(`\n✅ Fee vault initialization complete!`);
  console.log(`   Total shards initialized: ${txSignatures.length}`);
  console.log(`   Ready for script deployment on ${args.network}\n`);

  // Print shard addresses for reference
  console.log('Fee Vault Shard Addresses:');
  for (let i = 0; i < shardCount; i++) {
    const vault = await deriveFeeVault(programId, i);
    console.log(`   Shard ${i}: ${vault.address}`);
  }
}

main().catch(err => {
  console.error('\n❌ Error:', err.message);
  process.exit(1);
});
