#!/usr/bin/env node
import {
  assertJourneyPreflight,
  assertOrThrow,
  createUser,
  emitJourneyStep,
  ensureBalance,
  initMint,
  initTokenAccount,
  Keypair,
  loadJourneyContext,
} from '../token/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadJourneyContext();
  await assertJourneyPreflight(ctx);
  const owner = await createUser(ctx, 'duplicate_owner');
  const mint = Keypair.generate();
  const tokenAccount = Keypair.generate();

  await initMint(ctx, owner, mint, 'DuplicateToken');
  await initTokenAccount(ctx, owner, tokenAccount, mint, 'duplicate_first_submit');
  await ensureBalance(ctx, tokenAccount.publicKey, 0, 'duplicate_verify_initial_balance');

  const secondSubmit = await initTokenAccount(
    ctx,
    owner,
    tokenAccount,
    mint,
    'duplicate_second_submit',
    { allowFailure: true, expectedFailureClass: 'duplicate_submit' }
  );
  assertOrThrow(!secondSubmit.success, 'duplicate submit unexpectedly succeeded');
  await ensureBalance(ctx, tokenAccount.publicKey, 0, 'duplicate_verify_balance_unchanged');
  emitJourneyStep({
    step: 'duplicate_submit_exactly_once_outcome',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
