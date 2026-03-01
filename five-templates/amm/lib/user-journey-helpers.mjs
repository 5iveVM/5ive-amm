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

export async function createSplMintAndAccount(connection, payer, owner, decimals = 6) {
  const spl = await loadSplTokenModule();
  const mint = await spl.createMint(connection, payer, owner.publicKey, null, decimals);
  const tokenAccount = await spl.createAccount(connection, payer, mint, owner.publicKey);
  return { mint, tokenAccount };
}

export async function mintSplTo(connection, payer, mint, destination, authority, amount) {
  const spl = await loadSplTokenModule();
  return spl.mintTo(connection, payer, mint, destination, authority, amount);
}

export async function readSplTokenBalance(ctx, tokenAccount) {
  const spl = await loadSplTokenModule();
  const acct = await spl.getAccount(ctx.connection, tokenAccount);
  return Number(acct.amount);
}

export async function initPool(ctx, authority, pool, setup, fees = { feeNumerator: 3, feeDenominator: 1000, protocolFeeNumerator: 1 }) {
  const ix = await buildFiveInstruction(ctx, 'init_pool', {
    pool: pool.publicKey,
    creator: authority.publicKey,
  }, {
    token_a_mint: setup.tokenAMint,
    token_b_mint: setup.tokenBMint,
    token_a_vault: setup.poolTokenAVault,
    token_b_vault: setup.poolTokenBVault,
    lp_mint: setup.lpMint,
    fee_numerator: fees.feeNumerator,
    fee_denominator: fees.feeDenominator,
    protocol_fee_numerator: fees.protocolFeeNumerator,
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority, pool], 'amm_init_pool');
}

export async function addLiquidity(ctx, authority, pool, setup, amountA, amountB, minLiquidity, step = 'amm_add_liquidity') {
  const ix = await buildFiveInstruction(ctx, 'add_liquidity', {
    pool: pool.publicKey,
    user_token_a: setup.authorityTokenA,
    user_token_b: setup.authorityTokenB,
    pool_token_a_vault: setup.poolTokenAVault,
    pool_token_b_vault: setup.poolTokenBVault,
    lp_mint: setup.lpMint,
    user_lp_account: setup.authorityLpAccount,
    user_authority: authority.publicKey,
  }, {
    amount_a: amountA,
    amount_b: amountB,
    min_liquidity: minLiquidity,
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority], step);
}

export async function swapTokens(ctx, signer, pool, setup, amountIn, minAmountOut, isAToB, step = 'amm_swap') {
  const ix = await buildFiveInstruction(ctx, 'swap', {
    pool: pool.publicKey,
    user_source: isAToB ? setup.swapSourceA : setup.swapSourceB,
    user_destination: isAToB ? setup.traderTokenB : setup.traderTokenA,
    pool_source_vault: isAToB ? setup.poolTokenAVault : setup.poolTokenBVault,
    pool_destination_vault: isAToB ? setup.poolTokenBVault : setup.poolTokenAVault,
    user_authority: signer.publicKey,
  }, {
    amount_in: amountIn,
    min_amount_out: minAmountOut,
    is_a_to_b: isAToB,
  });
  return submitInstruction(ctx, ix, [ctx.payer, signer], step);
}

export async function removeLiquidity(ctx, authority, pool, setup, lpAmount, minAmountA, minAmountB, step = 'amm_remove_liquidity') {
  const ix = await buildFiveInstruction(ctx, 'remove_liquidity', {
    pool: pool.publicKey,
    user_lp_account: setup.authorityLpAccount,
    user_token_a: setup.authorityTokenA,
    user_token_b: setup.authorityTokenB,
    pool_token_a_vault: setup.poolTokenAVault,
    pool_token_b_vault: setup.poolTokenBVault,
    lp_mint: setup.lpMint,
    user_authority: authority.publicKey,
  }, {
    lp_amount: lpAmount,
    min_amount_a: minAmountA,
    min_amount_b: minAmountB,
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority], step);
}

export async function collectProtocolFees(ctx, authority, pool, setup, step = 'amm_collect_protocol_fees') {
  const ix = await buildFiveInstruction(ctx, 'collect_protocol_fees', {
    pool: pool.publicKey,
    pool_token_a_vault: setup.poolTokenAVault,
    pool_token_b_vault: setup.poolTokenBVault,
    recipient_a: setup.authorityTokenA,
    recipient_b: setup.authorityTokenB,
    authority: authority.publicKey,
  });
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

  const { mint: tokenAMint, tokenAccount: authorityTokenA } = await createSplMintAndAccount(ctx.connection, ctx.payer, authority);
  const { mint: tokenBMint, tokenAccount: authorityTokenB } = await createSplMintAndAccount(ctx.connection, ctx.payer, authority);
  const lpMint = await (await loadSplTokenModule()).createMint(ctx.connection, ctx.payer, authority.publicKey, null, 6);
  const authorityLpAccount = await (await loadSplTokenModule()).createAccount(ctx.connection, ctx.payer, lpMint, authority.publicKey);
  const poolTokenAVault = await (await loadSplTokenModule()).createAccount(ctx.connection, ctx.payer, tokenAMint, authority.publicKey);
  const poolTokenBVault = await (await loadSplTokenModule()).createAccount(ctx.connection, ctx.payer, tokenBMint, authority.publicKey);
  const traderTokenA = await (await loadSplTokenModule()).createAccount(ctx.connection, ctx.payer, tokenAMint, trader.publicKey);
  const traderTokenB = await (await loadSplTokenModule()).createAccount(ctx.connection, ctx.payer, tokenBMint, trader.publicKey);
  const swapSourceA = await (await loadSplTokenModule()).createAccount(ctx.connection, ctx.payer, tokenAMint, authority.publicKey);
  const swapSourceB = await (await loadSplTokenModule()).createAccount(ctx.connection, ctx.payer, tokenBMint, authority.publicKey);

  await mintSplTo(ctx.connection, ctx.payer, tokenAMint, authorityTokenA, authority, 1_000_000n);
  await mintSplTo(ctx.connection, ctx.payer, tokenBMint, authorityTokenB, authority, 1_000_000n);
  await mintSplTo(ctx.connection, ctx.payer, tokenAMint, swapSourceA, authority, 200_000n);
  await mintSplTo(ctx.connection, ctx.payer, tokenBMint, swapSourceB, authority, 200_000n);

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
