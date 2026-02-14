import { beforeAll, describe, expect, it } from "@jest/globals";
import { homedir } from "os";
import { readFileSync } from "fs";
import path from "path";
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { FiveSDK } from "../../FiveSDK.js";

const FEE_VAULT_NAMESPACE_SEED = Buffer.from([
  0xff, 0x66, 0x69, 0x76, 0x65, 0x5f, 0x76, 0x6d, 0x5f, 0x66, 0x65, 0x65,
  0x5f, 0x76, 0x61, 0x75, 0x6c, 0x74, 0x5f, 0x76, 0x31,
]);

function minimalHaltBytecode(): Uint8Array {
  // 5IVE magic + features(0) + public_count(1) + total_count(1) + HALT(0x2f)
  return Uint8Array.from([0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 1, 1, 0x2f]);
}

function loadKeypairFromPath(filePath: string): Keypair {
  const raw = readFileSync(filePath, "utf8");
  const arr = JSON.parse(raw) as number[];
  return Keypair.fromSecretKey(Uint8Array.from(arr));
}

function encodeExecuteData(functionIndex: number, shardIndex: number, vaultBump: number): Buffer {
  const out = Buffer.alloc(13);
  out[0] = 9; // Execute discriminator
  out[1] = 0xff;
  out[2] = 0x53;
  out[3] = shardIndex & 0xff;
  out[4] = vaultBump & 0xff;
  out.writeUInt32LE(functionIndex >>> 0, 5);
  out.writeUInt32LE(0, 9); // param count
  return out;
}

const enabled = process.env.RUN_LOCALNET_VALIDATOR_TESTS === "1";
const maybeDescribe = enabled ? describe : describe.skip;

maybeDescribe("Localnet Fee Vault Routing", () => {
  let connection: Connection;
  let payer: Keypair;
  let fiveVmProgramId: string;
  let warmupScriptAccount: string;
  let vmStateAddress: PublicKey;
  let feeVaultAddress: PublicKey;
  let feeVaultBump: number;

  beforeAll(async () => {
    fiveVmProgramId = process.env.FIVE_VM_PROGRAM_ID || "";
    if (!fiveVmProgramId) {
      throw new Error("FIVE_VM_PROGRAM_ID is required when RUN_LOCALNET_VALIDATOR_TESTS=1");
    }

    const rpcUrl = process.env.LOCALNET_RPC_URL || "http://127.0.0.1:8899";
    connection = new Connection(rpcUrl, "confirmed");

    const keypairPath =
      process.env.FIVE_TEST_KEYPAIR_PATH ||
      process.env.SOLANA_KEYPAIR ||
      path.join(homedir(), ".config", "solana", "id.json");
    payer = loadKeypairFromPath(keypairPath);

    const current = await connection.getBalance(payer.publicKey, "confirmed");
    if (current < 1_000_000_000) {
      const sig = await connection.requestAirdrop(payer.publicKey, 2_000_000_000);
      await connection.confirmTransaction(sig, "confirmed");
    }

    const programPk = new PublicKey(fiveVmProgramId);
    [vmStateAddress] = PublicKey.findProgramAddressSync([Buffer.from("vm_state")], programPk);
    [feeVaultAddress, feeVaultBump] = PublicKey.findProgramAddressSync(
      [FEE_VAULT_NAMESPACE_SEED, Buffer.from([0])],
      programPk,
    );

    // Warmup deploy to initialize canonical vm_state and fee vault shards if needed.
    const warmup = await FiveSDK.deployToSolana(
      minimalHaltBytecode(),
      connection,
      payer,
      {
        fiveVMProgramId,
        debug: false,
      },
    );
    if (!warmup.success || !warmup.programId || !warmup.transactionId) {
      throw new Error(`Warmup deploy failed: ${warmup.error || "unknown error"}`);
    }
    const tx = await connection.getTransaction(warmup.transactionId, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    });
    if (!tx || tx.meta?.err) {
      throw new Error(`Warmup deploy tx failed: ${JSON.stringify(tx?.meta?.err || null)}`);
    }
    warmupScriptAccount = warmup.programId;
  }, 120_000);

  it("routes deploy fee to shard-0 fee vault on localnet", async () => {
    const state = await FiveSDK.getVMState(connection, fiveVmProgramId);
    expect(state.deployFeeLamports).toBeGreaterThan(0);

    const before = await connection.getBalance(feeVaultAddress, "confirmed");
    const result = await FiveSDK.deployToSolana(
      minimalHaltBytecode(),
      connection,
      payer,
      { fiveVMProgramId, debug: false },
    );
    expect(result.success).toBe(true);
    expect(result.transactionId).toBeTruthy();

    const tx = await connection.getTransaction(result.transactionId!, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    });
    expect(tx).toBeTruthy();
    expect(tx?.meta?.err).toBeNull();

    const after = await connection.getBalance(feeVaultAddress, "confirmed");
    expect(after - before).toBe(state.deployFeeLamports);
  }, 120_000);

  it("routes execute fee to shard-0 fee vault on localnet", async () => {
    const state = await FiveSDK.getVMState(connection, fiveVmProgramId);
    expect(state.executeFeeLamports).toBeGreaterThan(0);

    const before = await connection.getBalance(feeVaultAddress, "confirmed");
    const exec = await FiveSDK.executeOnSolana(
      warmupScriptAccount,
      connection,
      payer,
      0,
      [],
      [],
      {
        fiveVMProgramId,
        feeShardIndex: 0,
        payerAccount: payer.publicKey.toBase58(),
        debug: false,
      },
    );
    expect(exec.success).toBe(true);
    expect(exec.transactionId).toBeTruthy();

    const tx = await connection.getTransaction(exec.transactionId!, {
      commitment: "confirmed",
      maxSupportedTransactionVersion: 0,
    });
    expect(tx).toBeTruthy();
    expect(tx?.meta?.err).toBeNull();

    const after = await connection.getBalance(feeVaultAddress, "confirmed");
    expect(after - before).toBe(state.executeFeeLamports);
  }, 120_000);

  it("rejects non-canonical vm_state in runtime and does not credit fee vault", async () => {
    const badVmState = payer.publicKey; // deliberately non-canonical
    const before = await connection.getBalance(feeVaultAddress, "confirmed");

    const ix = new TransactionInstruction({
      programId: new PublicKey(fiveVmProgramId),
      keys: [
        { pubkey: new PublicKey(warmupScriptAccount), isSigner: false, isWritable: false },
        { pubkey: badVmState, isSigner: false, isWritable: false },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: feeVaultAddress, isSigner: false, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data: encodeExecuteData(0, 0, feeVaultBump),
    });
    const tx = new Transaction().add(ix);

    await expect(
      sendAndConfirmTransaction(connection, tx, [payer], { commitment: "confirmed" }),
    ).rejects.toThrow();

    const after = await connection.getBalance(feeVaultAddress, "confirmed");
    expect(after).toBe(before);
  }, 120_000);
});
