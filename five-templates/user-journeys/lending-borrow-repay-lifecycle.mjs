#!/usr/bin/env node
import {
  assertLendingPreflight,
  assertOrThrow,
  depositReserveLiquidity,
  emitJourneyStep,
  loadLendingContext,
  prepareLendingFixture,
  borrowObligationLiquidity,
  readObligationState,
  readReserveState,
  readSplTokenBalance,
  refreshObligationWithOracle,
  repayObligationLiquidity,
  withdrawReserveLiquidity,
  writeScenarioArtifact,
} from '../lending/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadLendingContext();
  await assertLendingPreflight(ctx);

  const fixture = await prepareLendingFixture(ctx, 'lending_lifecycle');

  const depositAmount = 400_000;
  const borrowAmount = 200_000;
  const repayAmount = 100_000;
  const withdrawCollateralAmount = 100_000;

  await depositReserveLiquidity(
    ctx,
    fixture.admin,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.setup,
    depositAmount
  );
  await refreshObligationWithOracle(
    ctx,
    fixture.market.publicKey,
    fixture.obligation.publicKey,
    fixture.reserve.publicKey,
    fixture.oracle.publicKey
  );
  await borrowObligationLiquidity(
    ctx,
    fixture.admin,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.obligation.publicKey,
    fixture.setup,
    borrowAmount
  );
  await repayObligationLiquidity(
    ctx,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.obligation.publicKey,
    fixture.setup,
    repayAmount
  );
  await withdrawReserveLiquidity(
    ctx,
    fixture.admin,
    fixture.borrower,
    fixture.market.publicKey,
    fixture.reserve.publicKey,
    fixture.obligation.publicKey,
    fixture.setup,
    withdrawCollateralAmount
  );

  const reserveState = await readReserveState(ctx, fixture.reserve.publicKey);
  const obligationState = await readObligationState(ctx, fixture.obligation.publicKey);
  const borrowerLiquidityBalance = await readSplTokenBalance(ctx, fixture.setup.borrowerLiquidity);
  const borrowerCollateralBalance = await readSplTokenBalance(ctx, fixture.setup.borrowerCollateral);

  assertOrThrow(reserveState.collateralSupply > 0, 'collateral supply must be positive after deposit');
  assertOrThrow(reserveState.borrowedAmount > 0, 'borrowed amount must stay positive after partial repay');
  assertOrThrow(obligationState.borrowedValue === borrowAmount - repayAmount, 'borrowed value did not decrease after repay');
  assertOrThrow(obligationState.allowedBorrowValue > 0, 'allowed borrow value must be set');
  assertOrThrow(borrowerCollateralBalance > 0, 'borrower collateral balance must remain positive');
  assertOrThrow(borrowerLiquidityBalance > 0, 'borrower liquidity balance must remain readable');

  emitJourneyStep({
    step: 'lending_borrow_repay_lifecycle_state_valid',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });

  writeScenarioArtifact(ctx, 'lending-borrow-repay-lifecycle.json', {
    market: fixture.market.publicKey.toBase58(),
    reserve: fixture.reserve.publicKey.toBase58(),
    obligation: fixture.obligation.publicKey.toBase58(),
    oracle: fixture.oracle.publicKey.toBase58(),
    borrowerLiquidityBalance,
    borrowerCollateralBalance,
  });
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
