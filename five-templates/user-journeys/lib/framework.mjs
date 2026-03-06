import fs from 'fs';
import path from 'path';
import { createRequire } from 'module';
import { fileURLToPath, pathToFileURL } from 'url';
import { FiveProgram, FiveSDK } from '../../../five-sdk/dist/index.js';
import { loadSdkValidatorConfig } from '../../../scripts/lib/sdk-validator-config.mjs';
import { emitUserJourneyStep } from '../../../scripts/lib/user-journey-reporter.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const require = createRequire(import.meta.url);
const {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
} = require('../../../five-sdk/node_modules/@solana/web3.js');
const SPL_TOKEN_ENTRY = pathToFileURL(
  path.join(__dirname, '..', '..', 'cpi-examples', 'node_modules', '@solana', 'spl-token', 'lib', 'esm', 'index.js')
).href;
const FEE_VAULT_SEED_PREFIX = Buffer.from([0xff, ...Buffer.from('five_vm_fee_vault_v1')]);

let cachedSplToken = null;

export {
  Connection,
  FiveSDK,
  FiveProgram,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
};

function readSecretKey(keypairPath) {
  return Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, 'utf8')));
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function isRetryableRpcError(errorText) {
  return /fetch failed|429|timed out|timeout|econnreset|connection reset/i.test(errorText);
}

function resolveExplicitScriptAccount(envNames, label) {
  for (const name of envNames) {
    const raw = process.env[name];
    if (raw && String(raw).trim()) return new PublicKey(String(raw).trim());
  }
  throw new Error(`Missing ${envNames[0]} for ${label}. Hidden deployment fallbacks are disabled.`);
}

export function deriveVmStatePda(programId) {
  return PublicKey.findProgramAddressSync([Buffer.from('vm_state')], programId)[0];
}

export function assertOrThrow(condition, message) {
  if (!condition) throw new Error(message);
}

export function emitJourneyStep(step) {
  emitUserJourneyStep(step);
}

export async function withRpcRetries(ctx, work) {
  const backoff = [0, 2000, 5000, 10000];
  let lastError;
  for (let attempt = 0; attempt < backoff.length; attempt += 1) {
    if (attempt > 0) await sleep(backoff[attempt]);
    try {
      return await work();
    } catch (error) {
      lastError = error;
      const message = error?.message || String(error);
      const retryableNetwork = ctx.network === 'devnet' || ctx.network === 'mainnet';
      if (!retryableNetwork || !isRetryableRpcError(message) || attempt === backoff.length - 1) {
        throw error;
      }
    }
  }
  throw lastError;
}

export async function loadSplTokenModule() {
  if (!cachedSplToken) {
    cachedSplToken = await import(SPL_TOKEN_ENTRY);
  }
  return cachedSplToken;
}

export async function loadProtocolContext({
  scriptEnvNames,
  requiredScriptLabel,
  abi,
  family,
}) {
  const cfg = loadSdkValidatorConfig({
    network: process.env.FIVE_NETWORK || 'localnet',
  });
  const connection = new Connection(cfg.rpcUrl, 'confirmed');
  const payer = Keypair.fromSecretKey(readSecretKey(cfg.keypairPath));
  const fiveProgramId = new PublicKey(cfg.programId);
  const vmState = cfg.vmStatePda
    ? new PublicKey(cfg.vmStatePda)
    : deriveVmStatePda(fiveProgramId);
  const scriptAccount = resolveExplicitScriptAccount(scriptEnvNames, requiredScriptLabel);
  const feeVaultAccount = PublicKey.findProgramAddressSync([FEE_VAULT_SEED_PREFIX, Buffer.from([0])], fiveProgramId)[0];
  const program = FiveProgram.fromABI(scriptAccount.toBase58(), abi, {
    fiveVMProgramId: fiveProgramId.toBase58(),
    vmStateAccount: vmState.toBase58(),
    feeReceiverAccount: payer.publicKey.toBase58(),
    provider: {
      connection,
      publicKey: payer.publicKey,
    },
    debug: false,
  });

  return {
    family,
    network: cfg.network,
    connection,
    payer,
    fiveProgramId,
    vmState,
    scriptAccount,
    feeVaultAccount,
    program,
    resultsDir: process.env.FIVE_RESULTS_DIR || '',
    scenarioArtifactDir: process.env.FIVE_SCENARIO_ARTIFACT_DIR || '',
  };
}

export async function assertJourneyPreflight(ctx, checks = []) {
  const baseChecks = [
    { step: 'verify_script_account', pubkey: ctx.scriptAccount, label: 'Script account' },
    { step: 'verify_vm_state', pubkey: ctx.vmState, label: 'VM state' },
  ];
  for (const check of [...baseChecks, ...checks]) {
    const info = await withRpcRetries(ctx, () => ctx.connection.getAccountInfo(check.pubkey, 'confirmed'));
    if (!info) {
      emitJourneyStep({
        step: check.step,
        status: 'FAIL',
        computeUnits: null,
        missingCuReason: 'preflight account lookup',
        error: `${check.label} not found on-chain: ${check.pubkey.toBase58()}`,
        failureClass: 'account_fixture',
      });
      throw new Error(`${check.label} not found on-chain: ${check.pubkey.toBase58()}`);
    }
    if (check.requireOwner !== false && !info.owner.equals(ctx.fiveProgramId)) {
      emitJourneyStep({
        step: check.step,
        status: 'FAIL',
        computeUnits: null,
        missingCuReason: 'preflight ownership check',
        error: `${check.label} owner mismatch: expected ${ctx.fiveProgramId.toBase58()}, got ${info.owner.toBase58()}`,
        failureClass: 'program_id',
      });
      throw new Error(`${check.label} owner mismatch for ${check.pubkey.toBase58()}`);
    }
    emitJourneyStep({
      step: check.step,
      status: 'PASS',
      computeUnits: null,
      missingCuReason: 'preflight ownership check',
    });
  }
}

export async function createUser(ctx, label, lamports = Math.floor(0.05 * LAMPORTS_PER_SOL)) {
  const wallet = Keypair.generate();
  const transferIx = SystemProgram.transfer({
    fromPubkey: ctx.payer.publicKey,
    toPubkey: wallet.publicKey,
    lamports,
  });
  const result = await submitInstruction(ctx, transferIx, [ctx.payer], `${label}_fund_wallet`);
  assertOrThrow(result.success, `${label} funding failed`);
  return wallet;
}

export async function recordWalletReadable(ctx, wallet, label) {
  await withRpcRetries(ctx, () => ctx.connection.getBalance(wallet.publicKey, 'confirmed'));
  emitJourneyStep({
    step: `${label}_wallet_loaded`,
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'client-side check',
  });
}

export async function buildProgramInstruction(program, functionName, accounts, args, buildOptions = {}) {
  let builder = program.function(functionName).accounts(accounts);
  if (args && Object.keys(args).length > 0) {
    builder = builder.args(args);
  }
  if (buildOptions.payerAccount) {
    builder = builder.payer(buildOptions.payerAccount);
  }
  return builder.instruction();
}

export async function buildFiveInstruction(ctx, functionName, accounts, args, buildOptions = {}) {
  return buildProgramInstruction(ctx.program, functionName, accounts, args, buildOptions);
}

function normalizeInstruction(ctx, instructionOrData) {
  const rawKeys = instructionOrData instanceof TransactionInstruction
    ? instructionOrData.keys
    : instructionOrData.keys;
  const keys = rawKeys.map((entry) => ({
    pubkey: new PublicKey(entry.pubkey),
    isSigner: entry.isSigner,
    isWritable: entry.isWritable,
  }));

  const isCanonicalTailAt = (startIndex) => {
    if (startIndex < 0 || startIndex + 2 >= keys.length) return false;
    const tailPayer = keys[startIndex];
    const tailVault = keys[startIndex + 1];
    const tailSystem = keys[startIndex + 2];
    return (
      tailSystem.pubkey.equals(SystemProgram.programId) &&
      !tailSystem.isSigner &&
      !tailSystem.isWritable &&
      !tailVault.isSigner &&
      tailVault.isWritable &&
      tailPayer.isSigner &&
      tailPayer.isWritable
    );
  };

  const hasCanonicalTail = () => isCanonicalTailAt(keys.length - 3);
  const stripLegacyBuilderTailPair = () => {
    for (let startIndex = keys.length - 2; startIndex >= 0; startIndex -= 1) {
      const tailVault = keys[startIndex];
      const tailSystem = keys[startIndex + 1];
      if (
        tailVault?.pubkey?.equals?.(ctx.feeVaultAccount) &&
        !tailVault.isSigner &&
        tailVault.isWritable &&
        tailSystem?.pubkey?.equals?.(SystemProgram.programId) &&
        !tailSystem.isSigner &&
        !tailSystem.isWritable
      ) {
        keys.splice(startIndex, 2);
        return true;
      }
    }
    return false;
  };

  if (!hasCanonicalTail()) {
    for (let startIndex = keys.length - 4; startIndex >= 0; startIndex -= 1) {
      if (!isCanonicalTailAt(startIndex)) continue;
      const existingTail = keys.splice(startIndex, 3);
      keys.push(...existingTail);
      break;
    }
  }

  if (!hasCanonicalTail()) {
    stripLegacyBuilderTailPair();
  }

  if (!hasCanonicalTail()) {
    keys.push({
      pubkey: ctx.payer.publicKey,
      isSigner: true,
      isWritable: true,
    });
    keys.push({
      pubkey: ctx.feeVaultAccount,
      isSigner: false,
      isWritable: true,
    });
    keys.push({
      pubkey: SystemProgram.programId,
      isSigner: false,
      isWritable: false,
    });
  }

  return new TransactionInstruction({
    programId: instructionOrData instanceof TransactionInstruction
      ? instructionOrData.programId
      : new PublicKey(instructionOrData.programId),
    keys,
    data: instructionOrData instanceof TransactionInstruction
      ? instructionOrData.data
      : Buffer.from(instructionOrData.data, 'base64'),
  });
}

export function classifyFailure(message, logs = []) {
  const haystack = `${message}\n${logs.join('\n')}`.toLowerCase();
  if (/insufficient funds|insufficient lamports|debit an account/.test(haystack)) return 'funding';
  if (/signature verification failed|missing signature|must sign|not signer/.test(haystack)) return 'authority';
  if (/already initialized|already in use/.test(haystack)) return 'already_initialized';
  if (/missing required account|not provided/.test(haystack)) return 'missing_account';
  if (/duplicate/.test(haystack)) return 'duplicate_submit';
  if (/fetch failed|429|timed out|timeout|econnreset/.test(haystack)) return 'rpc';
  if (/program not found|invalid program argument/.test(haystack)) return 'program_id';
  return 'unknown';
}

export async function submitInstruction(ctx, instructionOrData, signers, step, options = {}) {
  const ix = normalizeInstruction(ctx, instructionOrData);
  if (
    step === 'lending_init_oracle' ||
    step === 'lending_set_oracle' ||
    step === 'lending_deposit_reserve_liquidity' ||
    step === 'lending_refresh_obligation_with_oracle'
  ) {
    console.log(
      `USER_JOURNEY_NORMALIZED_KEYS:${JSON.stringify(ix.keys.map((key) => ({
        pubkey: key.pubkey.toBase58(),
        isSigner: key.isSigner,
        isWritable: key.isWritable,
      })))}`
    );
    console.log(`USER_JOURNEY_NORMALIZED_DATA_LEN:${ix.data.length}`);
    console.log(`USER_JOURNEY_NORMALIZED_DATA_HEX:${Buffer.from(ix.data).toString('hex')}`);
  }
  const tx = new Transaction().add(ix);
  const allowFailure = options.allowFailure === true;
  const expectedFailureClass = options.expectedFailureClass || null;

  try {
    const signature = await withRpcRetries(ctx, () => sendAndConfirmTransaction(ctx.connection, tx, signers, {
      skipPreflight: false,
      commitment: 'confirmed',
    }));

    await sleep(500);
    const txDetails = await withRpcRetries(ctx, () => ctx.connection.getTransaction(signature, {
      maxSupportedTransactionVersion: 0,
      commitment: 'confirmed',
    }));
    const logs = txDetails?.meta?.logMessages || [];
    const cuLog = logs.find((line) => line.includes('consumed'));
    const cuMatch = cuLog ? cuLog.match(/consumed (\d+) of/) : null;
    const computeUnits = cuMatch ? Number(cuMatch[1]) : null;

    if (txDetails?.meta?.err) {
      const failureClass = expectedFailureClass || classifyFailure(JSON.stringify(txDetails.meta.err), logs);
      emitJourneyStep({
        step,
        status: 'FAIL',
        signature,
        computeUnits,
        error: JSON.stringify(txDetails.meta.err),
        failureClass,
        missingCuReason: computeUnits === null ? 'transaction meta.err present' : null,
      });
      const result = { success: false, signature, computeUnits, error: txDetails.meta.err, logs, failureClass };
      if (allowFailure) return result;
      const raised = new Error(`${step} failed: ${JSON.stringify(txDetails.meta.err)}`);
      raised.stepAlreadyEmitted = true;
      throw raised;
    }

    emitJourneyStep({
      step,
      status: 'PASS',
      signature,
      computeUnits,
      missingCuReason: computeUnits === null ? 'compute units unavailable' : null,
    });
    return { success: true, signature, computeUnits, error: null, logs, failureClass: null };
  } catch (error) {
    if (error?.stepAlreadyEmitted) throw error;
    const message = error?.message || String(error);
    const failureClass = expectedFailureClass || classifyFailure(message);
    emitJourneyStep({
      step,
      status: 'FAIL',
      signature: null,
      computeUnits: null,
      missingCuReason: 'transaction submission failed',
      error: message,
      failureClass,
    });
    const result = { success: false, signature: null, computeUnits: null, error, logs: [], failureClass };
    if (allowFailure) return result;
    throw error;
  }
}

export async function readAccountInfo(ctx, pubkey) {
  return withRpcRetries(ctx, () => ctx.connection.getAccountInfo(pubkey, 'confirmed'));
}

export function writeScenarioArtifact(ctx, name, payload) {
  if (!ctx.scenarioArtifactDir) return;
  fs.mkdirSync(ctx.scenarioArtifactDir, { recursive: true });
  fs.writeFileSync(path.join(ctx.scenarioArtifactDir, name), `${JSON.stringify(payload, null, 2)}\n`);
}
