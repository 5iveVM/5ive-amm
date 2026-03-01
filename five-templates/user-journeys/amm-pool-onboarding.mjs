#!/usr/bin/env node
import { assertAmmPreflight, emitJourneyStep, loadAmmContext, prepareAmmFixture, readPoolState } from '../amm/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadAmmContext();
  await assertAmmPreflight(ctx);
  const { authority, pool } = await prepareAmmFixture(ctx, 'amm_onboarding');
  const state = await readPoolState(ctx, pool.publicKey);

  if (!(state.reserveA > 0 && state.reserveB > 0 && state.lpSupply > 0)) {
    throw new Error('AMM onboarding produced empty reserves or LP supply');
  }
  if (state.authority !== authority.publicKey.toBase58()) {
    throw new Error(`Pool authority mismatch: expected ${authority.publicKey.toBase58()}, got ${state.authority}`);
  }
  if (state.isPaused) {
    throw new Error('Pool unexpectedly paused after initialization');
  }
  emitJourneyStep({
    step: 'amm_pool_state_valid',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
