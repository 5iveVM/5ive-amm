#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import web3 from "../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js";
import { loadClusterConfig, deriveVmAddresses, resolveClusterFromEnvOrDefault } from "./lib/vm-cluster-config.mjs";
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

const rpcUrl = readArg("rpc-url", "http://127.0.0.1:8899");
const cluster = readArg("cluster", resolveClusterFromEnvOrDefault());
const profile = loadClusterConfig({ cluster });
const derived = deriveVmAddresses(profile);
const programIdRaw = readArg("program-id", profile.programId);
const vmStateRaw = readArg("vm-state", derived.vmStatePda);
const feeRecipientRaw = readArg("fee-recipient");
const keypairPath = readArg("keypair");

if (!keypairPath || !feeRecipientRaw) {
  console.error(
    "usage: node scripts/vm-state-set-fee-recipient.mjs " +
      "--keypair <path> --fee-recipient <pubkey> " +
      "[--cluster localnet|devnet|mainnet] [--rpc-url ...] [--program-id ...] [--vm-state ...]",
  );
  process.exit(2);
}

const secret = JSON.parse(await readFile(keypairPath, "utf8"));
const signer = Keypair.fromSecretKey(Uint8Array.from(secret));
const connection = new Connection(rpcUrl, "confirmed");
const programId = new PublicKey(programIdRaw);
const vmState = new PublicKey(vmStateRaw);
const feeRecipient = new PublicKey(feeRecipientRaw);

const data = Buffer.alloc(33);
data[0] = 10; // SetFeeRecipient instruction discriminator
feeRecipient.toBuffer().copy(data, 1);

const ix = new TransactionInstruction({
  programId,
  keys: [
    { pubkey: vmState, isSigner: false, isWritable: true },
    { pubkey: signer.publicKey, isSigner: true, isWritable: false },
  ],
  data,
});

const tx = new Transaction().add(ix);
const sig = await sendAndConfirmTransaction(connection, tx, [signer], { commitment: "confirmed" });
console.log("VM_STATE_SET_FEE_RECIPIENT_OK");
console.log(`  signature: ${sig}`);
console.log(`  fee_recipient: ${feeRecipient.toBase58()}`);
