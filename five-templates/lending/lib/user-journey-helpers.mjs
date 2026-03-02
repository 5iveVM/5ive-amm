import {
  FiveProgram,
  Keypair,
  PublicKey,
  assertOrThrow,
  assertJourneyPreflight,
  buildFiveInstruction,
  buildProgramInstruction,
  createUser,
  emitJourneyStep,
  loadProtocolContext,
  loadSplTokenModule,
  readAccountInfo,
  submitInstruction,
  withRpcRetries,
  writeScenarioArtifact,
} from '../../user-journeys/lib/framework.mjs';

const LENDING_ABI = {
  functions: [
    {
      name: 'init_market',
      index: 0,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true, attributes: ['mut', 'init', 'signer'] },
        { name: 'quote_currency', type: 'account', is_account: true },
        { name: 'admin', type: 'account', is_account: true, attributes: ['signer'] },
      ],
    },
    {
      name: 'set_market_pause',
      index: 1,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true, attributes: ['mut'] },
        { name: 'admin', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'paused', type: 'bool' },
      ],
    },
    {
      name: 'transfer_market_admin',
      index: 2,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true, attributes: ['mut'] },
        { name: 'admin', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'new_admin', type: 'pubkey' },
      ],
    },
    {
      name: 'init_reserve',
      index: 3,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'reserve', type: 'Reserve', is_account: true, attributes: ['mut', 'init', 'signer'] },
        { name: 'liquidity_mint', type: 'account', is_account: true },
        { name: 'liquidity_supply', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'collateral_mint', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'admin', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'config_optimal_utilization', type: 'u8' },
        { name: 'config_loan_to_value', type: 'u8' },
        { name: 'config_reserve_factor', type: 'u8' },
        { name: 'config_supply_cap', type: 'u64' },
      ],
    },
    {
      name: 'set_reserve_config',
      index: 4,
      parameters: [
        { name: 'reserve', type: 'Reserve', is_account: true, attributes: ['mut'] },
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'admin', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'new_reserve_factor', type: 'u8' },
        { name: 'new_supply_cap', type: 'u64' },
        { name: 'new_loan_to_value', type: 'u8' },
      ],
    },
    {
      name: 'init_obligation',
      index: 5,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'obligation', type: 'Obligation', is_account: true, attributes: ['mut', 'init', 'signer'] },
        { name: 'borrower', type: 'account', is_account: true, attributes: ['signer'] },
      ],
    },
    {
      name: 'refresh_reserve',
      index: 6,
      parameters: [
        { name: 'reserve', type: 'Reserve', is_account: true, attributes: ['mut'] },
      ],
    },
    {
      name: 'refresh_obligation',
      index: 7,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'obligation', type: 'Obligation', is_account: true, attributes: ['mut'] },
        { name: 'reserve', type: 'Reserve', is_account: true },
        { name: 'liquidity_mint', type: 'account', is_account: true },
        { name: 'oracle', type: 'pubkey' },
      ],
    },
    {
      name: 'refresh_obligation_with_oracle',
      index: 8,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'obligation', type: 'Obligation', is_account: true, attributes: ['mut'] },
        { name: 'reserve', type: 'Reserve', is_account: true },
        { name: 'oracle_state', type: 'PriceOracle', is_account: true },
      ],
    },
    {
      name: 'deposit_reserve_liquidity',
      index: 9,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'reserve', type: 'Reserve', is_account: true, attributes: ['mut'] },
        { name: 'user_liquidity', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_collateral', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'liquidity_supply', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'collateral_mint', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'market_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'user_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'amount', type: 'u64' },
      ],
    },
    {
      name: 'withdraw_reserve_liquidity',
      index: 10,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'reserve', type: 'Reserve', is_account: true, attributes: ['mut'] },
        { name: 'obligation', type: 'Obligation', is_account: true },
        { name: 'user_liquidity', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_collateral', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'liquidity_supply', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'collateral_mint', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'market_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'user_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'collateral_amount', type: 'u64' },
      ],
    },
    {
      name: 'borrow_obligation_liquidity',
      index: 11,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'reserve', type: 'Reserve', is_account: true, attributes: ['mut'] },
        { name: 'obligation', type: 'Obligation', is_account: true, attributes: ['mut'] },
        { name: 'user_liquidity', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'liquidity_supply', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'market_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'user_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'amount', type: 'u64' },
      ],
    },
    {
      name: 'repay_obligation_liquidity',
      index: 12,
      parameters: [
        { name: 'market', type: 'LendingMarket', is_account: true },
        { name: 'reserve', type: 'Reserve', is_account: true, attributes: ['mut'] },
        { name: 'obligation', type: 'Obligation', is_account: true, attributes: ['mut'] },
        { name: 'user_liquidity', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'liquidity_supply', type: 'account', is_account: true, attributes: ['mut'] },
        { name: 'user_authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'amount', type: 'u64' },
      ],
    },
  ],
};

const ORACLE_HELPER_ABI = {
  functions: [
    {
      name: 'init_oracle',
      index: 0,
      parameters: [
        { name: 'oracle', type: 'PriceOracle', is_account: true, attributes: ['mut', 'init', 'signer'] },
        { name: 'authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'price', type: 'u64' },
        { name: 'decimals', type: 'u8' },
      ],
    },
    {
      name: 'set_oracle',
      index: 1,
      parameters: [
        { name: 'oracle', type: 'PriceOracle', is_account: true, attributes: ['mut'] },
        { name: 'authority', type: 'account', is_account: true, attributes: ['signer'] },
        { name: 'price', type: 'u64' },
        { name: 'decimals', type: 'u8' },
        { name: 'last_update', type: 'u64' },
      ],
    },
  ],
};

function requireExplicitPubkey(envName, label) {
  const raw = process.env[envName] || '';
  if (!raw.trim()) {
    throw new Error(`Missing ${envName} for ${label}. Hidden deployment fallbacks are disabled.`);
  }
  return new PublicKey(raw.trim());
}

function appendReadonlyExtra(ix, pubkey) {
  ix.keys.push({
    pubkey: pubkey.toBase58(),
    isSigner: false,
    isWritable: false,
  });
  return ix;
}

export async function createSplMintAndAccount(connection, payer, owner, decimals = 6) {
  const spl = await loadSplTokenModule();
  const mint = await spl.createMint(connection, payer, owner.publicKey, null, decimals);
  const tokenAccount = await spl.createAccount(connection, payer, mint, owner.publicKey);
  return { mint, tokenAccount };
}

export async function createSplTokenAccount(connection, payer, mint, owner) {
  const spl = await loadSplTokenModule();
  return spl.createAccount(connection, payer, mint, owner.publicKey);
}

export async function mintSplTo(connection, payer, mint, destination, authority, amount) {
  const spl = await loadSplTokenModule();
  return spl.mintTo(connection, payer, mint, destination, authority, amount);
}

async function readSplTokenBalance(ctx, tokenAccount) {
  const spl = await loadSplTokenModule();
  const acct = await spl.getAccount(ctx.connection, tokenAccount);
  return Number(acct.amount);
}

export { Keypair, PublicKey, assertOrThrow, emitJourneyStep, createUser, readSplTokenBalance, writeScenarioArtifact };

export async function loadLendingContext() {
  const ctx = await loadProtocolContext({
    scriptEnvNames: ['FIVE_LENDING_SCRIPT_ACCOUNT', 'LENDING_SCRIPT_ACCOUNT'],
    requiredScriptLabel: 'lending script account',
    abi: LENDING_ABI,
    family: 'lending',
  });

  const oracleScriptAccount = requireExplicitPubkey(
    'FIVE_LENDING_ORACLE_SCRIPT_ACCOUNT',
    'lending oracle helper script account'
  );

  ctx.oracleScriptAccount = oracleScriptAccount;
  ctx.oracleProgram = FiveProgram.fromABI(oracleScriptAccount.toBase58(), ORACLE_HELPER_ABI, {
    fiveVMProgramId: ctx.fiveProgramId.toBase58(),
    vmStateAccount: ctx.vmState.toBase58(),
    feeReceiverAccount: ctx.payer.publicKey.toBase58(),
    debug: false,
  });
  return ctx;
}

export async function assertLendingPreflight(ctx) {
  await assertJourneyPreflight(ctx, [
    {
      step: 'verify_lending_oracle_script_account',
      pubkey: ctx.oracleScriptAccount,
      label: 'Lending oracle helper script account',
    },
  ]);
}

export async function initOracle(ctx, authority, oracle, price, decimals = 6, step = 'lending_init_oracle', options = {}) {
  const ix = await buildProgramInstruction(ctx.oracleProgram, 'init_oracle', {
    oracle: oracle.publicKey,
    authority: authority.publicKey,
  }, {
    price,
    decimals,
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority, oracle], step, options);
}

export async function setOracle(ctx, authority, oraclePubkey, price, lastUpdate, decimals = 6, step = 'lending_set_oracle', options = {}) {
  const ix = await buildProgramInstruction(ctx.oracleProgram, 'set_oracle', {
    oracle: oraclePubkey,
    authority: authority.publicKey,
  }, {
    price,
    decimals,
    last_update: lastUpdate,
  });
  return submitInstruction(ctx, ix, [ctx.payer, authority], step, options);
}

export async function initMarket(ctx, admin, market, quoteCurrency, step = 'lending_init_market') {
  const ix = await buildFiveInstruction(ctx, 'init_market', {
    market: market.publicKey,
    quote_currency: quoteCurrency,
    admin: admin.publicKey,
  });
  return submitInstruction(ctx, ix, [ctx.payer, admin, market], step);
}

export async function setMarketPause(ctx, admin, marketPubkey, paused, step = 'lending_set_market_pause', options = {}) {
  const ix = await buildFiveInstruction(ctx, 'set_market_pause', {
    market: marketPubkey,
    admin: admin.publicKey,
  }, {
    paused,
  });
  return submitInstruction(ctx, ix, [ctx.payer, admin], step, options);
}

export async function initReserve(ctx, admin, marketPubkey, reserve, setup, config, step = 'lending_init_reserve') {
  const ix = await buildFiveInstruction(ctx, 'init_reserve', {
    market: marketPubkey,
    reserve: reserve.publicKey,
    liquidity_mint: setup.liquidityMint,
    liquidity_supply: setup.liquiditySupply,
    collateral_mint: setup.collateralMint,
    admin: admin.publicKey,
  }, {
    config_optimal_utilization: config.optimalUtilization,
    config_loan_to_value: config.loanToValue,
    config_reserve_factor: config.reserveFactor,
    config_supply_cap: config.supplyCap,
  });
  return submitInstruction(ctx, ix, [ctx.payer, admin, reserve], step);
}

export async function initObligation(ctx, borrower, marketPubkey, obligation, step = 'lending_init_obligation') {
  const ix = await buildFiveInstruction(ctx, 'init_obligation', {
    market: marketPubkey,
    obligation: obligation.publicKey,
    borrower: borrower.publicKey,
  });
  return submitInstruction(ctx, ix, [ctx.payer, borrower, obligation], step);
}

export async function refreshObligationWithOracle(ctx, marketPubkey, obligationPubkey, reservePubkey, oraclePubkey, step = 'lending_refresh_obligation_with_oracle', options = {}) {
  const ix = await buildFiveInstruction(ctx, 'refresh_obligation_with_oracle', {
    market: marketPubkey,
    obligation: obligationPubkey,
    reserve: reservePubkey,
    oracle_state: oraclePubkey,
  });
  return submitInstruction(ctx, ix, [ctx.payer], step, options);
}

export async function depositReserveLiquidity(ctx, admin, borrower, marketPubkey, reservePubkey, setup, amount, step = 'lending_deposit_reserve_liquidity', options = {}) {
  const spl = await loadSplTokenModule();
  const ix = appendReadonlyExtra(await buildFiveInstruction(ctx, 'deposit_reserve_liquidity', {
    market: marketPubkey,
    reserve: reservePubkey,
    user_liquidity: setup.borrowerLiquidity,
    user_collateral: setup.borrowerCollateral,
    liquidity_supply: setup.liquiditySupply,
    collateral_mint: setup.collateralMint,
    market_authority: admin.publicKey,
    user_authority: borrower.publicKey,
  }, {
    amount,
  }), spl.TOKEN_PROGRAM_ID);
  return submitInstruction(ctx, ix, [ctx.payer, admin, borrower], step, options);
}

export async function borrowObligationLiquidity(ctx, admin, borrower, marketPubkey, reservePubkey, obligationPubkey, setup, amount, step = 'lending_borrow_obligation_liquidity', options = {}) {
  const spl = await loadSplTokenModule();
  const ix = appendReadonlyExtra(await buildFiveInstruction(ctx, 'borrow_obligation_liquidity', {
    market: marketPubkey,
    reserve: reservePubkey,
    obligation: obligationPubkey,
    user_liquidity: setup.borrowerLiquidity,
    liquidity_supply: setup.liquiditySupply,
    market_authority: admin.publicKey,
    user_authority: borrower.publicKey,
  }, {
    amount,
  }), spl.TOKEN_PROGRAM_ID);
  return submitInstruction(ctx, ix, [ctx.payer, admin, borrower], step, options);
}

export async function repayObligationLiquidity(ctx, borrower, marketPubkey, reservePubkey, obligationPubkey, setup, amount, step = 'lending_repay_obligation_liquidity', options = {}) {
  const spl = await loadSplTokenModule();
  const ix = appendReadonlyExtra(await buildFiveInstruction(ctx, 'repay_obligation_liquidity', {
    market: marketPubkey,
    reserve: reservePubkey,
    obligation: obligationPubkey,
    user_liquidity: setup.borrowerLiquidity,
    liquidity_supply: setup.liquiditySupply,
    user_authority: borrower.publicKey,
  }, {
    amount,
  }), spl.TOKEN_PROGRAM_ID);
  return submitInstruction(ctx, ix, [ctx.payer, borrower], step, options);
}

export async function withdrawReserveLiquidity(ctx, admin, borrower, marketPubkey, reservePubkey, obligationPubkey, setup, collateralAmount, step = 'lending_withdraw_reserve_liquidity', options = {}) {
  const spl = await loadSplTokenModule();
  const ix = appendReadonlyExtra(await buildFiveInstruction(ctx, 'withdraw_reserve_liquidity', {
    market: marketPubkey,
    reserve: reservePubkey,
    obligation: obligationPubkey,
    user_liquidity: setup.borrowerLiquidity,
    user_collateral: setup.borrowerCollateral,
    liquidity_supply: setup.liquiditySupply,
    collateral_mint: setup.collateralMint,
    market_authority: admin.publicKey,
    user_authority: borrower.publicKey,
  }, {
    collateral_amount: collateralAmount,
  }), spl.TOKEN_PROGRAM_ID);
  return submitInstruction(ctx, ix, [ctx.payer, admin, borrower], step, options);
}

export async function readMarketState(ctx, marketPubkey) {
  const info = await readAccountInfo(ctx, marketPubkey);
  assertOrThrow(info, `Lending market account not found: ${marketPubkey.toBase58()}`);
  const data = info.data;
  return {
    admin: new PublicKey(data.subarray(0, 32)).toBase58(),
    quoteCurrency: new PublicKey(data.subarray(32, 64)).toBase58(),
    isPaused: data[64] === 1,
    abiVersion: data.readUInt16LE(65),
    protocolFeesCollected: Number(data.readBigUInt64LE(67)),
  };
}

export async function readReserveState(ctx, reservePubkey) {
  const info = await readAccountInfo(ctx, reservePubkey);
  assertOrThrow(info, `Reserve account not found: ${reservePubkey.toBase58()}`);
  const data = info.data;
  return {
    market: new PublicKey(data.subarray(0, 32)).toBase58(),
    liquidityMint: new PublicKey(data.subarray(32, 64)).toBase58(),
    liquiditySupply: new PublicKey(data.subarray(64, 96)).toBase58(),
    collateralMint: new PublicKey(data.subarray(96, 128)).toBase58(),
    collateralSupply: Number(data.readBigUInt64LE(128)),
    liquidityAvailable: Number(data.readBigUInt64LE(136)),
    borrowedAmount: Number(data.readBigUInt64LE(144)),
    cumulativeBorrowRate: Number(data.readBigUInt64LE(152)),
    lastUpdateSlot: Number(data.readBigUInt64LE(160)),
    protocolFees: Number(data.readBigUInt64LE(168)),
    optimalUtilizationRate: data[176],
    loanToValueRatio: data[177],
    liquidationThreshold: data[178],
    liquidationBonus: data[179],
    maxBorrowRate: data[180],
    minBorrowRate: data[181],
    reserveFactor: data[182],
    supplyCap: Number(data.readBigUInt64LE(183)),
  };
}

export async function readObligationState(ctx, obligationPubkey) {
  const info = await readAccountInfo(ctx, obligationPubkey);
  assertOrThrow(info, `Obligation account not found: ${obligationPubkey.toBase58()}`);
  const data = info.data;
  return {
    market: new PublicKey(data.subarray(0, 32)).toBase58(),
    owner: new PublicKey(data.subarray(32, 64)).toBase58(),
    depositedValue: Number(data.readBigUInt64LE(64)),
    borrowedValue: Number(data.readBigUInt64LE(72)),
    allowedBorrowValue: Number(data.readBigUInt64LE(80)),
  };
}

export async function readOracleState(ctx, oraclePubkey) {
  const info = await readAccountInfo(ctx, oraclePubkey);
  assertOrThrow(info, `Oracle account not found: ${oraclePubkey.toBase58()}`);
  const data = info.data;
  return {
    price: Number(data.readBigUInt64LE(0)),
    decimals: data[8],
    lastUpdate: Number(data.readBigUInt64LE(9)),
  };
}

export async function prepareLendingFixture(ctx, labelPrefix = 'lending') {
  const admin = await createUser(ctx, `${labelPrefix}_admin`);
  const borrower = await createUser(ctx, `${labelPrefix}_borrower`);

  const { mint: liquidityMint, tokenAccount: liquiditySupply } = await createSplMintAndAccount(
    ctx.connection,
    ctx.payer,
    admin
  );
  const collateralMint = await (await loadSplTokenModule()).createMint(
    ctx.connection,
    ctx.payer,
    admin.publicKey,
    null,
    6
  );
  const borrowerLiquidity = await createSplTokenAccount(ctx.connection, ctx.payer, liquidityMint, borrower);
  const borrowerCollateral = await createSplTokenAccount(ctx.connection, ctx.payer, collateralMint, borrower);
  await mintSplTo(ctx.connection, ctx.payer, liquidityMint, borrowerLiquidity, admin, 2_000_000n);

  const market = Keypair.generate();
  const reserve = Keypair.generate();
  const obligation = Keypair.generate();
  const oracle = Keypair.generate();

  await initMarket(ctx, admin, market, liquidityMint);
  await initReserve(ctx, admin, market.publicKey, reserve, {
    liquidityMint,
    liquiditySupply,
    collateralMint,
  }, {
    optimalUtilization: 80,
    loanToValue: 75,
    reserveFactor: 10,
    supplyCap: 10_000_000,
  });
  await initObligation(ctx, borrower, market.publicKey, obligation);
  await initOracle(ctx, admin, oracle, 1_000_000, 6);
  await setOracle(
    ctx,
    admin,
    oracle.publicKey,
    1_000_000,
    await currentSlot(ctx),
    6,
    `${labelPrefix}_set_oracle_fresh`
  );

  return {
    admin,
    borrower,
    market,
    reserve,
    obligation,
    oracle,
    setup: {
      liquidityMint,
      liquiditySupply,
      collateralMint,
      borrowerLiquidity,
      borrowerCollateral,
    },
  };
}

export async function currentSlot(ctx) {
  return withRpcRetries(ctx, () => ctx.connection.getSlot('confirmed'));
}
