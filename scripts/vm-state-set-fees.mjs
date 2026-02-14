#!/usr/bin/env node
import { readFile } from "node:fs/promises";
import web3 from "../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js";
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
const programIdRaw = readArg("program-id", "4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d");
const vmStateRaw = readArg("vm-state", "8ip3qGGETf8774jo6kXbsTTrMm5V9bLuGC4znmyZjT3z");
const keypairPath = readArg("keypair");
const deployFeeRaw = readArg("deploy-fee");
const executeFeeRaw = readArg("execute-fee");

if (!keypairPath || deployFeeRaw === undefined || executeFeeRaw === undefined) {
  console.error(
    "usage: node scripts/vm-state-set-fees.mjs " +
      "--keypair <path> --deploy-fee <lamports> --execute-fee <lamports> " +
      "[--rpc-url ...] [--program-id ...] [--vm-state ...]",
  );
  process.exit(2);
}

const deployFee = Number(deployFeeRaw);
const executeFee = Number(executeFeeRaw);
if (!Number.isInteger(deployFee) || deployFee < 0 || !Number.isInteger(executeFee) || executeFee < 0) {
  console.error("deploy-fee/execute-fee must be non-negative integers");
  process.exit(2);
}

const secret = JSON.parse(await readFile(keypairPath, "utf8"));
const signer = Keypair.fromSecretKey(Uint8Array.from(secret));
const connection = new Connection(rpcUrl, "confirmed");
const programId = new PublicKey(programIdRaw);
const vmState = new PublicKey(vmStateRaw);

const data = Buffer.alloc(9);
data[0] = 6; // SetFees instruction discriminator
data.writeUInt32LE(deployFee, 1);
data.writeUInt32LE(executeFee, 5);

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
console.log("VM_STATE_SET_FEES_OK");
console.log(`  signature: ${sig}`);
console.log(`  deploy_fee_lamports: ${deployFee}`);
console.log(`  execute_fee_lamports: ${executeFee}`);
