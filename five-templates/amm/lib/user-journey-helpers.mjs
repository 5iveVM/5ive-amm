import {
  Keypair,
  PublicKey,
  assertOrThrow,
  assertJourneyPreflight,
  buildFiveInstruction,
  createUser,
  emitJourneyStep,
  loadProtocolContext,
  loadSplTokenModule,
  readAccountInfo,
  submitInstruction,
} from '../../user-journeys/lib/framework.mjs';

const SPL_TOKEN_PROGRAM_ID = new PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');

const AMM_ABI = {
  functions: [
    {
      name: 'init_pool',
      index: 0,
      parameters: [
        { name: 'pool', type: 'Pool', is_account: true, attributes: ['mut', 'init', 'signer'] },
        { name: 'creator', type: 'account', is_account: true, attributes: ['mut', 'signer'] },
        { name: 'token_a_mint', type: 'pubkey' },
        { name: 'token_b_mint', type: 'pubkey' },
        { name: 'token_a_vault', type: 'pubkey' },
        { name: 'token_b_vault', type: 'pubkey' },
        { name: 'lp_mint', type: 'pubkey' },
        { name: 'fee_numerator', type: 'u64' },
        { name: 'fee_denominator', type: 'u64' },
        { name: 'protocol_fee_numerator', type: 'u64' },
      ],
    },
    {
      name: 'add_liquidity',
      index: 1,
      parameters: [
        { name: 'pool', type: 'Pool', is_account: true, attributes: ['mut'] },
        { name: 'user_token_a', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_token_b', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'pool_token_a_vault', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'pool_token_b_vault', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'lp_mint', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_lp_account', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'amount_a', type: 'u64' },
        { name: 'amount_b', type: 'u64' },
        { name: 'min_liquidity', type: 'u64' },
      ],
    },
    {
      name: 'swap',
      index: 2,
      parameters: [
        { name: 'pool', type: 'Pool', is_account: true, attributes: ['mut'] },
        { name: 'user_source', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_destination', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'pool_source_vault', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'pool_destination_vault', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'amount_in', type: 'u64' },
        { name: 'min_amount_out', type: 'u64' },
        { name: 'is_a_to_b', type: 'bool' },
      ],
    },
    {
      name: 'remove_liquidity',
      index: 3,
      parameters: [
        { name: 'pool', type: 'Pool', is_account: true, attributes: ['mut'] },
        { name: 'user_lp_account', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_token_a', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_token_b', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'pool_token_a_vault', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'pool_token_b_vault', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'lp_mint', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'lp_amount', type: 'u64' },
        { name: 'min_amount_a', type: 'u64' },
        { name: 'min_amount_b', type: 'u64' },
      ],
    },
    {
      name: 'collect_protocol_fees',
      index: 4,
      parameters: [
        { name: 'pool', type: 'Pool', is_account: true, attributes: ['mut'] },
        { name: 'pool_token_a_vault', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'pool_token_b_vault', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'recipient_a', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'recipient_b', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'authority', type: 'account', is_account: true, attributes: ['signer'] },
      ],
    },
    {
      name: 'set_paused',
      index: 7,
      parameters: [
        { name: 'pool', type: 'Pool', is_account: true, attributes: ['mut'] },
        { name: 'authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'paused', type: 'bool' },
      ],
    },
  ],
};

export { Keypair, PublicKey, assertOrThrow, emitJourneyStep, createUser };

export async function loadAmmContext() {
  return loadProtocolContext({
    scriptEnvNames: ['FIVE_AMM_SCRIPT_ACCOUNT', 'AMM_SCRIPT_ACCOUNT'],
    requiredScriptLabel: 'AMM script account',
    abi: AMM_ABI,
    family: 'amm',
  });
}

export async function assertAmmPreflight(ctx) {
  await assertJourneyPreflight(ctx);
}

export async function createMintAndAccount(ctx, owner, name = 'AmmToken') {
  const spl = await loadSplTokenModule();
  const mint = Keypair.generate();
  const tokenAccount = Keypair.generate();
  await spl.createMint(
    ctx.connection,
    ctx.payer,
    owner.publicKey,
    null,
    6,
    mint,
    undefined,
    SPL_TOKEN_PROGRAM_ID,
  );
  await spl.createAccount(
    ctx.connection,
    ctx.payer,
    mint.publicKey,
    owner.publicKey,
    tokenAccount,
    undefined,
    SPL_TOKEN_PROGRAM_ID,
  );
  return { mint, tokenAccount };
}

export async function createTokenAccount(ctx, owner, mint, step = 'init_token_account') {
  const spl = await loadSplTokenModule();
  const tokenAccount = Keypair.generate();
  await spl.createAccount(
    ctx.connection,
    ctx.payer,
    mint.publicKey || mint,
    owner.publicKey,
    tokenAccount,
    undefined,
    SPL_TOKEN_PROGRAM_ID,
  );
  emitJourneyStep({
    step,
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'native SPL helper',
  });
  return tokenAccount;
}

export async function mintTokens(ctx, mint, destination, authority, amount, step = 'mint_to') {
  const spl = await loadSplTokenModule();
  const signature = await spl.mintTo(
    ctx.connection,
    ctx.payer,
    mint.publicKey || mint,
    destination.publicKey || destination,
    authority,
    amount,
    [],
    undefined,
    SPL_TOKEN_PROGRAM_ID,
  );
  emitJourneyStep({
    step,
    status: 'PASS',
    signature,
    computeUnits: null,
    missingCuReason: 'native SPL helper',
  });
  return { success: true, signature };
}

export async function readSplTokenBalance(ctx, tokenAccount) {
  const spl = await loadSplTokenModule();
  const account = await spl.getAccount(
    ctx.connection,
    tokenAccount.publicKey || tokenAccount,
    'confirmed',
    SPL_TOKEN_PROGRAM_ID,
  );
  return Number(account.amount);
}

function appendReadonlyExtra(ix, pubkey) {
  ix.keys.push({
    pubkey: pubkey.toBase58(),
    isSigner: false,
    isWritable: false,
  });
  return ix;
}

export async function initPool(ctx, authority, pool, setup, fees = { feeNumerator: 3, feeDenominator: 1000, protocolFeeNumerator: 1 }) {
  const ix = await buildFiveInstruction(ctx, 'init_pool', {
    pool: pool.publicKey,
    creator: authority.publicKey,
  }, {
    token_a_mint: setup.tokenAMint.publicKey || setup.tokenAMint,
    token_b_mint: setup.tokenBMint.publicKey || setup.tokenBMint,
    token_a_vault: setup.poolTokenAVault.publicKey || setup.poolTokenAVault,
    token_b_vault: setup.poolTokenBVault.publicKey || setup.poolTokenBVault,
    lp_mint: setup.lpMint.publicKey || setup.lpMint,
    fee_numerator: fees.feeNumerator,
    fee_denominator: fees.feeDenominator,
    protocol_fee_numerator: fees.protocolFeeNumerator,
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority, pool], 'amm_init_pool');
}

export async function addLiquidity(ctx, authority, pool, setup, amountA, amountB, minLiquidity, step = 'amm_add_liquidity') {
  const ix = appendReadonlyExtra(await buildFiveInstruction(ctx, 'add_liquidity', {
    pool: pool.publicKey,
    user_token_a: setup.authorityTokenA.publicKey || setup.authorityTokenA,
    user_token_b: setup.authorityTokenB.publicKey || setup.authorityTokenB,
    pool_token_a_vault: setup.poolTokenAVault.publicKey || setup.poolTokenAVault,
    pool_token_b_vault: setup.poolTokenBVault.publicKey || setup.poolTokenBVault,
    lp_mint: setup.lpMint.publicKey || setup.lpMint,
    user_lp_account: setup.authorityLpAccount.publicKey || setup.authorityLpAccount,
    user_authority: authority.publicKey,
  }, {
    amount_a: amountA,
    amount_b: amountB,
    min_liquidity: minLiquidity,
  }), SPL_TOKEN_PROGRAM_ID);
  return submitInstruction(ctx, ix, [ctx.payer, authority], step);
}

export async function swapTokens(ctx, signer, pool, setup, amountIn, minAmountOut, isAToB, step = 'amm_swap') {
  const ix = appendReadonlyExtra(await buildFiveInstruction(ctx, 'swap', {
    pool: pool.publicKey,
    user_source: (isAToB ? setup.swapSourceA : setup.swapSourceB).publicKey || (isAToB ? setup.swapSourceA : setup.swapSourceB),
    user_destination: (isAToB ? setup.traderTokenB : setup.traderTokenA).publicKey || (isAToB ? setup.traderTokenB : setup.traderTokenA),
    pool_source_vault: (isAToB ? setup.poolTokenAVault : setup.poolTokenBVault).publicKey || (isAToB ? setup.poolTokenAVault : setup.poolTokenBVault),
    pool_destination_vault: (isAToB ? setup.poolTokenBVault : setup.poolTokenAVault).publicKey || (isAToB ? setup.poolTokenBVault : setup.poolTokenAVault),
    user_authority: signer.publicKey,
  }, {
    amount_in: amountIn,
    min_amount_out: minAmountOut,
    is_a_to_b: isAToB,
  }), SPL_TOKEN_PROGRAM_ID);
  return submitInstruction(ctx, ix, [ctx.payer, signer], step);
}

export async function removeLiquidity(ctx, authority, pool, setup, lpAmount, minAmountA, minAmountB, step = 'amm_remove_liquidity') {
  const ix = appendReadonlyExtra(await buildFiveInstruction(ctx, 'remove_liquidity', {
    pool: pool.publicKey,
    user_lp_account: setup.authorityLpAccount.publicKey || setup.authorityLpAccount,
    user_token_a: setup.authorityTokenA.publicKey || setup.authorityTokenA,
    user_token_b: setup.authorityTokenB.publicKey || setup.authorityTokenB,
    pool_token_a_vault: setup.poolTokenAVault.publicKey || setup.poolTokenAVault,
    pool_token_b_vault: setup.poolTokenBVault.publicKey || setup.poolTokenBVault,
    lp_mint: setup.lpMint.publicKey || setup.lpMint,
    user_authority: authority.publicKey,
  }, {
    lp_amount: lpAmount,
    min_amount_a: minAmountA,
    min_amount_b: minAmountB,
  }), SPL_TOKEN_PROGRAM_ID);
  return submitInstruction(ctx, ix, [ctx.payer, authority], step);
}

export async function collectProtocolFees(ctx, authority, pool, setup, step = 'amm_collect_protocol_fees') {
  const ix = appendReadonlyExtra(await buildFiveInstruction(ctx, 'collect_protocol_fees', {
    pool: pool.publicKey,
    pool_token_a_vault: setup.poolTokenAVault.publicKey || setup.poolTokenAVault,
    pool_token_b_vault: setup.poolTokenBVault.publicKey || setup.poolTokenBVault,
    recipient_a: setup.authorityTokenA.publicKey || setup.authorityTokenA,
    recipient_b: setup.authorityTokenB.publicKey || setup.authorityTokenB,
    authority: authority.publicKey,
  }), SPL_TOKEN_PROGRAM_ID);
  return submitInstruction(ctx, ix, [ctx.payer, authority], step);
}

export async function setPaused(ctx, authority, pool, paused, step = 'amm_set_paused') {
  const ix = await buildFiveInstruction(ctx, 'set_paused', {
    pool: pool.publicKey,
    authority: authority.publicKey,
  }, {
    paused,
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority], step);
}

export async function readPoolState(ctx, poolPubkey) {
  const info = await readAccountInfo(ctx, poolPubkey);
  assertOrThrow(info, `Pool account not found: ${poolPubkey.toBase58()}`);
  const data = info.data;
  const reserveA = Number(data.readBigUInt64LE(160));
  const reserveB = Number(data.readBigUInt64LE(168));
  const lpSupply = Number(data.readBigUInt64LE(176));
  const protocolFeesA = Number(data.readBigUInt64LE(208));
  const protocolFeesB = Number(data.readBigUInt64LE(216));
  const authority = new PublicKey(data.subarray(224, 256)).toBase58();
  const isPaused = data[256] === 1;
  return { reserveA, reserveB, lpSupply, protocolFeesA, protocolFeesB, authority, isPaused };
}

export async function prepareAmmFixture(ctx, labelPrefix = 'amm') {
  const authority = await createUser(ctx, `${labelPrefix}_authority`);
  const trader = await createUser(ctx, `${labelPrefix}_trader`);
  const { mint: tokenAMint, tokenAccount: authorityTokenA } = await createMintAndAccount(ctx, authority, `${labelPrefix}TokenA`);
  const { mint: tokenBMint, tokenAccount: authorityTokenB } = await createMintAndAccount(ctx, authority, `${labelPrefix}TokenB`);
  const lpMint = Keypair.generate();
  const spl = await loadSplTokenModule();
  await spl.createMint(
    ctx.connection,
    ctx.payer,
    authority.publicKey,
    null,
    6,
    lpMint,
    undefined,
    SPL_TOKEN_PROGRAM_ID,
  );
  emitJourneyStep({
    step: `${labelPrefix}_init_lp_mint`,
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'native SPL helper',
  });
  const authorityLpAccount = await createTokenAccount(ctx, authority, lpMint, `${labelPrefix}_init_lp_account`);
  const poolTokenAVault = await createTokenAccount(ctx, authority, tokenAMint, `${labelPrefix}_init_pool_token_a_vault`);
  const poolTokenBVault = await createTokenAccount(ctx, authority, tokenBMint, `${labelPrefix}_init_pool_token_b_vault`);
  const traderTokenA = await createTokenAccount(ctx, trader, tokenAMint, `${labelPrefix}_init_trader_token_a`);
  const traderTokenB = await createTokenAccount(ctx, trader, tokenBMint, `${labelPrefix}_init_trader_token_b`);
  const swapSourceA = await createTokenAccount(ctx, authority, tokenAMint, `${labelPrefix}_init_swap_source_a`);
  const swapSourceB = await createTokenAccount(ctx, authority, tokenBMint, `${labelPrefix}_init_swap_source_b`);

  await mintTokens(ctx, tokenAMint, authorityTokenA, authority, 1_000_000, `${labelPrefix}_mint_token_a_to_authority`);
  await mintTokens(ctx, tokenBMint, authorityTokenB, authority, 1_000_000, `${labelPrefix}_mint_token_b_to_authority`);
  await mintTokens(ctx, tokenAMint, swapSourceA, authority, 200_000, `${labelPrefix}_mint_swap_source_a`);
  await mintTokens(ctx, tokenBMint, swapSourceB, authority, 200_000, `${labelPrefix}_mint_swap_source_b`);

  const pool = Keypair.generate();
  const setup = {
    tokenAMint,
    tokenBMint,
    authorityTokenA,
    authorityTokenB,
    poolTokenAVault,
    poolTokenBVault,
    lpMint,
    authorityLpAccount,
    traderTokenA,
    traderTokenB,
    swapSourceA,
    swapSourceB,
  };
  await initPool(ctx, authority, pool, setup);
  await addLiquidity(ctx, authority, pool, setup, 500_000, 500_000, 1_000_000);
  return { authority, trader, pool, setup };
}
