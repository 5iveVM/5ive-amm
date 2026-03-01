#!/usr/bin/env node
import {
  assertJourneyPreflight,
  assertOrThrow,
  createUser,
  emitJourneyStep,
  initMint,
  Keypair,
  loadJourneyContext,
  readMintAuthority,
  recordWalletReadable,
  writeScenarioArtifact,
} from '../token/lib/user-journey-helpers.mjs';

async function main() {
  const ctx = await loadJourneyContext();
  await assertJourneyPreflight(ctx);
  const user = await createUser(ctx, 'onboarding_user');
  await recordWalletReadable(ctx, user, 'onboarding_user');

  const mintAccount = Keypair.generate();
  const initRes = await initMint(ctx, user, mintAccount, 'OnboardingToken');
  assertOrThrow(initRes.success, 'first user action failed');

  const authority = await readMintAuthority(ctx, mintAccount.publicKey);
  assertOrThrow(authority === user.publicKey.toBase58(), 'mint authority did not match onboarding user');
  emitJourneyStep({
    step: 'mint_state_readable',
    status: 'PASS',
    computeUnits: null,
    missingCuReason: 'state verification',
  });

  writeScenarioArtifact(ctx, 'wallet-onboarding.json', {
    user: user.publicKey.toBase58(),
    mint: mintAccount.publicKey.toBase58(),
  });
}

main().catch((error) => {
  console.error(error?.stack || error?.message || String(error));
  process.exit(1);
});
