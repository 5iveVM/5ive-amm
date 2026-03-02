#!/usr/bin/env node
import {
  assertLendingPreflight,
  assertOrThrow,
  borrowObligationLiquidity,
  currentSlot,
  depositReserveLiquidity,
  emitJourneyStep,
  loadLendingContext,
  prepareLendingFixture,
  readMarketState,
  readObligationState,
  refreshObligationWithOracle,
  repayObligationLiquidity,
  setMarketPause,
  setOracle,
  withdrawReserveLiquidity,
  writeScenarioArtifact,
} from '../lending/lib/user-journey-helpers.mjs';

async function expectFailure(result, message) {
  assertOrThrow(result.success === false, message);
}

async function main() {
  const ctx = await loadLendingContext();
  await assertLendingPreflight(ctx);

  const fixture = await prepareLendingFixture(ctx, 'lending_failure');

  await depositReserveLiquidity(
    ctx,
    fixture.admin,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.setup,
    400_000
  );
  await refreshObligationWithOracle(
    ctx,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.obligation.publicKey,
    fixture.reserve.publicKey,
    fixture.oracle.publicKey
  );

  const overBorrow = await borrowObligationLiquidity(
    ctx,
    fixture.admin,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.obligation.publicKey,
    fixture.setup,
    500_000,
    'lending_borrow_above_ltv',
    { allowFailure: true, expectedFailureClass: 'unknown' }
  );
  await expectFailure(overBorrow, 'over-borrow should fail');

  await setMarketPause(
    ctx,
    fixture.admin,
    fixture.market.publicKey,
    true,
    'lending_pause_market'
  );

  const pausedBorrow = await borrowObligationLiquidity(
    ctx,
    fixture.admin,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.obligation.publicKey,
    fixture.setup,
    100_000,
    'lending_borrow_while_paused',
    { allowFailure: true, expectedFailureClass: 'unknown' }
  );
  await expectFailure(pausedBorrow, 'borrow while paused should fail');

  await setMarketPause(
    ctx,
    fixture.admin,
    fixture.market.publicKey,
    false,
    'lending_unpause_market'
  );

  const slot = await currentSlot(ctx);
  await setOracle(
    ctx,
    fixture.admin,
    fixture.oracle.publicKey,
    1_000_000,
    Math.max(0, slot - 200),
    6,
    'lending_set_stale_oracle'
  );

  const staleRefresh = await refreshObligationWithOracle(
    ctx,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.obligation.publicKey,
    fixture.reserve.publicKey,
    fixture.oracle.publicKey,
    'lending_refresh_with_stale_oracle',
    { allowFailure: true, expectedFailureClass: 'account_fixture' }
  );
  await expectFailure(staleRefresh, 'refresh with stale oracle should fail');

  await setOracle(
    ctx,
    fixture.admin,
    fixture.oracle.publicKey,
    1_000_000,
    slot,
    6,
    'lending_refresh_oracle_current'
  );
  await refreshObligationWithOracle(
    ctx,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.obligation.publicKey,
    fixture.reserve.publicKey,
    fixture.oracle.publicKey,
    'lending_refresh_after_oracle_recovery'
  );

  await borrowObligationLiquidity(
    ctx,
    fixture.admin,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.obligation.publicKey,
    fixture.setup,
    200_000,
    'lending_borrow_recovery_success'
  );

  const badWithdraw = await withdrawReserveLiquidity(
    ctx,
    fixture.admin,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.obligation.publicKey,
    fixture.setup,
    350_000,
    'lending_unhealthy_withdraw',
    { allowFailure: true, expectedFailureClass: 'unknown' }
  );
  await expectFailure(badWithdraw, 'unhealthy withdraw should fail');

  await repayObligationLiquidity(
    ctx,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.obligation.publicKey,
    fixture.setup,
    100_000,
    'lending_repay_recovery'
  );

  const marketState = await readMarketState(ctx, fixture.market.publicKey);
  const obligationState = await readObligationState(ctx, fixture.obligation.publicKey);
  assertOrThrow(marketState.isPaused === false, 'market should be unpaused after recovery');
  assertOrThrow(obligationState.borrowedValue === 100_000, 'borrowed value should reflect recovery repay');

  emitJourneyStep({
    step: 'lending_failure_recovery_state_valid',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });

  writeScenarioArtifact(ctx, 'lending-failure-recovery.json', {
    market: fixture.market.publicKey.toBase58(),
    reserve: fixture.reserve.publicKey.toBase58(),
    obligation: fixture.obligation.publicKey.toBase58(),
    oracle: fixture.oracle.publicKey.toBase58(),
  });
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
