#!/usr/bin/env node
import {
  assertLendingPreflight,
  assertOrThrow,
  emitJourneyStep,
  loadLendingContext,
  prepareLendingFixture,
  readMarketState,
  readObligationState,
  readOracleState,
  readReserveState,
  writeScenarioArtifact,
} from '../lending/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadLendingContext();
  await assertLendingPreflight(ctx);

  const fixture = await prepareLendingFixture(ctx, 'lending_onboarding');
  const marketState = await readMarketState(ctx, fixture.market.publicKey);
  const reserveState = await readReserveState(ctx, fixture.reserve.publicKey);
  const obligationState = await readObligationState(ctx, fixture.obligation.publicKey);
  const oracleState = await readOracleState(ctx, fixture.oracle.publicKey);

  assertOrThrow(marketState.admin === fixture.admin.publicKey.toBase58(), 'market admin mismatch');
  assertOrThrow(marketState.isPaused === false, 'market should start unpaused');
  assertOrThrow(reserveState.market === fixture.market.publicKey.toBase58(), 'reserve market mismatch');
  assertOrThrow(reserveState.loanToValueRatio === 75, 'reserve loan-to-value mismatch');
  assertOrThrow(reserveState.reserveFactor === 10, 'reserve factor mismatch');
  assertOrThrow(obligationState.owner === fixture.borrower.publicKey.toBase58(), 'obligation owner mismatch');
  assertOrThrow(oracleState.price > 0, 'oracle price must be positive');
  assertOrThrow(oracleState.lastUpdate > 0, 'oracle must be fresh');

  emitJourneyStep({
    step: 'lending_market_onboarding_state_valid',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });

  writeScenarioArtifact(ctx, 'lending-market-onboarding.json', {
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
