import fs from 'fs';
import path from 'path';
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import { FiveProgram } from '../../../five-sdk/dist/index.js';
import { loadSdkValidatorConfig } from '../../../scripts/lib/sdk-validator-config.mjs';
import { emitUserJourneyStep } from '../../../scripts/lib/user-journey-reporter.mjs';

const FEE_VAULT_SEED_PREFIX = Buffer.from([0xff, ...Buffer.from('five_vm_fee_vault_v1')]);

const TOKEN_ABI = {
  functions: [
    {
      name: 'init_mint',
      index: 0,
      parameters: [
        { name: 'mint_account', type: 'Mint', is_account: true, attributes: ['mut', 'init', 'signer'] },
        { name: 'authority', type: 'account', is_account: true, attributes: ['mut', 'signer'] },
        { name: 'freeze_authority', type: 'pubkey' },
        { name: 'decimals', type: 'u8' },
        { name: 'name', type: 'string' },
        { name: 'symbol', type: 'string' },
        { name: 'uri', type: 'string' },
      ],
    },
    {
      name: 'init_token_account',
      index: 1,
      parameters: [
        { name: 'token_account', type: 'TokenAccount', is_account: true, attributes: ['mut', 'init', 'signer'] },
        { name: 'owner', type: 'account', is_account: true, attributes: ['mut', 'signer'] },
        { name: 'mint', type: 'pubkey' },
      ],
    },
    {
      name: 'mint_to',
      index: 2,
      parameters: [
        { name: 'mint_state', type: 'Mint', is_account: true, attributes: ['mut'] },
        { name: 'destination_account', type: 'TokenAccount', is_account: true, attributes: ['mut'] },
        { name: 'mint_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'amount', type: 'u64' },
      ],
    },
    {
      name: 'transfer',
      index: 3,
      parameters: [
        { name: 'source_account', type: 'TokenAccount', is_account: true, attributes: ['mut'] },
        { name: 'destination_account', type: 'TokenAccount', is_account: true, attributes: ['mut'] },
        { name: 'owner', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'amount', type: 'u64' },
      ],
    },
    {
      name: 'approve',
      index: 5,
      parameters: [
        { name: 'source_account', type: 'TokenAccount', is_account: true, attributes: ['mut'] },
        { name: 'owner', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'delegate', type: 'pubkey' },
        { name: 'amount', type: 'u64' },
      ],
    },
    {
      name: 'transfer_from',
      index: 4,
      parameters: [
        { name: 'source_account', type: 'TokenAccount', is_account: true, attributes: ['mut'] },
        { name: 'destination_account', type: 'TokenAccount', is_account: true, attributes: ['mut'] },
        { name: 'authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'amount', type: 'u64' },
      ],
    },
    {
      name: 'revoke',
      index: 6,
      parameters: [
        { name: 'source_account', type: 'TokenAccount', is_account: true, attributes: ['mut'] },
        { name: 'owner', type: 'account', is_account: true, attributes: ['signer'] },
      ],
    },
  ],
};

export { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL };

function readSecretKey(keypairPath) {
  return Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, 'utf8')));
}

function maybeDeriveVmState(programId) {
  return PublicKey.findProgramAddressSync([Buffer.from('vm_state')], programId)[0];
}

function isRetryableRpcError(errorText) {
  return /fetch failed|429|timed out|timeout|econnreset|connection reset/i.test(errorText);
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
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
      if (ctx.network !== 'devnet' || !isRetryableRpcError(message) || attempt === backoff.length - 1) {
        throw error;
      }
    }
  }
  throw lastError;
}

export async function loadJourneyContext() {
  const cfg = loadSdkValidatorConfig({
    network: process.env.FIVE_NETWORK || 'localnet',
  });
  const connection = new Connection(cfg.rpcUrl, 'confirmed');
  const payer = Keypair.fromSecretKey(readSecretKey(cfg.keypairPath));
  const fiveProgramId = new PublicKey(cfg.programId);
  const vmState = cfg.vmStatePda
    ? new PublicKey(cfg.vmStatePda)
    : maybeDeriveVmState(fiveProgramId);
  const tokenScriptAccountRaw = process.env.FIVE_TOKEN_SCRIPT_ACCOUNT || process.env.TOKEN_SCRIPT_ACCOUNT || process.env.SCRIPT_ACCOUNT || '';
  if (!tokenScriptAccountRaw) {
    throw new Error(
      'Missing FIVE_TOKEN_SCRIPT_ACCOUNT (or TOKEN_SCRIPT_ACCOUNT). ' +
      'User-journey tests require an explicit token script deployment and do not auto-load deployment-config.json.'
    );
  }
  const tokenScriptAccount = new PublicKey(tokenScriptAccountRaw);
  const feeVaultAccount = PublicKey.findProgramAddressSync([FEE_VAULT_SEED_PREFIX, Buffer.from([0])], fiveProgramId)[0];
  const program = FiveProgram.fromABI(tokenScriptAccount.toBase58(), TOKEN_ABI, {
    fiveVMProgramId: fiveProgramId.toBase58(),
    vmStateAccount: vmState.toBase58(),
    feeReceiverAccount: payer.publicKey.toBase58(),
    debug: false,
  });

  return {
    network: cfg.network,
    connection,
    payer,
    fiveProgramId,
    vmState,
    tokenScriptAccount,
    feeVaultAccount,
    program,
    resultsDir: process.env.FIVE_RESULTS_DIR || '',
    scenarioArtifactDir: process.env.FIVE_SCENARIO_ARTIFACT_DIR || '',
  };
}

export async function recordWalletReadable(ctx, wallet, label) {
  const balance = await withRpcRetries(ctx, () => ctx.connection.getBalance(wallet.publicKey, 'confirmed'));
  emitJourneyStep({
    step: `${label}_wallet_loaded`,
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'client-side check',
  });
  return balance;
}

export async function assertJourneyPreflight(ctx) {
  const scriptInfo = await withRpcRetries(ctx, () => ctx.connection.getAccountInfo(ctx.tokenScriptAccount, 'confirmed'));
  if (!scriptInfo) {
    emitJourneyStep({
      step: 'verify_script_account',
      status: 'FAIL',
      computeUnits: null,
      missingCuReason: 'preflight account lookup',
      error: `Script account not found on-chain: ${ctx.tokenScriptAccount.toBase58()}`,
      failureClass: 'account_fixture',
    });
    throw new Error(`Script account not found on-chain: ${ctx.tokenScriptAccount.toBase58()}`);
  }
  if (!scriptInfo.owner.equals(ctx.fiveProgramId)) {
    emitJourneyStep({
      step: 'verify_script_account',
      status: 'FAIL',
      computeUnits: null,
      missingCuReason: 'preflight ownership check',
      error: `Script account owner mismatch: expected ${ctx.fiveProgramId.toBase58()}, got ${scriptInfo.owner.toBase58()}`,
      failureClass: 'program_id',
    });
    throw new Error(`Script account owner mismatch for ${ctx.tokenScriptAccount.toBase58()}`);
  }
  emitJourneyStep({
    step: 'verify_script_account',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'preflight ownership check',
  });

  const vmStateInfo = await withRpcRetries(ctx, () => ctx.connection.getAccountInfo(ctx.vmState, 'confirmed'));
  if (!vmStateInfo) {
    emitJourneyStep({
      step: 'verify_vm_state',
      status: 'FAIL',
      computeUnits: null,
      missingCuReason: 'preflight account lookup',
      error: `VM state not found on-chain: ${ctx.vmState.toBase58()}`,
      failureClass: 'account_fixture',
    });
    throw new Error(`VM state not found on-chain: ${ctx.vmState.toBase58()}`);
  }
  if (!vmStateInfo.owner.equals(ctx.fiveProgramId)) {
    emitJourneyStep({
      step: 'verify_vm_state',
      status: 'FAIL',
      computeUnits: null,
      missingCuReason: 'preflight ownership check',
      error: `VM state owner mismatch: expected ${ctx.fiveProgramId.toBase58()}, got ${vmStateInfo.owner.toBase58()}`,
      failureClass: 'program_id',
    });
    throw new Error(`VM state owner mismatch for ${ctx.vmState.toBase58()}`);
  }
  emitJourneyStep({
    step: 'verify_vm_state',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'preflight ownership check',
  });
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

export async function buildFiveInstruction(ctx, functionName, accounts, args) {
  let builder = ctx.program.function(functionName).accounts(accounts);
  if (args && Object.keys(args).length > 0) {
    builder = builder.args(args);
  }
  return builder.instruction();
}

function normalizeInstruction(ctx, instructionOrData) {
  if (instructionOrData instanceof TransactionInstruction) return instructionOrData;
  const keys = instructionOrData.keys.map((entry) => ({
    pubkey: new PublicKey(entry.pubkey),
    isSigner: entry.isSigner,
    isWritable: entry.isWritable,
  }));

  const hasCanonicalTail = (() => {
    if (keys.length < 3) return false;
    const tailSystem = keys[keys.length - 1];
    const tailVault = keys[keys.length - 2];
    const tailPayer = keys[keys.length - 3];
    return (
      tailSystem.pubkey.equals(SystemProgram.programId) &&
      !tailSystem.isSigner &&
      !tailSystem.isWritable &&
      !tailVault.isSigner &&
      tailVault.isWritable &&
      tailPayer.isSigner &&
      tailPayer.isWritable
    );
  })();

  if (!hasCanonicalTail) {
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
    programId: new PublicKey(instructionOrData.programId),
    keys,
    data: Buffer.from(instructionOrData.data, 'base64'),
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
      const result = {
        success: false,
        signature,
        computeUnits,
        error: txDetails.meta.err,
        logs,
        failureClass,
      };
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
    return {
      success: true,
      signature,
      computeUnits,
      error: null,
      logs,
      failureClass: null,
    };
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
    const result = {
      success: false,
      signature: null,
      computeUnits: null,
      error,
      logs: [],
      failureClass,
    };
    if (allowFailure) return result;
    throw error;
  }
}

export async function initMint(ctx, authority, mintAccount, name = 'JourneyToken') {
  const ix = await buildFiveInstruction(ctx, 'init_mint', {
    mint_account: mintAccount.publicKey,
    authority: authority.publicKey,
  }, {
    freeze_authority: authority.publicKey,
    decimals: 6,
    name,
    symbol: 'JRN',
    uri: 'https://example.com/journey',
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority, mintAccount], 'init_mint');
}

export async function initTokenAccount(ctx, owner, tokenAccount, mint, step = 'init_token_account', options = {}) {
  const ix = await buildFiveInstruction(ctx, 'init_token_account', {
    token_account: tokenAccount.publicKey,
    owner: owner.publicKey,
  }, {
    mint: mint.publicKey,
  });
  const signers = options.signers || [ctx.payer, owner, tokenAccount];
  return submitInstruction(ctx, ix, signers, step, options);
}

export async function mintTo(ctx, mint, destinationAccount, mintAuthority, amount, step = 'mint_to') {
  const ix = await buildFiveInstruction(ctx, 'mint_to', {
    mint_state: mint.publicKey,
    destination_account: destinationAccount.publicKey,
    mint_authority: mintAuthority.publicKey,
  }, {
    amount,
  });
  return submitInstruction(ctx, ix, [ctx.payer, mintAuthority], step);
}

export async function transferTokens(ctx, sourceAccount, destinationAccount, owner, amount, step = 'transfer') {
  const ix = await buildFiveInstruction(ctx, 'transfer', {
    source_account: sourceAccount.publicKey,
    destination_account: destinationAccount.publicKey,
    owner: owner.publicKey,
  }, {
    amount,
  });
  return submitInstruction(ctx, ix, [ctx.payer, owner], step);
}

export async function approveDelegate(ctx, sourceAccount, owner, delegate, amount, step = 'approve') {
  const ix = await buildFiveInstruction(ctx, 'approve', {
    source_account: sourceAccount.publicKey,
    owner: owner.publicKey,
  }, {
    delegate: delegate.publicKey,
    amount,
  });
  return submitInstruction(ctx, ix, [ctx.payer, owner], step);
}

export async function transferFrom(ctx, sourceAccount, destinationAccount, authority, amount, step = 'transfer_from', options = {}) {
  const ix = await buildFiveInstruction(ctx, 'transfer_from', {
    source_account: sourceAccount.publicKey,
    destination_account: destinationAccount.publicKey,
    authority: authority.publicKey,
  }, {
    amount,
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority], step, options);
}

export async function revokeDelegate(ctx, sourceAccount, owner, step = 'revoke') {
  const ix = await buildFiveInstruction(ctx, 'revoke', {
    source_account: sourceAccount.publicKey,
    owner: owner.publicKey,
  });
  return submitInstruction(ctx, ix, [ctx.payer, owner], step);
}

export async function readAccountInfo(ctx, pubkey) {
  return withRpcRetries(ctx, () => ctx.connection.getAccountInfo(pubkey, 'confirmed'));
}

export async function readMintAuthority(ctx, mintPubkey) {
  const info = await readAccountInfo(ctx, mintPubkey);
  if (!info) return null;
  return new PublicKey(info.data.subarray(0, 32)).toBase58();
}

export async function readTokenBalance(ctx, tokenAccountPubkey) {
  const info = await readAccountInfo(ctx, tokenAccountPubkey);
  if (!info) return null;
  return Number(info.data.readBigUInt64LE(64));
}

export async function ensureBalance(ctx, tokenAccountPubkey, expected, step) {
  const actual = await readTokenBalance(ctx, tokenAccountPubkey);
  assertOrThrow(actual === expected, `${step}: expected balance ${expected}, got ${actual}`);
  emitJourneyStep({
    step,
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });
  return actual;
}

export function writeScenarioArtifact(ctx, name, payload) {
  if (!ctx.scenarioArtifactDir) return;
  fs.mkdirSync(ctx.scenarioArtifactDir, { recursive: true });
  fs.writeFileSync(path.join(ctx.scenarioArtifactDir, name), `${JSON.stringify(payload, null, 2)}\n`);
}
