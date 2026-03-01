#!/usr/bin/env node
import {
  addLiquidity,
  assertAmmPreflight,
  assertOrThrow,
  emitJourneyStep,
  loadAmmContext,
  prepareAmmFixture,
  readPoolState,
  setPaused,
  swapTokens,
} from '../amm/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadAmmContext();
  await assertAmmPreflight(ctx);
  const { authority, pool, setup } = await prepareAmmFixture(ctx, 'amm_failure');

  const slippageFailure = await swapTokens(
    ctx,
    authority,
    pool,
    setup,
    5_000,
    1_000_000_000,
    true,
    'amm_swap_impossible_min_out',
  ).catch((error) => ({ success: false, error }));
  assertOrThrow(!slippageFailure.success, 'AMM swap unexpectedly succeeded with impossible min_amount_out');

  const ratioFailure = await addLiquidity(
    ctx,
    authority,
    pool,
    setup,
    10_000,
    9_000,
    1,
    'amm_add_liquidity_wrong_ratio',
  ).catch((error) => ({ success: false, error }));
  assertOrThrow(!ratioFailure.success, 'AMM add_liquidity unexpectedly succeeded with wrong ratio');

  await setPaused(ctx, authority, pool, true, 'amm_pause_pool');
  const pausedFailure = await swapTokens(
    ctx,
    authority,
    pool,
    setup,
    1_000,
    1,
    true,
    'amm_swap_while_paused',
  ).catch((error) => ({ success: false, error }));
  assertOrThrow(!pausedFailure.success, 'AMM swap unexpectedly succeeded while paused');

  await setPaused(ctx, authority, pool, false, 'amm_unpause_pool');
  await swapTokens(ctx, authority, pool, setup, 1_000, 1, true, 'amm_recovery_swap');

  const state = await readPoolState(ctx, pool.publicKey);
  if (state.isPaused) {
    throw new Error('Pool remained paused after recovery flow');
  }
  emitJourneyStep({
    step: 'amm_recovery_flow_valid',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
