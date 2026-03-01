#!/usr/bin/env node
import {
  approveDelegate,
  assertJourneyPreflight,
  assertOrThrow,
  createUser,
  emitJourneyStep,
  ensureBalance,
  initMint,
  initTokenAccount,
  Keypair,
  loadJourneyContext,
  mintTo,
  revokeDelegate,
  transferFrom,
  transferTokens,
  writeScenarioArtifact,
} from '../token/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadJourneyContext();
  await assertJourneyPreflight(ctx);
  const userA = await createUser(ctx, 'lifecycle_user_a');
  const userB = await createUser(ctx, 'lifecycle_user_b');
  const mint = Keypair.generate();
  const userAToken = Keypair.generate();
  const userBToken = Keypair.generate();

  await initMint(ctx, userA, mint, 'LifecycleToken');
  await initTokenAccount(ctx, userA, userAToken, mint, 'init_token_account_user_a');
  await initTokenAccount(ctx, userB, userBToken, mint, 'init_token_account_user_b');
  await mintTo(ctx, mint, userAToken, userA, 1000, 'mint_to_user_a');
  await ensureBalance(ctx, userAToken.publicKey, 1000, 'verify_mint_balance_user_a');
  await ensureBalance(ctx, userBToken.publicKey, 0, 'verify_mint_balance_user_b');

  await transferTokens(ctx, userAToken, userBToken, userA, 250, 'transfer_user_a_to_user_b');
  await ensureBalance(ctx, userAToken.publicKey, 750, 'verify_transfer_balance_user_a');
  await ensureBalance(ctx, userBToken.publicKey, 250, 'verify_transfer_balance_user_b');

  await approveDelegate(ctx, userAToken, userA, userB, 150, 'approve_delegate_user_b');
  await transferFrom(ctx, userAToken, userBToken, userB, 100, 'delegate_transfer_from');
  await ensureBalance(ctx, userAToken.publicKey, 650, 'verify_delegate_balance_user_a');
  await ensureBalance(ctx, userBToken.publicKey, 350, 'verify_delegate_balance_user_b');

  await revokeDelegate(ctx, userAToken, userA, 'revoke_delegate');
  const postRevoke = await transferFrom(
    ctx,
    userAToken,
    userBToken,
    userB,
    1,
    'delegate_transfer_after_revoke',
    { allowFailure: true, expectedFailureClass: 'authority' }
  );
  assertOrThrow(!postRevoke.success, 'delegate transfer unexpectedly succeeded after revoke');
  emitJourneyStep({
    step: 'revoke_blocks_delegate_reuse',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'expected failure verified',
  });

  writeScenarioArtifact(ctx, 'token-lifecycle-two-users.json', {
    userA: userA.publicKey.toBase58(),
    userB: userB.publicKey.toBase58(),
    mint: mint.publicKey.toBase58(),
    userAToken: userAToken.publicKey.toBase58(),
    userBToken: userBToken.publicKey.toBase58(),
  });
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
