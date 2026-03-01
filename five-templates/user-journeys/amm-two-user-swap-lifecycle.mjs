#!/usr/bin/env node
import {
  assertAmmPreflight,
  collectProtocolFees,
  emitJourneyStep,
  loadAmmContext,
  prepareAmmFixture,
  readPoolState,
  readSplTokenBalance,
  removeLiquidity,
  swapTokens,
} from '../amm/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadAmmContext();
  await assertAmmPreflight(ctx);
  const { authority, trader, pool, setup } = await prepareAmmFixture(ctx, 'amm_lifecycle');

  const beforeTraderB = await readSplTokenBalance(ctx, setup.traderTokenB);
  const beforePool = await readPoolState(ctx, pool.publicKey);

  const swap = await swapTokens(ctx, authority, pool, setup, 10_000, 1, true, 'amm_swap_a_to_b');
  if (!swap.success) throw new Error('AMM swap failed');

  const afterTraderB = await readSplTokenBalance(ctx, setup.traderTokenB);
  const afterPool = await readPoolState(ctx, pool.publicKey);

  if (afterTraderB <= beforeTraderB) {
    throw new Error('Trader destination balance did not increase after AMM swap');
  }
  if (!(afterPool.reserveA > beforePool.reserveA && afterPool.reserveB < beforePool.reserveB)) {
    throw new Error('AMM reserves did not move in the expected direction after swap');
  }
  if (!(afterPool.protocolFeesA > 0 || afterPool.protocolFeesB > 0)) {
    throw new Error('AMM protocol fees did not accrue after swap');
  }
  emitJourneyStep({
    step: 'amm_swap_state_valid',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });

  await removeLiquidity(ctx, authority, pool, setup, 100_000, 1, 1, 'amm_remove_partial_liquidity');
  const afterRemove = await readPoolState(ctx, pool.publicKey);
  if (!(afterRemove.lpSupply < afterPool.lpSupply)) {
    throw new Error('LP supply did not decrease after AMM liquidity removal');
  }

  await collectProtocolFees(ctx, authority, pool, setup);
  const afterCollect = await readPoolState(ctx, pool.publicKey);
  if (!(afterCollect.protocolFeesA === 0 && afterCollect.protocolFeesB === 0)) {
    throw new Error('Protocol fees were not fully cleared after collection');
  }
  emitJourneyStep({
    step: 'amm_collect_protocol_fees_clears_counters',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });

  void trader;
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
