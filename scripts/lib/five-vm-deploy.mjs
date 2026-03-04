import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const FIVE_MAGIC = Buffer.from('5IVE');
const FIVE_HEADER_OPTIMIZED_SIZE = 10;
const FEATURE_IMPORT_VERIFICATION = 1 << 4;
const FEATURE_FUNCTION_NAMES = 1 << 8;
const FEATURE_CONSTANT_POOL = 1 << 10;
const FEATURE_PUBLIC_ENTRY_TABLE = 1 << 12;

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
  const resolvedRpcUrl = requireString(rpcUrl, 'FIVE_RPC_URL');
  const resolvedProgramId = requireString(fiveProgramId, 'FIVE_PROGRAM_ID');
  const resolvedVmStatePda = requireString(vmStatePda, 'VM_STATE_PDA');
  const payerPath = requireString(keypairPath, 'FIVE_KEYPAIR_PATH');
  const artifact = requireString(artifactPath, 'FIVE_ARTIFACT_PATH');
  const resolvedArtifactPath = path.resolve(artifact);

  const { bytecode: compiledBytecode } = loadFiveArtifact(resolvedArtifactPath);
  const split = splitDeployPayload(compiledBytecode);

  if (permissions !== 0) {
    throw new Error('Large-script deployment with non-zero permissions is not supported by this deploy helper yet');
  }

  const cliPath = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    '..',
    '..',
    'five-cli',
    'dist',
    'index.js',
  );

  const args = [
    cliPath,
    'deploy',
    resolvedArtifactPath,
    '--format',
    'json',
    '--network',
    resolvedRpcUrl,
    '--keypair',
    path.resolve(payerPath),
    '--program-id',
    resolvedProgramId,
    '--vm-state-account',
    resolvedVmStatePda,
  ];

  const result = spawnSync(process.execPath, args, {
    cwd: path.dirname(resolvedArtifactPath),
    env: {
      ...process.env,
      FIVE_FEE_SHARD_INDEX: String(feeShardIndex),
      ...(feeVaultAccount ? { FEE_VAULT_ACCOUNT: feeVaultAccount } : {}),
    },
    encoding: 'utf-8',
  });

  if (result.status !== 0) {
    const errorText = [result.stdout, result.stderr].filter(Boolean).join('\n').trim();
    throw new Error(errorText || `${label} deployment failed`);
  }

  let parsed;
  try {
    parsed = JSON.parse((result.stdout || '').trim());
  } catch (error) {
    const details = [result.stdout, result.stderr].filter(Boolean).join('\n').trim();
    throw new Error(`Failed to parse ${label} deploy JSON output: ${details}`);
  }

  const scriptAccount = parsed.scriptAccount || parsed.programId;
  if (!parsed.success || !scriptAccount) {
    throw new Error(parsed.error || `${label} deployment did not return a script account`);
  }

  return {
    signature: parsed.transactionId,
    transactionIds: parsed.transactionIds,
    totalTransactions: parsed.totalTransactions,
    chunksUsed: parsed.chunksUsed,
    deploymentMode: parsed.deploymentMode,
    deploymentCost: parsed.deploymentCost,
    scriptAccount,
    fiveProgramId: resolvedProgramId,
    vmStatePda: resolvedVmStatePda,
    rpcUrl: resolvedRpcUrl,
    bytecodeLength: split.bytecode.length,
    metadataLength: split.metadata.length,
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
    keypairPath: process.env.FIVE_KEYPAIR_PATH || path.join(process.env.HOME, '.config', 'solana', 'id.json'),
    artifactPath: process.env.FIVE_ARTIFACT_PATH || defaultArtifactPath,
    feeShardIndex: Number.parseInt(process.env.FIVE_FEE_SHARD_INDEX || '0', 10) || 0,
    feeVaultAccount: process.env.FEE_VAULT_ACCOUNT || '',
  };
}
