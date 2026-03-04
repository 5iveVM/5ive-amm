import fs from 'node:fs';
import path from 'node:path';
import web3 from '../../five-cli/node_modules/@solana/web3.js/lib/index.cjs.js';
import { confirmSignature } from './solana-confirm.mjs';

const {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  ComputeBudgetProgram,
} = web3;

const FIVE_MAGIC = Buffer.from('5IVE');
const FIVE_HEADER_OPTIMIZED_SIZE = 10;
const FEATURE_IMPORT_VERIFICATION = 1 << 4;
const FEATURE_FUNCTION_NAMES = 1 << 8;
const FEATURE_CONSTANT_POOL = 1 << 10;
const FEATURE_PUBLIC_ENTRY_TABLE = 1 << 12;
const INIT_LARGE_PROGRAM_V2_INSTRUCTION = 13;
const SCRIPT_HEADER_SIZE = 64;
const DEFAULT_COMPUTE_UNIT_LIMIT = 1_400_000;
const DEFAULT_CHUNK_SIZE = 380;
const FEE_VAULT_SEED_PREFIX = Buffer.from([0xff, ...Buffer.from('five_vm_fee_vault_v1')]);

function requireString(value, name) {
  if (!value) {
    throw new Error(`Missing ${name}. Deployment must target an explicit cluster.`);
  }
  return value;
}

function readU16LE(bytes, offset) {
  return bytes[offset] | (bytes[offset + 1] << 8);
}

function readU32LE(bytes, offset) {
  return (
    bytes[offset] |
    (bytes[offset + 1] << 8) |
    (bytes[offset + 2] << 16) |
    (bytes[offset + 3] << 24)
  ) >>> 0;
}

function u32le(value) {
  const out = Buffer.alloc(4);
  out.writeUInt32LE(value >>> 0, 0);
  return out;
}

function parseImportMetadataOffset(bytecode) {
  if (bytecode.length < FIVE_HEADER_OPTIMIZED_SIZE) {
    throw new Error('Invalid Five artifact: bytecode shorter than optimized header');
  }
  if (!Buffer.from(bytecode.subarray(0, 4)).equals(FIVE_MAGIC)) {
    throw new Error('Invalid Five artifact: missing 5IVE magic header');
  }

  const features = readU32LE(bytecode, 4);
  let offset = FIVE_HEADER_OPTIMIZED_SIZE;

  if ((features & FEATURE_FUNCTION_NAMES) !== 0) {
    if (offset + 2 > bytecode.length) {
      throw new Error('Invalid Five artifact: truncated function metadata');
    }
    offset += 2 + readU16LE(bytecode, offset);
  }

  if ((features & FEATURE_PUBLIC_ENTRY_TABLE) !== 0) {
    if (offset + 2 > bytecode.length) {
      throw new Error('Invalid Five artifact: truncated public entry metadata');
    }
    offset += 2 + readU16LE(bytecode, offset);
  }

  if ((features & FEATURE_IMPORT_VERIFICATION) === 0) {
    return bytecode.length;
  }

  if ((features & FEATURE_CONSTANT_POOL) === 0) {
    throw new Error(
      'Import-aware bytecode is missing a constant-pool descriptor; cannot split deploy metadata safely',
    );
  }

  const descSize = 16;
  if (offset + descSize > bytecode.length) {
    throw new Error('Invalid Five artifact: truncated constant-pool descriptor');
  }

  const stringBlobOffset = readU32LE(bytecode, offset + 4);
  const stringBlobLen = readU32LE(bytecode, offset + 8);
  const importMetadataOffset = stringBlobOffset + stringBlobLen;

  if (importMetadataOffset > bytecode.length) {
    throw new Error('Invalid Five artifact: import metadata offset exceeds bytecode length');
  }

  return importMetadataOffset;
}

export function splitDeployPayload(bytecode) {
  const importMetadataOffset = parseImportMetadataOffset(bytecode);
  return {
    bytecode: bytecode.subarray(0, importMetadataOffset),
    metadata: bytecode.subarray(importMetadataOffset),
    importMetadataOffset,
    hadImportMetadata: importMetadataOffset < bytecode.length,
  };
}

export function loadFiveArtifact(artifactPath) {
  const raw = fs.readFileSync(artifactPath);
  try {
    const parsed = JSON.parse(raw.toString('utf-8'));
    if (!parsed?.bytecode || typeof parsed.bytecode !== 'string') {
      throw new Error('Artifact JSON is missing a base64 bytecode field');
    }
    return {
      artifact: parsed,
      bytecode: new Uint8Array(Buffer.from(parsed.bytecode, 'base64')),
    };
  } catch (error) {
    if (error instanceof SyntaxError) {
      return { artifact: null, bytecode: new Uint8Array(raw) };
    }
    throw error;
  }
}

function resolveFeeVault(fiveProgramId, feeShardIndex, feeVaultAccount) {
  if (feeVaultAccount) {
    return new PublicKey(feeVaultAccount);
  }
  return PublicKey.findProgramAddressSync(
    [FEE_VAULT_SEED_PREFIX, Buffer.from([feeShardIndex])],
    fiveProgramId,
  )[0];
}

async function confirmTx(connection, sendResult, description) {
  const confirmation = await confirmSignature(connection, {
    signature: sendResult.signature,
    commitment: 'confirmed',
    blockhash: sendResult.blockhash,
    lastValidBlockHeight: sendResult.lastValidBlockHeight,
  });
  if (!confirmation.success) {
    throw new Error(`${description} failed: ${confirmation.error}`);
  }
}

async function sendAndTrack(connection, tx, signers, options) {
  const latestBlockhash = await connection.getLatestBlockhash('confirmed');
  tx.recentBlockhash = latestBlockhash.blockhash;
  if (!tx.feePayer && signers[0]?.publicKey) {
    tx.feePayer = signers[0].publicKey;
  }
  const signature = await connection.sendTransaction(tx, signers, options);
  return {
    signature,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
  };
}

export async function deployFiveVmScript({
  rpcUrl,
  fiveProgramId,
  vmStatePda,
  keypairPath,
  artifactPath,
  permissions = 0,
  feeShardIndex = 0,
  feeVaultAccount = '',
  label = 'script',
}) {
  const connection = new Connection(requireString(rpcUrl, 'FIVE_RPC_URL'), 'confirmed');
  const programId = new PublicKey(requireString(fiveProgramId, 'FIVE_PROGRAM_ID'));
  const vmState = new PublicKey(requireString(vmStatePda, 'VM_STATE_PDA'));
  const payerPath = requireString(keypairPath, 'FIVE_KEYPAIR_PATH');
  const artifact = requireString(artifactPath, 'FIVE_ARTIFACT_PATH');

  const payer = Keypair.fromSecretKey(
    Uint8Array.from(JSON.parse(fs.readFileSync(path.resolve(payerPath), 'utf-8'))),
  );

  const { bytecode: compiledBytecode } = loadFiveArtifact(path.resolve(artifact));
  const split = splitDeployPayload(compiledBytecode);
  const deployBytecode = Buffer.from(split.bytecode);
  const deployMetadata = Buffer.from(split.metadata);
  const uploadPayload = Buffer.concat([deployMetadata, deployBytecode]);

  if (permissions !== 0) {
    throw new Error('Large-script deployment with non-zero permissions is not supported by this deploy helper yet');
  }

  const balance = await connection.getBalance(payer.publicKey);
  if (balance < 0.05 * LAMPORTS_PER_SOL) {
    throw new Error(`Insufficient balance for ${label} deployment`);
  }

  const vmStateInfo = await connection.getAccountInfo(vmState);
  if (!vmStateInfo || !vmStateInfo.owner.equals(programId)) {
    throw new Error('VM state missing or owned by wrong program');
  }

  const scriptKeypair = Keypair.generate();
  const finalScriptSize = SCRIPT_HEADER_SIZE + uploadPayload.length;
  const rentRequired = await connection.getMinimumBalanceForRentExemption(finalScriptSize);
  const feeVault = resolveFeeVault(programId, feeShardIndex, feeVaultAccount);

  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: scriptKeypair.publicKey,
    lamports: rentRequired,
    space: finalScriptSize,
    programId,
  });

  const firstChunk = uploadPayload.subarray(0, Math.min(DEFAULT_CHUNK_SIZE, uploadPayload.length));
  const initData = Buffer.concat([
    Buffer.from([INIT_LARGE_PROGRAM_V2_INSTRUCTION]),
    u32le(deployBytecode.length),
    u32le(deployMetadata.length),
    firstChunk,
  ]);

  const uploadKeys = [
    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: vmState, isSigner: false, isWritable: true },
    { pubkey: feeVault, isSigner: false, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ];

  const initIx = new TransactionInstruction({
    keys: uploadKeys,
    programId,
    data: initData,
  });

  const tx = new Transaction().add(
    ComputeBudgetProgram.setComputeUnitLimit({ units: DEFAULT_COMPUTE_UNIT_LIMIT }),
    createAccountIx,
    initIx,
  );

  const initResult = await sendAndTrack(connection, tx, [payer, scriptKeypair], {
    skipPreflight: true,
    preflightCommitment: 'confirmed',
    maxRetries: 5,
  });
  await confirmTx(connection, initResult, `${label} init`);

  let lastSignature = initResult.signature;
  for (let offset = firstChunk.length; offset < uploadPayload.length; offset += DEFAULT_CHUNK_SIZE) {
    const chunk = uploadPayload.subarray(offset, Math.min(offset + DEFAULT_CHUNK_SIZE, uploadPayload.length));
    const appendIx = new TransactionInstruction({
      keys: uploadKeys,
      programId,
      data: Buffer.concat([Buffer.from([5]), chunk]),
    });
    const appendTx = new Transaction().add(
      ComputeBudgetProgram.setComputeUnitLimit({ units: DEFAULT_COMPUTE_UNIT_LIMIT }),
      appendIx,
    );
    const appendResult = await sendAndTrack(connection, appendTx, [payer], {
      skipPreflight: true,
      preflightCommitment: 'confirmed',
      maxRetries: 5,
    });
    await confirmTx(connection, appendResult, `${label} append`);
    lastSignature = appendResult.signature;
  }

  const finalizeTx = new Transaction().add(
    ComputeBudgetProgram.setComputeUnitLimit({ units: DEFAULT_COMPUTE_UNIT_LIMIT }),
    new TransactionInstruction({
      keys: [
        { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      ],
      programId,
      data: Buffer.from([7]),
    }),
  );
  const finalizeResult = await sendAndTrack(connection, finalizeTx, [payer], {
    skipPreflight: true,
    preflightCommitment: 'confirmed',
    maxRetries: 5,
  });
  await confirmTx(connection, finalizeResult, `${label} finalize`);
  lastSignature = finalizeResult.signature;

  return {
    signature: lastSignature,
    scriptAccount: scriptKeypair.publicKey.toBase58(),
    fiveProgramId: programId.toBase58(),
    vmStatePda: vmState.toBase58(),
    rpcUrl,
    bytecodeLength: deployBytecode.length,
    metadataLength: deployMetadata.length,
    hadImportMetadata: split.hadImportMetadata,
  };
}

export function loadExplicitDeployEnv(defaultArtifactPath) {
  return {
    rpcUrl: requireString(process.env.FIVE_RPC_URL || process.env.RPC_URL || '', 'FIVE_RPC_URL'),
    fiveProgramId: requireString(
      process.env.FIVE_PROGRAM_ID || process.env.FIVE_VM_PROGRAM_ID || '',
      'FIVE_PROGRAM_ID',
    ),
    vmStatePda: requireString(
      process.env.VM_STATE_PDA || process.env.FIVE_VM_STATE_PDA || '',
      'VM_STATE_PDA',
    ),
    keypairPath: requireString(process.env.FIVE_KEYPAIR_PATH || '', 'FIVE_KEYPAIR_PATH'),
    artifactPath: process.env.FIVE_ARTIFACT_PATH || defaultArtifactPath,
    feeShardIndex: Number.parseInt(process.env.FIVE_FEE_SHARD_INDEX || '0', 10) || 0,
    feeVaultAccount: process.env.FEE_VAULT_ACCOUNT || '',
  };
}
