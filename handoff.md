# HANDOFF

## Scope
This handoff captures current status for the `5ive-*` consolidation/readiness effort in `/Users/ivmidable/Development/five-mono`.

## What Is Already Implemented
- Canonical lending is set to `5ive-lending-2` (`src/main.v`) with ABI metadata/changelog.
- Deprecated lending variants are kept and clearly marked:
  - `5ive-lending`
  - `5ive-lending-3`
  - `5ive-lending-4`
- Standardized scripts were added across all `5ive-*` projects:
  - `test:onchain:local`, `test:onchain:devnet`
  - `deploy:local`, `deploy:devnet`
  - `verify`
  - plus `client:run:local` and `client:run:devnet` where clients exist.
- Shared client env contract introduced (`FIVE_NETWORK`, `FIVE_RPC_URL`, `FIVE_VM_PROGRAM_ID`, `FIVE_SCRIPT_ACCOUNT`, `FIVE_PAYER_PATH`) via `client/env.ts` in client-enabled projects.
- Matrix scripts added:
  - `scripts/verify-5ive-projects.sh`
  - `scripts/verify-5ive-devnet.sh`
- CI workflow added:
  - `.github/workflows/verify-5ive.yml`

## Baseline/Current Blockers
### Toolchain mismatch
- Current environment observed:
  - Node: `v24.10.0`
  - npm: `11.6.1`
  - solana-cli: `3.1.8`
  - 5ive CLI: `1.0.26`
- Plan expects Node 20 for consistency with CI.

### Deterministic compile/unit blockers
- `5ive-amm` build fails with `E1000` type mismatch (`src/main.v`).
- `5ive-cfd` build fails with syntax error (`unexpected 'if'` in `src/main.v`).
- `5ive-esccrow` test fixture parse issue: `tests/main.test.json` (`testCases is not iterable`).

### Client TypeScript blockers
- NodeNext import resolution errors after env module addition:
  - Relative imports require explicit extension (`./env.js`) in client TS files.
- `5ive-cfd/client/main.ts` has additional SDK typing mismatches around generated instruction usage.

### Localnet runtime blockers
- Multiple on-chain tests fail with:
  - `ProgramAccountNotFound`
- This indicates local VM program/state initialization prerequisites are not consistently satisfied before per-project tests.

## Files To Recheck First
- `5ive-amm/src/main.v`
- `5ive-cfd/src/main.v`
- `5ive-cfd/client/main.ts`
- `5ive-esccrow/tests/main.test.json`
- `scripts/verify-5ive-projects.sh`
- `scripts/verify-5ive-devnet.sh`
- `.github/workflows/verify-5ive.yml`

## Recommended Next Steps (in order)
1. Lock execution env to Node 20 (or update docs/CI policy if staying on Node 24).
2. Fix deterministic compile/unit failures:
   - `5ive-amm` compile
   - `5ive-cfd` compile/tests
   - `5ive-esccrow` fixture/test parsing
3. Normalize all client TS imports to `./env.js` and resolve `5ive-cfd` SDK typing compile errors.
4. Add localnet preflight in `verify-5ive-projects.sh`:
   - RPC reachable
   - VM program exists
   - VM state account exists
   - payer funded
   - auto-init local VM state when missing
5. Re-run local matrix until `.reports/5ive-validation.json` is fully green (`allGreen: true`).
6. Run devnet matrix and classify blockers by:
   - compile/config
   - authority/permissions
   - account fixtures
   - funding/RPC
7. Extend devnet report payload with signature/meta.err/CU where available.
8. Once stable, make `verify-devnet` required (remove `continue-on-error: true`).

## Acceptance Criteria for “100%”
- `./scripts/verify-5ive-projects.sh` exits 0 and `.reports/5ive-validation.json` has zero failures.
- `./scripts/verify-5ive-devnet.sh` reports zero blocked steps.
- Client smoke flows pass for `5ive-cfd`, `5ive-esccrow`, `5ive-token`, `5ive-token-2` on localnet and devnet.
- CI has both `verify-local` and `verify-devnet` as required checks.

## Notes
- Workspace contains pre-existing dirty/untracked changes in multiple nested project repos. Preserve unrelated work while applying targeted fixes.
