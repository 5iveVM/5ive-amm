import { readFile } from 'fs/promises';
import { homedir } from 'os';
import { join } from 'path';
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  type ConfirmOptions,
} from '@solana/web3.js';
import {
  ACCOUNT_SIZE,
  TOKEN_PROGRAM_ID,
  createInitializeAccountInstruction,
  createMint,
  getAccount,
  getMinimumBalanceForRentExemptAccount,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from '@solana/spl-token';
import { FiveProgram, FiveSDK } from '@5ive-tech/sdk';

type EncodedInstruction = {
  programId: string;
  keys: Array<{ pubkey: string; isSigner: boolean; isWritable: boolean }>;
  data: string;
};

type StepResult = {
  name: string;
  signature: string | null;
  computeUnits: number | null;
  ok: boolean;
  err: string | null;
};

const NETWORK = process.env.FIVE_NETWORK || 'localnet';
const RPC_BY_NETWORK: Record<string, string> = {
  localnet: 'http://127.0.0.1:8899',
  devnet: 'https://api.devnet.solana.com',
  mainnet: 'https://api.mainnet-beta.solana.com',
};
const PROGRAM_BY_NETWORK: Record<string, string> = {
  localnet: '8h8gqgMhfq5qmPbs9nNHkXNoy2jb1JywxaRC6W68wGVm',
  devnet: '5ive5hbC3aRsvq37MP5m4sHtTSFxT4Cq1smS4ddyWJ6h',
  mainnet: '5ive5hbC3aRsvq37MP5m4sHtTSFxT4Cq1smS4ddyWJ6h',
};
const RPC_URL = process.env.FIVE_RPC_URL || (RPC_BY_NETWORK[NETWORK] || RPC_BY_NETWORK.localnet);
const FIVE_VM_PROGRAM_ID =
  process.env.FIVE_VM_PROGRAM_ID ||
  process.env.FIVE_PROGRAM_ID ||
  (PROGRAM_BY_NETWORK[NETWORK] || PROGRAM_BY_NETWORK.localnet);
const EXISTING_SCRIPT_ACCOUNT = process.env.FIVE_SCRIPT_ACCOUNT || '';
const CONFIRM: ConfirmOptions = {
  commitment: 'confirmed',
  preflightCommitment: 'confirmed',
  skipPreflight: true,
};

function parseConsumedUnits(logs: string[] | null | undefined): number | null {
  if (!logs) return null;
  for (const line of logs) {
    const m = line.match(/consumed (\d+) of/);
    if (m) return Number(m[1]);
  }
  return null;
}

function printableError(err: unknown): string {
  if (err instanceof Error) return err.message || err.stack || err.name;
  try {
    const json = JSON.stringify(err);
    if (json && json !== '{}') return json;
  } catch {
    // ignore
  }
  return String(err);
}

async function loadPayer(): Promise<Keypair> {
  const path = join(homedir(), '.config/solana/id.json');
  const secret = JSON.parse(await readFile(path, 'utf8')) as number[];
  return Keypair.fromSecretKey(new Uint8Array(secret));
}

async function sendIx(
  connection: Connection,
  payer: Keypair,
  encoded: EncodedInstruction,
  signers: Keypair[],
  name: string
): Promise<StepResult> {
  const tx = new Transaction().add(
    new TransactionInstruction({
      programId: new PublicKey(encoded.programId),
      keys: encoded.keys.map((k) => ({
        pubkey: new PublicKey(k.pubkey),
        isSigner: k.isSigner,
        isWritable: k.isWritable,
      })),
      data: Buffer.from(encoded.data, 'base64'),
    })
  );
  tx.feePayer = payer.publicKey;

  const allSignersMap = new Map<string, Keypair>();
  allSignersMap.set(payer.publicKey.toBase58(), payer);
  for (const signer of signers) allSignersMap.set(signer.publicKey.toBase58(), signer);

  const requiredSignerSet = new Set(
    encoded.keys.filter((k) => k.isSigner).map((k) => k.pubkey)
  );
  const neededSigners = Array.from(allSignersMap.values()).filter(
    (kp) => kp.publicKey.equals(payer.publicKey) || requiredSignerSet.has(kp.publicKey.toBase58())
  );

  try {
    const signature = await connection.sendTransaction(tx, neededSigners, CONFIRM);
    const latest = await connection.getLatestBlockhash('confirmed');
    await connection.confirmTransaction({ signature, ...latest }, 'confirmed');
    const txMeta = await connection.getTransaction(signature, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0,
    });
    const metaErr = txMeta?.meta?.err ?? null;
    const cu = txMeta?.meta?.computeUnitsConsumed ?? parseConsumedUnits(txMeta?.meta?.logMessages);
    return {
      name,
      signature,
      computeUnits: cu,
      ok: metaErr == null,
      err: metaErr == null ? null : JSON.stringify(metaErr),
    };
  } catch (err) {
    return {
      name,
      signature: null,
      computeUnits: null,
      ok: false,
      err: printableError(err),
    };
  }
}

async function sendSystemTx(
  connection: Connection,
  payer: Keypair,
  ix: TransactionInstruction,
  signers: Keypair[],
  name: string
): Promise<StepResult> {
  try {
    const tx = new Transaction().add(ix);
    const signature = await connection.sendTransaction(tx, [payer, ...signers], CONFIRM);
    const latest = await connection.getLatestBlockhash('confirmed');
    await connection.confirmTransaction({ signature, ...latest }, 'confirmed');
    const meta = await connection.getTransaction(signature, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0,
    });
    return {
      name,
      signature,
      computeUnits: meta?.meta?.computeUnitsConsumed ?? parseConsumedUnits(meta?.meta?.logMessages),
      ok: meta?.meta?.err == null,
      err: meta?.meta?.err ? JSON.stringify(meta.meta.err) : null,
    };
  } catch (err) {
    return { name, signature: null, computeUnits: null, ok: false, err: printableError(err) };
  }
}

async function createTokenVault(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey
): Promise<Keypair> {
  const vault = Keypair.generate();
  const lamports = await getMinimumBalanceForRentExemptAccount(connection);
  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: vault.publicKey,
      space: ACCOUNT_SIZE,
      lamports,
      programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeAccountInstruction(vault.publicKey, mint, owner, TOKEN_PROGRAM_ID)
  );
  const signature = await connection.sendTransaction(tx, [payer, vault], CONFIRM);
  const latest = await connection.getLatestBlockhash('confirmed');
  await connection.confirmTransaction({ signature, ...latest }, 'confirmed');
  return vault;
}

async function deployScript(connection: Connection, payer: Keypair, loaded: any) {
  let result: any;
  if (loaded.bytecode.length > 1200) {
    result = await FiveSDK.deployLargeProgramToSolana(loaded.bytecode, connection, payer, {
      fiveVMProgramId: FIVE_VM_PROGRAM_ID,
    });
  } else {
    result = await FiveSDK.deployToSolana(loaded.bytecode, connection, payer, {
      fiveVMProgramId: FIVE_VM_PROGRAM_ID,
    });
  }
  if (!result.success && String(result.error || '').toLowerCase().includes('transaction too large')) {
    result = await FiveSDK.deployLargeProgramToSolana(loaded.bytecode, connection, payer, {
      fiveVMProgramId: FIVE_VM_PROGRAM_ID,
    });
  }

  const scriptAccount = result.scriptAccount || result.programId;
  if (!result.success || !scriptAccount) {
    throw new Error(`deploy failed: ${result.error || 'unknown error'}`);
  }

  return {
    scriptAccount,
    signature: result.transactionId || null,
    deploymentCost: result.deploymentCost || null,
  };
}

function pad(name: string): string {
  return name.padEnd(30, ' ');
}

async function assertTokenDelta(
  connection: Connection,
  account: PublicKey,
  before: bigint,
  expectedDelta: bigint,
  label: string
) {
  const after = (await getAccount(connection, account, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  if (after - before !== expectedDelta) {
    throw new Error(`${label} token delta mismatch: expected ${expectedDelta}, got ${after - before}`);
  }
}

async function main() {
  const connection = new Connection(RPC_URL, 'confirmed');
  const payer = await loadPayer();

  const artifactCandidates = [
    join(process.cwd(), '..', 'build', 'main.five'),
    join(process.cwd(), '..', 'build', '5ive-amm.five'),
  ];
  let artifactText = '';
  let artifactPath = '';
  for (const candidate of artifactCandidates) {
    try {
      artifactText = await readFile(candidate, 'utf8');
      artifactPath = candidate;
      break;
    } catch {
      continue;
    }
  }
  if (!artifactText) {
    throw new Error(`missing build artifact: ${artifactCandidates.join(', ')}`);
  }

  const loaded = await FiveSDK.loadFiveFile(artifactText);
  const deploy = EXISTING_SCRIPT_ACCOUNT
    ? { scriptAccount: EXISTING_SCRIPT_ACCOUNT, signature: null, deploymentCost: 0 }
    : await deployScript(connection, payer, loaded);
  const program = FiveProgram.fromABI(deploy.scriptAccount, loaded.abi, {
    fiveVMProgramId: FIVE_VM_PROGRAM_ID,
  });

  const setup: StepResult[] = [];
  const report: StepResult[] = [];

  const decimals = 6;
  const bootstrapA = 5_000_000n;
  const bootstrapB = 5_000_000n;
  const addA = 1_000_000n;
  const addB = 1_000_000n;
  const removeLp = 1_000_000n;
  const swapIn = 1_000_000n;

  const lpUser = Keypair.generate();
  const trader = Keypair.generate();
  setup.push(
    await sendSystemTx(
      connection,
      payer,
      SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: lpUser.publicKey, lamports: 30_000_000 }),
      [],
      'setup:fund_lp_user'
    )
  );
  setup.push(
    await sendSystemTx(
      connection,
      payer,
      SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: trader.publicKey, lamports: 30_000_000 }),
      [],
      'setup:fund_trader'
    )
  );

  const mintA = await createMint(connection, payer, payer.publicKey, null, decimals);
  const mintB = await createMint(connection, payer, payer.publicKey, null, decimals);

  const lpUserAtaA = await getOrCreateAssociatedTokenAccount(connection, payer, mintA, lpUser.publicKey);
  const lpUserAtaB = await getOrCreateAssociatedTokenAccount(connection, payer, mintB, lpUser.publicKey);
  const traderAtaA = await getOrCreateAssociatedTokenAccount(connection, payer, mintA, trader.publicKey);
  const traderAtaB = await getOrCreateAssociatedTokenAccount(connection, payer, mintB, trader.publicKey);

  await mintTo(connection, payer, mintA, lpUserAtaA.address, payer, 20_000_000);
  await mintTo(connection, payer, mintB, lpUserAtaB.address, payer, 20_000_000);
  await mintTo(connection, payer, mintA, traderAtaA.address, payer, 10_000_000);

  const pool = Keypair.generate();

  const vaultA = await createTokenVault(connection, payer, mintA, pool.publicKey);
  const vaultB = await createTokenVault(connection, payer, mintB, pool.publicKey);
  const lpMint = await createMint(connection, payer, pool.publicKey, null, decimals);
  const lpUserLpAta = await getOrCreateAssociatedTokenAccount(connection, payer, lpMint, lpUser.publicKey);

  const initIx = (await program
    .function('init_pool')
    .payer(payer.publicKey.toBase58())
    .accounts({ pool: pool.publicKey.toBase58(), creator: payer.publicKey.toBase58() })
    .args({
      token_a_mint: mintA.toBase58(),
      token_b_mint: mintB.toBase58(),
      token_a_vault: vaultA.publicKey.toBase58(),
      token_b_vault: vaultB.publicKey.toBase58(),
      lp_mint: lpMint.toBase58(),
      fee_numerator: 3,
      fee_denominator: 1000,
      protocol_fee_numerator: 1,
    })
    .instruction()) as EncodedInstruction;
  report.push(await sendIx(connection, payer, initIx, [pool], 'init_pool'));

  const lpUserABefore = (await getAccount(connection, lpUserAtaA.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const lpUserBBefore = (await getAccount(connection, lpUserAtaB.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const vaultABefore = (await getAccount(connection, vaultA.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const vaultBBefore = (await getAccount(connection, vaultB.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const lpTokenBefore = (await getAccount(connection, lpUserLpAta.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;

  const bootstrapIx = (await program
    .function('bootstrap_liquidity')
    .payer(payer.publicKey.toBase58())
    .accounts({
      pool: pool.publicKey.toBase58(),
      user_token_a: lpUserAtaA.address.toBase58(),
      user_token_b: lpUserAtaB.address.toBase58(),
      pool_token_a_vault: vaultA.publicKey.toBase58(),
      pool_token_b_vault: vaultB.publicKey.toBase58(),
      lp_mint: lpMint.toBase58(),
      user_lp_account: lpUserLpAta.address.toBase58(),
      user_authority: lpUser.publicKey.toBase58(),
      token_program: TOKEN_PROGRAM_ID.toBase58(),
    })
    .args({
      amount_a: Number(bootstrapA),
      amount_b: Number(bootstrapB),
      min_liquidity: Number(bootstrapA + bootstrapB),
    })
    .instruction()) as EncodedInstruction;
  report.push(await sendIx(connection, payer, bootstrapIx, [lpUser, pool], 'bootstrap_liquidity'));

  if (report[report.length - 1].ok) {
    await assertTokenDelta(connection, lpUserAtaA.address, lpUserABefore, -bootstrapA, 'bootstrap:lp_user_a');
    await assertTokenDelta(connection, lpUserAtaB.address, lpUserBBefore, -bootstrapB, 'bootstrap:lp_user_b');
    await assertTokenDelta(connection, vaultA.publicKey, vaultABefore, bootstrapA, 'bootstrap:vault_a');
    await assertTokenDelta(connection, vaultB.publicKey, vaultBBefore, bootstrapB, 'bootstrap:vault_b');
    await assertTokenDelta(connection, lpUserLpAta.address, lpTokenBefore, bootstrapA + bootstrapB, 'bootstrap:lp_minted');
  }

  const lpUserABeforeAdd = (await getAccount(connection, lpUserAtaA.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const lpUserBBeforeAdd = (await getAccount(connection, lpUserAtaB.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const vaultABeforeAdd = (await getAccount(connection, vaultA.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const vaultBBeforeAdd = (await getAccount(connection, vaultB.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const lpTokenBeforeAdd = (await getAccount(connection, lpUserLpAta.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const expectedAddLp = (addA * (bootstrapA + bootstrapB)) / bootstrapA;

  const addIx = (await program
    .function('add_liquidity')
    .payer(payer.publicKey.toBase58())
    .accounts({
      pool: pool.publicKey.toBase58(),
      user_token_a: lpUserAtaA.address.toBase58(),
      user_token_b: lpUserAtaB.address.toBase58(),
      pool_token_a_vault: vaultA.publicKey.toBase58(),
      pool_token_b_vault: vaultB.publicKey.toBase58(),
      lp_mint: lpMint.toBase58(),
      user_lp_account: lpUserLpAta.address.toBase58(),
      user_authority: lpUser.publicKey.toBase58(),
      token_program: TOKEN_PROGRAM_ID.toBase58(),
    })
    .args({
      amount_a: Number(addA),
      amount_b: Number(addB),
      min_liquidity: Number(expectedAddLp),
    })
    .instruction()) as EncodedInstruction;
  report.push(await sendIx(connection, payer, addIx, [lpUser, pool], 'add_liquidity'));

  if (report[report.length - 1].ok) {
    await assertTokenDelta(connection, lpUserAtaA.address, lpUserABeforeAdd, -addA, 'add:lp_user_a');
    await assertTokenDelta(connection, lpUserAtaB.address, lpUserBBeforeAdd, -addB, 'add:lp_user_b');
    await assertTokenDelta(connection, vaultA.publicKey, vaultABeforeAdd, addA, 'add:vault_a');
    await assertTokenDelta(connection, vaultB.publicKey, vaultBBeforeAdd, addB, 'add:vault_b');
    await assertTokenDelta(connection, lpUserLpAta.address, lpTokenBeforeAdd, expectedAddLp, 'add:lp_minted');
  }

  const vaultABeforeSwap = (await getAccount(connection, vaultA.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const vaultBBeforeSwap = (await getAccount(connection, vaultB.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const traderABefore = (await getAccount(connection, traderAtaA.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const traderBBefore = (await getAccount(connection, traderAtaB.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;

  const protocolFee = (swapIn * 1n) / 1000n;
  const lpFee = (swapIn * 2n) / 1000n;
  const dxAfterFee = swapIn - protocolFee - lpFee;
  const expectedOut = (vaultBBeforeSwap * dxAfterFee) / (vaultABeforeSwap + dxAfterFee);
  if (expectedOut <= 0n) {
    throw new Error('expectedOut computed to zero');
  }
  const minOut = expectedOut - 1n;

  const swapIx = (await program
    .function('swap')
    .payer(payer.publicKey.toBase58())
    .accounts({
      pool: pool.publicKey.toBase58(),
      user_source: traderAtaA.address.toBase58(),
      user_destination: traderAtaB.address.toBase58(),
      pool_source_vault: vaultA.publicKey.toBase58(),
      pool_destination_vault: vaultB.publicKey.toBase58(),
      user_authority: trader.publicKey.toBase58(),
      token_program: TOKEN_PROGRAM_ID.toBase58(),
    })
    .args({
      amount_in: Number(swapIn),
      min_amount_out: Number(minOut),
      is_a_to_b: true,
    })
    .instruction()) as EncodedInstruction;
  report.push(await sendIx(connection, payer, swapIx, [trader, pool], 'swap:a_to_b'));

  if (report[report.length - 1].ok) {
    await assertTokenDelta(connection, traderAtaA.address, traderABefore, -swapIn, 'swap:trader_a');
    await assertTokenDelta(connection, traderAtaB.address, traderBBefore, expectedOut, 'swap:trader_b');
    await assertTokenDelta(connection, vaultA.publicKey, vaultABeforeSwap, swapIn, 'swap:vault_a');
    await assertTokenDelta(connection, vaultB.publicKey, vaultBBeforeSwap, -expectedOut, 'swap:vault_b');
  }

  const reserveABeforeRemove = bootstrapA + addA + swapIn - protocolFee;
  const reserveBBeforeRemove = bootstrapB + addB - expectedOut;
  const lpSupplyBeforeRemove = bootstrapA + bootstrapB + expectedAddLp;
  const expectedRemoveA = (removeLp * reserveABeforeRemove) / lpSupplyBeforeRemove;
  const expectedRemoveB = (removeLp * reserveBBeforeRemove) / lpSupplyBeforeRemove;

  const lpUserABeforeRemove = (await getAccount(connection, lpUserAtaA.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const lpUserBBeforeRemove = (await getAccount(connection, lpUserAtaB.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const lpUserLpBeforeRemove = (await getAccount(connection, lpUserLpAta.address, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const vaultABeforeRemove = (await getAccount(connection, vaultA.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
  const vaultBBeforeRemove = (await getAccount(connection, vaultB.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;

  const removeIx = (await program
    .function('remove_liquidity')
    .payer(payer.publicKey.toBase58())
    .accounts({
      pool: pool.publicKey.toBase58(),
      user_lp_account: lpUserLpAta.address.toBase58(),
      user_token_a: lpUserAtaA.address.toBase58(),
      user_token_b: lpUserAtaB.address.toBase58(),
      pool_token_a_vault: vaultA.publicKey.toBase58(),
      pool_token_b_vault: vaultB.publicKey.toBase58(),
      lp_mint: lpMint.toBase58(),
      user_authority: lpUser.publicKey.toBase58(),
      token_program: TOKEN_PROGRAM_ID.toBase58(),
    })
    .args({
      lp_amount: Number(removeLp),
      min_amount_a: Number(expectedRemoveA),
      min_amount_b: Number(expectedRemoveB),
    })
    .instruction()) as EncodedInstruction;
  report.push(await sendIx(connection, payer, removeIx, [lpUser, pool], 'remove_liquidity'));

  if (report[report.length - 1].ok) {
    await assertTokenDelta(connection, lpUserLpAta.address, lpUserLpBeforeRemove, -removeLp, 'remove:lp_burned');
    await assertTokenDelta(connection, lpUserAtaA.address, lpUserABeforeRemove, expectedRemoveA, 'remove:lp_user_a');
    await assertTokenDelta(connection, lpUserAtaB.address, lpUserBBeforeRemove, expectedRemoveB, 'remove:lp_user_b');
    await assertTokenDelta(connection, vaultA.publicKey, vaultABeforeRemove, -expectedRemoveA, 'remove:vault_a');
    await assertTokenDelta(connection, vaultB.publicKey, vaultBBeforeRemove, -expectedRemoveB, 'remove:vault_b');
  }

  console.log('--- 5ive-amm token swap report ---');
  console.log('artifact:', artifactPath);
  console.log('rpc:', RPC_URL);
  console.log('five_vm_program_id:', FIVE_VM_PROGRAM_ID);
  console.log('script_account:', deploy.scriptAccount);
  console.log('deploy_signature:', deploy.signature);
  console.log('deployment_cost_lamports:', deploy.deploymentCost);
  console.log('expected_swap_out:', expectedOut.toString());

  for (const item of report) {
    console.log(
      `${pad(item.name)} | ok=${item.ok} | sig=${item.signature ?? 'n/a'} | cu=${item.computeUnits ?? 'n/a'} | err=${item.err ?? 'none'}`
    );
  }

  const failedSetup = setup.filter((r) => !r.ok);
  const failed = report.filter((r) => !r.ok);
  if (failedSetup.length > 0 || failed.length > 0) process.exitCode = 1;
}

main().catch((err) => {
  console.error('run failed:', printableError(err));
  process.exit(1);
});
