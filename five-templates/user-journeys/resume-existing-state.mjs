#!/usr/bin/env node
import fs from 'fs';
import path from 'path';
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
  PublicKey,
  transferTokens,
  writeScenarioArtifact,
} from '../token/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadJourneyContext();
  await assertJourneyPreflight(ctx);
  const owner = await createUser(ctx, 'resume_owner');
  const receiver = await createUser(ctx, 'resume_receiver');
  const mint = Keypair.generate();
  const ownerToken = Keypair.generate();
  const receiverToken = Keypair.generate();

  await initMint(ctx, owner, mint, 'ResumeToken');
  await initTokenAccount(ctx, owner, ownerToken, mint, 'init_token_account_resume_owner');
  await initTokenAccount(ctx, receiver, receiverToken, mint, 'init_token_account_resume_receiver');
  await mintTo(ctx, mint, ownerToken, owner, 400, 'mint_to_resume_owner');

  const artifact = {
    owner: owner.publicKey.toBase58(),
    receiver: receiver.publicKey.toBase58(),
    mint: mint.publicKey.toBase58(),
    ownerToken: ownerToken.publicKey.toBase58(),
    receiverToken: receiverToken.publicKey.toBase58(),
  };
  writeScenarioArtifact(ctx, 'resume-existing-state.json', artifact);
  emitJourneyStep({
    step: 'persist_resume_metadata',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'local artifact write',
  });

  const ctxReloaded = await loadJourneyContext();
  const artifactPath = path.join(ctxReloaded.scenarioArtifactDir, 'resume-existing-state.json');
  const saved = JSON.parse(fs.readFileSync(artifactPath, 'utf8'));
  await ensureBalance(ctxReloaded, new PublicKey(saved.ownerToken), 400, 'resume_read_existing_owner_balance');
  await ensureBalance(ctxReloaded, new PublicKey(saved.receiverToken), 0, 'resume_read_existing_receiver_balance');

  const duplicateInit = await initTokenAccount(
    ctxReloaded,
    owner,
    ownerToken,
    mint,
    'resume_duplicate_init_existing_token',
    { allowFailure: true, expectedFailureClass: 'already_initialized' }
  );
  assertOrThrow(!duplicateInit.success, 'existing account was recreated unexpectedly');
  emitJourneyStep({
    step: 'existing_account_not_recreated',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'expected duplicate init failure',
  });

  await transferTokens(ctxReloaded, ownerToken, receiverToken, owner, 125, 'resume_transfer_after_reload');
  await ensureBalance(ctxReloaded, new PublicKey(saved.ownerToken), 275, 'resume_verify_owner_balance_after_transfer');
  await ensureBalance(ctxReloaded, new PublicKey(saved.receiverToken), 125, 'resume_verify_receiver_balance_after_transfer');
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
