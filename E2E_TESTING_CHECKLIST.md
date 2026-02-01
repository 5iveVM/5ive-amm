# E2E Testing Checklist - Register-Optimized Token.v

**Status:** Ready to execute
**Artifact:** `five-templates/token/build/five-token-registers-opt.five`

## Phase 1: Deployment Preparation

- [ ] Verify register-optimized artifact exists
  ```bash
  ls -lh five-templates/token/build/five-token-registers-opt.five
  # Should show: ~1KB file size (base64 encoded bytecode)
  ```

- [ ] Confirm bytecode size
  ```bash
  # Should be 770 bytes (4.3% reduction from 805 baseline)
  node -e "
    const fs = require('fs');
    const artifact = JSON.parse(fs.readFileSync('five-templates/token/build/five-token-registers-opt.five'));
    const bytecode = Buffer.from(artifact.bytecode, 'base64');
    console.log('Bytecode size:', bytecode.length, 'bytes');
  "
  ```

- [ ] Ensure localnet is running
  ```bash
  # In separate terminal
  solana-test-validator

  # Verify with
  solana cluster-version
  ```

## Phase 2: Deployment

- [ ] Back up existing deployment config (if any)
  ```bash
  cd five-templates/token
  mv deployment-config.json deployment-config.json.backup 2>/dev/null || true
  ```

- [ ] Update deployment configuration to use register-optimized artifact
  ```bash
  # Edit deploy-to-five-vm.mjs or equivalent to point to:
  # five-token-registers-opt.five
  ```

- [ ] Deploy register-optimized bytecode
  ```bash
  npm run deploy

  # Expected output:
  # ✅ Script account deployed successfully
  # ✅ Bytecode chunks uploaded
  # ✅ VM state initialized
  # ✅ No error 8122 (critical!)
  ```

- [ ] Verify deployment success
  ```bash
  npm run test:debug-owner

  # Expected:
  # ✅ Script account owner: Five VM program ID
  # ✅ VM state account initialized
  # ✅ Accounts properly linked
  ```

## Phase 3: Execute E2E Tests

- [ ] Run full test suite
  ```bash
  npm run test:e2e 2>&1 | tee e2e-results.log

  # Expected: All 14 functions pass
  ```

- [ ] Verify each function executes
  ```
  ✓ init_mint succeeded
  ✓ init_token_account succeeded (x3)
  ✓ mint_to succeeded (x3)
  ✓ transfer succeeded
  ✓ approve succeeded
  ✓ transfer_from succeeded
  ✓ revoke succeeded
  ✓ burn succeeded
  ✓ freeze_account succeeded
  ✓ thaw_account succeeded
  ✓ set_mint_authority succeeded
  ✓ set_freeze_authority succeeded
  ✓ disable_mint succeeded
  ✓ disable_freeze succeeded
  ```

## Phase 4: Capture CU Measurements

### Option A: Extract from E2E Test Output
If e2e-token-test.mjs includes CU logging:
```bash
grep "CU" e2e-results.log > cu-results.txt
cat cu-results.txt
```

### Option B: Modify E2E Test
Update `five-templates/token/e2e-token-test.mjs` to capture CU:
```javascript
// At end of each function call, capture CU
const tx = await connection.getTransaction(signature);
const cuUsed = tx.meta.computeUnitsConsumed;
console.log(`${functionName}: ${cuUsed} CU`);
```

### Option C: Run Comparison Script
```bash
# If compare script exists
node compare-baseline-vs-registers.mjs
```

Expected results:
```
┌──────────────────┬──────────┬──────────┬──────────┐
│ Function         │ Baseline │ Register │ Savings  │
├──────────────────┼──────────┼──────────┼──────────┤
│ init_mint        │  10,777  │   ~9,160 │   -15%   │
│ mint_to          │  12,234  │  ~10,399 │   -15%   │
│ transfer         │  <4,000  │  ~3,400  │   -15%   │
│ approve          │  ~2,500  │  ~2,125  │   -15%   │
│ burn             │  ~8,100  │  ~6,885  │   -15%   │
└──────────────────┴──────────┴──────────┴──────────┘
```

## Phase 5: Validation & Documentation

- [ ] Verify all tests passed
  - [ ] 0 test failures
  - [ ] All 14 functions executed
  - [ ] No error 8122 (bytecode verification)
  - [ ] No IllegalOwner errors
  - [ ] No transaction failures

- [ ] Capture actual CU data
  ```bash
  # Save results
  cat e2e-results.log > register-e2e-results.txt
  cat cu-results.txt > register-cu-results.txt
  ```

- [ ] Compare vs baseline
  - [ ] Calculate actual CU reduction
  - [ ] Document which functions benefit most
  - [ ] Note any functions with minimal savings

- [ ] Update documentation
  ```markdown
  # Register Optimization - Verified Results

  **Bytecode Size:** 770 bytes (4.3% reduction)
  **Compute Units:** [actual measured savings]
  **Functions Tested:** 14/14 ✅
  **Deployment:** Success ✅

  [Detailed results table]
  ```

## Troubleshooting

### Error 8122: Bytecode Verification Failed
**Cause:** JUMP instruction targeting out-of-bounds address
**Action:**
1. Check bytecode structure with `node analyze-jumps.js`
2. Enhance `recalculate_label_positions()` method
3. Contact support if persists

### IllegalOwner Error
**Cause:** Script account ownership incorrect
**Action:**
```bash
npm run test:debug-owner
# Follow remediation steps shown
```

### Transaction Failures
**Cause:** Bytecode execution error
**Action:**
1. Check VM state account initialization
2. Verify account constraints
3. Review function parameters in test

### Low CU Savings
**Cause:** Function doesn't use optimizable patterns
**Action:**
- Expected: Some functions may show <15% savings
- Documented: Transfer showed <4k CU baseline
- Note which functions benefit most

## Success Criteria

✅ **Deployment Phase:**
- [ ] No error 8122
- [ ] All bytecode chunks uploaded
- [ ] Script account owned by Five VM program

✅ **E2E Testing Phase:**
- [ ] All 14 functions pass
- [ ] No execution errors
- [ ] All transactions confirmed

✅ **CU Measurement Phase:**
- [ ] Actual CU captured for each function
- [ ] Comparison vs baseline possible
- [ ] Results documented

✅ **Overall Success:**
- [ ] Register optimization works correctly
- [ ] Measurable CU improvements demonstrated
- [ ] Option A approach validated

---

## Quick Command Reference

```bash
# Compile with registers
cd five-dsl-compiler
cargo run --bin five -- compile \
  ../five-templates/token/src/token.v \
  --enable-registers --use-linear-scan \
  --output ../five-templates/token/build/token-registers-opt.fbin

# Create artifact
cd ../five-templates/token
node create-artifact.js  # modify to use registers-opt bytecode

# Deploy
npm run deploy

# Test
npm run test:e2e

# Measure
node compare-baseline-vs-registers.mjs
```

---

**Ready to execute when you're prepared to deploy and test on-chain.**
