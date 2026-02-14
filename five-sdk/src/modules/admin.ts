import { validator } from "../validation/index.js";
import { ProgramIdResolver } from "../config/ProgramIdResolver.js";
import bs58 from "bs58";

const VM_STATE_SEED = Buffer.from("vm_state", "utf8");
const FEE_VAULT_NAMESPACE_SEED = Buffer.from([
  0xff, 0x66, 0x69, 0x76, 0x65, 0x5f, 0x76, 0x6d, 0x5f, 0x66, 0x65, 0x65,
  0x5f, 0x76, 0x61, 0x75, 0x6c, 0x74, 0x5f, 0x76, 0x31,
]);

async function deriveCanonicalVmState(programId: string): Promise<{ address: string; bump: number }> {
  const { PublicKey } = await import("@solana/web3.js");
  const [pda, bump] = PublicKey.findProgramAddressSync(
    [VM_STATE_SEED],
    new PublicKey(programId),
  );
  return { address: pda.toBase58(), bump };
}

async function deriveFeeVault(
  programId: string,
  shardIndex: number,
): Promise<{ address: string; bump: number }> {
  const { PublicKey } = await import("@solana/web3.js");
  const [pda, bump] = PublicKey.findProgramAddressSync(
    [FEE_VAULT_NAMESPACE_SEED, Buffer.from([shardIndex & 0xff])],
    new PublicKey(programId),
  );
  return { address: pda.toBase58(), bump };
}

function encodeInitializeVmState(bump: number): Uint8Array {
  return Uint8Array.from([0, bump & 0xff]);
}

function encodeSetFees(deployFeeLamports: number, executeFeeLamports: number): Uint8Array {
  const out = new Uint8Array(9);
  out[0] = 6;
  const view = new DataView(out.buffer);
  view.setUint32(1, deployFeeLamports >>> 0, true);
  view.setUint32(5, executeFeeLamports >>> 0, true);
  return out;
}

function encodeInitFeeVault(shardIndex: number, bump: number): Uint8Array {
  return Uint8Array.from([11, shardIndex & 0xff, bump & 0xff]);
}

function encodeWithdrawScriptFees(scriptAccount: string, shardIndex: number, lamports: number): Uint8Array {
  const out = new Uint8Array(42);
  out[0] = 12;
  const scriptBytes = bs58.decode(scriptAccount);
  if (scriptBytes.length !== 32) {
    throw new Error("scriptAccount must decode to 32 bytes");
  }
  out.set(scriptBytes, 1);
  out[33] = shardIndex & 0xff;
  const view = new DataView(out.buffer);
  view.setBigUint64(34, BigInt(lamports), true);
  return out;
}

function requireU32(name: string, value: number): void {
  validator.validateNumber(value, name);
  if (!Number.isInteger(value) || value < 0 || value > 0xffff_ffff) {
    throw new Error(`${name} must be a u32 integer`);
  }
}

function requireU64(name: string, value: number): void {
  validator.validateNumber(value, name);
  if (!Number.isInteger(value) || value < 0 || value > Number.MAX_SAFE_INTEGER) {
    throw new Error(`${name} must be a non-negative safe integer`);
  }
}

function requireShardIndex(shardIndex: number): void {
  validator.validateNumber(shardIndex, "shardIndex");
  if (!Number.isInteger(shardIndex) || shardIndex < 0 || shardIndex > 255) {
    throw new Error("shardIndex must be in range 0..255");
  }
}

export async function generateInitializeVmStateInstruction(
  authority: string,
  options: {
    fiveVMProgramId?: string;
    payer?: string;
    vmStateAccount?: string;
  } = {},
): Promise<{
  programId: string;
  vmStateAccount: string;
  bump: number;
  instruction: { programId: string; accounts: Array<{ pubkey: string; isSigner: boolean; isWritable: boolean }>; data: string };
  requiredSigners: string[];
}> {
  validator.validateBase58Address(authority, "authority");
  const payer = options.payer || authority;
  validator.validateBase58Address(payer, "payer");

  const programId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const canonical = await deriveCanonicalVmState(programId);
  if (options.vmStateAccount && options.vmStateAccount !== canonical.address) {
    throw new Error(
      `vmStateAccount must be canonical PDA ${canonical.address}; got ${options.vmStateAccount}`,
    );
  }

  const accounts = [
    { pubkey: canonical.address, isSigner: false, isWritable: true },
    { pubkey: authority, isSigner: true, isWritable: false },
    { pubkey: payer, isSigner: true, isWritable: true },
    { pubkey: "11111111111111111111111111111111", isSigner: false, isWritable: false },
  ];
  return {
    programId,
    vmStateAccount: canonical.address,
    bump: canonical.bump,
    instruction: {
      programId,
      accounts,
      data: Buffer.from(encodeInitializeVmState(canonical.bump)).toString("base64"),
    },
    requiredSigners: Array.from(new Set([authority, payer])),
  };
}

export async function generateSetFeesInstruction(
  authority: string,
  deployFeeLamports: number,
  executeFeeLamports: number,
  options: {
    fiveVMProgramId?: string;
    vmStateAccount?: string;
  } = {},
): Promise<{
  programId: string;
  vmStateAccount: string;
  instruction: { programId: string; accounts: Array<{ pubkey: string; isSigner: boolean; isWritable: boolean }>; data: string };
  requiredSigners: string[];
}> {
  validator.validateBase58Address(authority, "authority");
  requireU32("deployFeeLamports", deployFeeLamports);
  requireU32("executeFeeLamports", executeFeeLamports);

  const programId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const canonical = await deriveCanonicalVmState(programId);
  if (options.vmStateAccount && options.vmStateAccount !== canonical.address) {
    throw new Error(
      `vmStateAccount must be canonical PDA ${canonical.address}; got ${options.vmStateAccount}`,
    );
  }

  const accounts = [
    { pubkey: canonical.address, isSigner: false, isWritable: true },
    { pubkey: authority, isSigner: true, isWritable: false },
  ];
  return {
    programId,
    vmStateAccount: canonical.address,
    instruction: {
      programId,
      accounts,
      data: Buffer.from(encodeSetFees(deployFeeLamports, executeFeeLamports)).toString("base64"),
    },
    requiredSigners: [authority],
  };
}

export async function generateInitFeeVaultInstruction(
  payer: string,
  shardIndex: number,
  options: {
    fiveVMProgramId?: string;
    vmStateAccount?: string;
    feeVaultAccount?: string;
  } = {},
): Promise<{
  programId: string;
  vmStateAccount: string;
  feeVaultAccount: string;
  bump: number;
  instruction: { programId: string; accounts: Array<{ pubkey: string; isSigner: boolean; isWritable: boolean }>; data: string };
  requiredSigners: string[];
}> {
  validator.validateBase58Address(payer, "payer");
  requireShardIndex(shardIndex);
  const programId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const canonical = await deriveCanonicalVmState(programId);
  const feeVault = await deriveFeeVault(programId, shardIndex);
  if (options.vmStateAccount && options.vmStateAccount !== canonical.address) {
    throw new Error(
      `vmStateAccount must be canonical PDA ${canonical.address}; got ${options.vmStateAccount}`,
    );
  }
  if (options.feeVaultAccount && options.feeVaultAccount !== feeVault.address) {
    throw new Error(
      `feeVaultAccount must be canonical PDA ${feeVault.address}; got ${options.feeVaultAccount}`,
    );
  }

  const accounts = [
    { pubkey: canonical.address, isSigner: false, isWritable: false },
    { pubkey: payer, isSigner: true, isWritable: true },
    { pubkey: feeVault.address, isSigner: false, isWritable: true },
    { pubkey: "11111111111111111111111111111111", isSigner: false, isWritable: false },
  ];
  return {
    programId,
    vmStateAccount: canonical.address,
    feeVaultAccount: feeVault.address,
    bump: feeVault.bump,
    instruction: {
      programId,
      accounts,
      data: Buffer.from(encodeInitFeeVault(shardIndex, feeVault.bump)).toString("base64"),
    },
    requiredSigners: [payer],
  };
}

export async function generateWithdrawScriptFeesInstruction(
  authority: string,
  recipient: string,
  scriptAccount: string,
  shardIndex: number,
  lamports: number,
  options: {
    fiveVMProgramId?: string;
    vmStateAccount?: string;
    feeVaultAccount?: string;
  } = {},
): Promise<{
  programId: string;
  vmStateAccount: string;
  feeVaultAccount: string;
  instruction: { programId: string; accounts: Array<{ pubkey: string; isSigner: boolean; isWritable: boolean }>; data: string };
  requiredSigners: string[];
}> {
  validator.validateBase58Address(authority, "authority");
  validator.validateBase58Address(recipient, "recipient");
  validator.validateBase58Address(scriptAccount, "scriptAccount");
  requireShardIndex(shardIndex);
  requireU64("lamports", lamports);

  const programId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const canonical = await deriveCanonicalVmState(programId);
  const feeVault = await deriveFeeVault(programId, shardIndex);

  if (options.vmStateAccount && options.vmStateAccount !== canonical.address) {
    throw new Error(
      `vmStateAccount must be canonical PDA ${canonical.address}; got ${options.vmStateAccount}`,
    );
  }
  if (options.feeVaultAccount && options.feeVaultAccount !== feeVault.address) {
    throw new Error(
      `feeVaultAccount must be canonical PDA ${feeVault.address}; got ${options.feeVaultAccount}`,
    );
  }

  const accounts = [
    { pubkey: canonical.address, isSigner: false, isWritable: false },
    { pubkey: authority, isSigner: true, isWritable: false },
    { pubkey: feeVault.address, isSigner: false, isWritable: true },
    { pubkey: recipient, isSigner: false, isWritable: true },
  ];
  return {
    programId,
    vmStateAccount: canonical.address,
    feeVaultAccount: feeVault.address,
    instruction: {
      programId,
      accounts,
      data: Buffer.from(encodeWithdrawScriptFees(scriptAccount, shardIndex, lamports)).toString("base64"),
    },
    requiredSigners: [authority],
  };
}

export async function initializeVmStateOnSolana(
  connection: any,
  authorityKeypair: any,
  options: {
    fiveVMProgramId?: string;
    payerKeypair?: any;
    vmStateAccount?: string;
    maxRetries?: number;
    debug?: boolean;
  } = {},
): Promise<{ success: boolean; transactionId?: string; vmStateAccount?: string; error?: string }> {
  const { PublicKey, Transaction, TransactionInstruction } = await import("@solana/web3.js");
  const payerKeypair = options.payerKeypair || authorityKeypair;
  const generated = await generateInitializeVmStateInstruction(
    authorityKeypair.publicKey.toBase58(),
    {
      fiveVMProgramId: options.fiveVMProgramId,
      payer: payerKeypair.publicKey.toBase58(),
      vmStateAccount: options.vmStateAccount,
    },
  );
  const tx = new Transaction().add(
    new TransactionInstruction({
      keys: generated.instruction.accounts.map((a) => ({
        pubkey: new PublicKey(a.pubkey),
        isSigner: a.isSigner,
        isWritable: a.isWritable,
      })),
      programId: new PublicKey(generated.instruction.programId),
      data: Buffer.from(generated.instruction.data, "base64"),
    }),
  );
  const { blockhash } = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = blockhash;
  tx.feePayer = payerKeypair.publicKey;
  tx.partialSign(payerKeypair);
  if (authorityKeypair.publicKey.toBase58() !== payerKeypair.publicKey.toBase58()) {
    tx.partialSign(authorityKeypair);
  }
  const signature = await connection.sendRawTransaction(tx.serialize(), {
    skipPreflight: false,
    preflightCommitment: "confirmed",
    maxRetries: options.maxRetries || 3,
  });
  const confirmation = await connection.confirmTransaction(signature, "confirmed");
  if (confirmation?.value?.err) {
    return { success: false, transactionId: signature, vmStateAccount: generated.vmStateAccount, error: JSON.stringify(confirmation.value.err) };
  }
  return { success: true, transactionId: signature, vmStateAccount: generated.vmStateAccount };
}

export async function setFeesOnSolana(
  connection: any,
  authorityKeypair: any,
  deployFeeLamports: number,
  executeFeeLamports: number,
  options: {
    fiveVMProgramId?: string;
    vmStateAccount?: string;
    maxRetries?: number;
  } = {},
): Promise<{ success: boolean; transactionId?: string; vmStateAccount?: string; error?: string }> {
  const { PublicKey, Transaction, TransactionInstruction } = await import("@solana/web3.js");
  const generated = await generateSetFeesInstruction(
    authorityKeypair.publicKey.toBase58(),
    deployFeeLamports,
    executeFeeLamports,
    {
      fiveVMProgramId: options.fiveVMProgramId,
      vmStateAccount: options.vmStateAccount,
    },
  );
  const tx = new Transaction().add(
    new TransactionInstruction({
      keys: generated.instruction.accounts.map((a) => ({
        pubkey: new PublicKey(a.pubkey),
        isSigner: a.isSigner,
        isWritable: a.isWritable,
      })),
      programId: new PublicKey(generated.instruction.programId),
      data: Buffer.from(generated.instruction.data, "base64"),
    }),
  );
  const { blockhash } = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = blockhash;
  tx.feePayer = authorityKeypair.publicKey;
  tx.partialSign(authorityKeypair);
  const signature = await connection.sendRawTransaction(tx.serialize(), {
    skipPreflight: false,
    preflightCommitment: "confirmed",
    maxRetries: options.maxRetries || 3,
  });
  const confirmation = await connection.confirmTransaction(signature, "confirmed");
  if (confirmation?.value?.err) {
    return { success: false, transactionId: signature, vmStateAccount: generated.vmStateAccount, error: JSON.stringify(confirmation.value.err) };
  }
  return { success: true, transactionId: signature, vmStateAccount: generated.vmStateAccount };
}

export async function initFeeVaultOnSolana(
  connection: any,
  payerKeypair: any,
  shardIndex: number,
  options: {
    fiveVMProgramId?: string;
    vmStateAccount?: string;
    feeVaultAccount?: string;
    maxRetries?: number;
  } = {},
): Promise<{ success: boolean; transactionId?: string; vmStateAccount?: string; feeVaultAccount?: string; error?: string }> {
  const { PublicKey, Transaction, TransactionInstruction } = await import("@solana/web3.js");
  const generated = await generateInitFeeVaultInstruction(
    payerKeypair.publicKey.toBase58(),
    shardIndex,
    {
      fiveVMProgramId: options.fiveVMProgramId,
      vmStateAccount: options.vmStateAccount,
      feeVaultAccount: options.feeVaultAccount,
    },
  );
  const tx = new Transaction().add(
    new TransactionInstruction({
      keys: generated.instruction.accounts.map((a) => ({
        pubkey: new PublicKey(a.pubkey),
        isSigner: a.isSigner,
        isWritable: a.isWritable,
      })),
      programId: new PublicKey(generated.instruction.programId),
      data: Buffer.from(generated.instruction.data, "base64"),
    }),
  );
  const { blockhash } = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = blockhash;
  tx.feePayer = payerKeypair.publicKey;
  tx.partialSign(payerKeypair);
  const signature = await connection.sendRawTransaction(tx.serialize(), {
    skipPreflight: false,
    preflightCommitment: "confirmed",
    maxRetries: options.maxRetries || 3,
  });
  const confirmation = await connection.confirmTransaction(signature, "confirmed");
  if (confirmation?.value?.err) {
    return {
      success: false,
      transactionId: signature,
      vmStateAccount: generated.vmStateAccount,
      feeVaultAccount: generated.feeVaultAccount,
      error: JSON.stringify(confirmation.value.err),
    };
  }
  return {
    success: true,
    transactionId: signature,
    vmStateAccount: generated.vmStateAccount,
    feeVaultAccount: generated.feeVaultAccount,
  };
}

export async function withdrawScriptFeesOnSolana(
  connection: any,
  authorityKeypair: any,
  recipient: string,
  scriptAccount: string,
  shardIndex: number,
  lamports: number,
  options: {
    fiveVMProgramId?: string;
    vmStateAccount?: string;
    feeVaultAccount?: string;
    maxRetries?: number;
  } = {},
): Promise<{ success: boolean; transactionId?: string; vmStateAccount?: string; feeVaultAccount?: string; error?: string }> {
  const { PublicKey, Transaction, TransactionInstruction } = await import("@solana/web3.js");
  const generated = await generateWithdrawScriptFeesInstruction(
    authorityKeypair.publicKey.toBase58(),
    recipient,
    scriptAccount,
    shardIndex,
    lamports,
    {
      fiveVMProgramId: options.fiveVMProgramId,
      vmStateAccount: options.vmStateAccount,
      feeVaultAccount: options.feeVaultAccount,
    },
  );
  const tx = new Transaction().add(
    new TransactionInstruction({
      keys: generated.instruction.accounts.map((a) => ({
        pubkey: new PublicKey(a.pubkey),
        isSigner: a.isSigner,
        isWritable: a.isWritable,
      })),
      programId: new PublicKey(generated.instruction.programId),
      data: Buffer.from(generated.instruction.data, "base64"),
    }),
  );
  const { blockhash } = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = blockhash;
  tx.feePayer = authorityKeypair.publicKey;
  tx.partialSign(authorityKeypair);
  const signature = await connection.sendRawTransaction(tx.serialize(), {
    skipPreflight: false,
    preflightCommitment: "confirmed",
    maxRetries: options.maxRetries || 3,
  });
  const confirmation = await connection.confirmTransaction(signature, "confirmed");
  if (confirmation?.value?.err) {
    return {
      success: false,
      transactionId: signature,
      vmStateAccount: generated.vmStateAccount,
      feeVaultAccount: generated.feeVaultAccount,
      error: JSON.stringify(confirmation.value.err),
    };
  }
  return {
    success: true,
    transactionId: signature,
    vmStateAccount: generated.vmStateAccount,
    feeVaultAccount: generated.feeVaultAccount,
  };
}
