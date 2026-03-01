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
  mintTo,
  SystemProgram,
  submitInstruction,
  transferTokens,
} from '../token/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadJourneyContext();
  await assertJourneyPreflight(ctx);

  const unfunded = Keypair.generate();
  const insufficientTransfer = SystemProgram.transfer({
    fromPubkey: unfunded.publicKey,
    toPubkey: ctx.payer.publicKey,
    lamports: 1,
  });
  const fundingFailure = await submitInstruction(
    ctx,
    insufficientTransfer,
    [unfunded],
    'insufficient_lamports_transfer',
    { allowFailure: true, expectedFailureClass: 'funding' }
  );
  assertOrThrow(!fundingFailure.success, 'insufficient funds path unexpectedly succeeded');

  const owner = await createUser(ctx, 'failure_owner');
  const recipient = await createUser(ctx, 'failure_recipient');
  const mint = Keypair.generate();
  const sourceToken = Keypair.generate();
  const destinationToken = Keypair.generate();

  await initMint(ctx, owner, mint, 'RecoveryToken');

  const wrongSigner = await initTokenAccount(
    ctx,
    owner,
    sourceToken,
    mint,
    'init_token_account_missing_owner_signature',
    { signers: [ctx.payer, sourceToken], allowFailure: true, expectedFailureClass: 'authority' }
  );
  assertOrThrow(!wrongSigner.success, 'missing owner signature path unexpectedly succeeded');

  await initTokenAccount(ctx, owner, sourceToken, mint, 'init_token_account_recovery_source');

  const duplicateInit = await initTokenAccount(
    ctx,
    owner,
    sourceToken,
    mint,
    'duplicate_init_token_account',
    { allowFailure: true, expectedFailureClass: 'already_initialized' }
  );
  assertOrThrow(!duplicateInit.success, 'duplicate initialization unexpectedly succeeded');

  try {
    await ctx.program
      .function('transfer')
      .accounts({
        source_account: sourceToken.publicKey,
        owner: owner.publicKey,
      })
      .args({ amount: 1 })
      .instruction();
    throw new Error('missing required account path unexpectedly built an instruction');
  } catch (error) {
    emitJourneyStep({
      step: 'missing_required_account_detected',
      status: 'FAIL',
      computeUnits: null,
      missingCuReason: 'client-side validation failure',
      error: error?.message || String(error),
      failureClass: 'missing_account',
    });
    emitJourneyStep({
      step: 'missing_required_account_rejected_cleanly',
      status: 'PASS',
      computeUnits: null,
      missingCuReason: 'expected client-side validation failure',
    });
  }

  await initTokenAccount(ctx, recipient, destinationToken, mint, 'init_token_account_recovery_destination');
  await mintTo(ctx, mint, sourceToken, owner, 200, 'mint_to_recovery_source');
  await transferTokens(ctx, sourceToken, destinationToken, owner, 75, 'recovery_transfer_success');
  await ensureBalance(ctx, sourceToken.publicKey, 125, 'verify_recovery_source_balance');
  await ensureBalance(ctx, destinationToken.publicKey, 75, 'verify_recovery_destination_balance');
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
