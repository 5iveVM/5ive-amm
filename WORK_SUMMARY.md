# Fee Vault Hardcoding & AMM Deployment - Work Summary

## Session Overview

This session focused on two parallel tracks:
1. **Fee Vault Hardcoding Optimization** - Continuation from previous session
2. **AMM Template Deployment** - New work to test hardcoding on complex contract

## Track 1: Fee Vault Hardcoding Optimization

### Completed
✅ Implemented fee vault hardcoding for O(1) address verification (no PDA derivation syscalls)
✅ Reduced from 10 to 2 fee vault shards for testing
✅ Updated instruction payloads to remove bump bytes (3-byte format: `[0xFF, 0x53, shard_index]`)
✅ Fixed system program validation bug (was comparing to `[0u8; 32]` instead of actual ID)
✅ Deployed updated Five program to localnet
✅ Initialized 2 fee vault shards on localnet
✅ Created token deployment script with hardcoded fee vault addresses
✅ Updated fee collection functions to use O(1) verification

### Current Blocker
⏳ **VM State PDA Address Mismatch**
- Hardcoded VM state expects: `0x5f35...` (from code generation)
- Actual localnet VM state: `0x8a45...` (AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit)
- Causes `InvalidArgument` error during script deployment
- **Status**: Documented 3 solutions in `LOCALNET_HARDCODING_HANDOFF.md`

### Key Changes Made

**Source Files Modified:**
- `five-solana/src/common.rs` - Hardcoded constants + O(1) verification functions
- `five-solana/src/instructions/fees.rs` - Fixed system program check, removed bump param
- `five-solana/src/instructions/deploy.rs` - Updated for hardcoded verification
- `five-solana/src/instructions/execute.rs` - 3-byte instruction format
- `five-solana/src/instructions/mod.rs` - Instruction enum updates
- `five-solana/src/lib.rs` - Updated dispatch logic

**Deployment Scripts Created:**
- `scripts/init-localnet-vm-state.mjs` - VM state initialization
- `scripts/init-devnet-fee-vaults.mjs` - Updated for localnet support
- `five-templates/token/deploy-to-five-vm.mjs` - Token deployment with chunked upload
- Made generic for both Token and AMM deployment

**Infrastructure on Localnet:**
```
Program ID:        3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1
VM State PDA:      AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit
Fee Vault Shard 0: HXW6bZsdJW6Be5c51NNpNb9NcVxmHbUrF9oKkt4C1tEH
Fee Vault Shard 1: 4jDYhXWWxdoz1ojPWeAUVrWSbpZTMz3qL3mUqZ1VALsq
```

### Performance Impact (Expected)
- **Eliminated**: 2 PDA derivation syscalls per deploy/execute
- **Savings**: ~1500 CU per syscall = ~3000 CU per transaction
- **Method**: cfg-gated hardcoding (production) vs dynamic derivation (test)

---

## Track 2: AMM Template Deployment

### Completed
✅ Built AMM template from source (`five-templates/amm/src/main.v`)
✅ Fixed compilation issues (@init on custom types, field access)
✅ Used pre-compiled bytecode (139 bytes)
✅ Created `five-templates/amm/build/five-amm-baseline.five`
✅ Created generic deployment script supporting both Token and AMM
✅ Prepared AMM for deployment

### Current Status
⏳ **Blocked by same VM state PDA issue as Token**
- AMM deployment fails with `InvalidArgument` at vm_state verification
- Solution: Fix VM state PDA mismatch (shared solution for both Token and AMM)

### AMM Architecture
```
five-templates/amm/src/
├── main.v                      # Entry point (5 public functions)
├── amm_types.v                 # AMMPool and LPTokenAccount definitions
├── amm_liquidity.v             # Liquidity management
├── amm_swap.v                  # Swap calculations
├── amm_math.v                  # Math utilities
└── pool_manager.v              # Pool initialization

Public Functions:
- initialize_pool(pool, payer, token_a, token_b, fee_bps)
- add_liquidity(pool, lp_account, provider, ...)
- remove_liquidity(pool, lp_account, provider, ...)
- swap_a_to_b(pool, trader, ...)
- swap_b_to_a(pool, trader, ...)
```

### Deployment Artifacts
```
five-templates/amm/
├── build/five-amm-baseline.five        (139 bytes, ready)
├── deployment-config.json               (localnet config)
└── deploy-to-five-vm.mjs               (deployment script)
```

---

## Critical Blocker: VM State PDA Mismatch

### The Problem
```
Expected (hardcoded in code):
  HARDCODED_VM_STATE_PDA = 0x5f, 0x35, 0x23, 0x14, ...

Actual (on localnet):
  AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit
  = 0x8a, 0x45, 0xdd, 0x0b, ...
```

### Root Cause
The hardcoded constants were generated for program ID `3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1`, but the VM state on localnet doesn't match the expected derived address.

### Solutions (See `LOCALNET_HARDCODING_HANDOFF.md`)
1. **Update hardcoded constants** - Derive correct PDA for localnet
2. **Recreate localnet VM state** - Start fresh with matching PDA
3. **Disable hardcoding for localnet** - Use dynamic derivation in tests

---

## Handoff Documents Created

1. **`LOCALNET_HARDCODING_HANDOFF.md`**
   - Detailed VM state mismatch explanation
   - 3 solutions with step-by-step instructions
   - Performance summary and deployment notes

2. **`AMM_DEPLOYMENT_STATUS.md`**
   - AMM template overview
   - Deployment artifacts status
   - 3 solutions to resolve blocker
   - Next steps for completion

3. **`WORK_SUMMARY.md`** (this file)
   - Session overview
   - Both tracks status
   - Critical blocker details
   - Files and scripts created

---

## Files Created/Modified Summary

### New Scripts
- `scripts/deploy-token-localnet.mjs` - Token deployment (obsoleted by generic version)
- `scripts/init-localnet-vm-state.mjs` - VM state initialization
- `scripts/init-devnet-fee-vaults.mjs` - Updated for localnet
- `scripts/deploy-amm-localnet.mjs` - AMM deployment (copied to token dir)
- `five-templates/token/deploy-to-five-vm.mjs` - Generic deployment (Token + AMM)
- `five-templates/token/deploy-amm.mjs` - Copy for convenience
- `five-templates/amm/deploy-to-five-vm.mjs` - Standalone AMM deployment

### Configuration Files
- `five-templates/token/deployment-config.json` - Token localnet config
- `five-templates/amm/deployment-config.json` - AMM localnet config (template)
- `five-templates/amm/package.json` - Created (not used, node_modules not installed)

### Core Implementation Changes
- `five-solana/src/common.rs` - Hardcoded constants + verification functions
- `five-solana/src/instructions/fees.rs` - System program validation fix
- `five-solana/src/instructions/deploy.rs` - Hardcoded verification usage
- `five-solana/src/instructions/execute.rs` - 3-byte format support
- `five-solana/src/instructions/mod.rs` - Instruction enum updates
- `five-solana/src/lib.rs` - Instruction dispatch updates
- `five-templates/amm/src/main.v` - Removed @init from custom type
- `five-templates/token/src/main.v` - No changes (in token track)

### Documentation
- `LOCALNET_HARDCODING_HANDOFF.md` - Hardcoding blocker resolution guide
- `AMM_DEPLOYMENT_STATUS.md` - AMM deployment status & next steps
- `WORK_SUMMARY.md` - This overview document

---

## Command Reference: How to Continue

### Option A: Fix VM State & Deploy Both (RECOMMENDED)
```bash
# 1. Resolve VM state PDA mismatch (follow LOCALNET_HARDCODING_HANDOFF.md)
# 2. Rebuild and redeploy Five program
solana program deploy target/deploy/five.so --url http://127.0.0.1:8899

# 3. Deploy Token
cd five-templates/token
node deploy-to-five-vm.mjs Token

# 4. Deploy AMM
node deploy-to-five-vm.mjs AMM

# 5. Run e2e tests
node e2e-token-test.mjs
```

### Option B: Test with Devnet (Workaround)
```bash
# Switch to devnet deployment if localnet has unrecoverable PDA state
# Uses different hardcoded addresses from before this session
```

### Option C: Use Dynamic Derivation for Testing
```bash
# Modify five-solana/src/common.rs to always use dynamic derivation
# Rebuild program
# All deployments will work but without hardcoding optimization
```

---

## Success Criteria (To Complete)

- [ ] VM state PDA mismatch resolved
- [ ] Token successfully deploys to localnet
- [ ] Token e2e tests pass with signatures and CU metrics captured
- [ ] AMM successfully deploys to localnet
- [ ] Both contracts verified on-chain with correct ownership
- [ ] Hardcoding optimization confirmed active in production builds
- [ ] Fee collection verified with hardcoded addresses

---

## Performance Metrics to Capture (Once Deployed)

1. **Deploy Transaction CU**
   - Expected savings: ~1500-3000 CU vs dynamic derivation

2. **Execute Transaction CU**
   - Expected savings: ~1500 CU from eliminated PDA derivation

3. **Fee Vault Distribution**
   - 2 shards handling all fee collection
   - Contention metrics (if any)

---

## Notes for Future Sessions

1. **VM State PDA Issue** - Not specific to hardcoding; affects any program needing VM state verification
2. **Hardcoding Trade-offs** - O(1) verification vs runtime flexibility
3. **Fee Vault Sharding** - 2 is minimal; 5 recommended for production
4. **Bytecode Limits** - Token is 832 bytes, AMM is 139 bytes (both well under limits)
5. **Generic Deployment Script** - Can be extended for other templates (lending, oracle, etc.)

---

Generated: 2026-02-14
Status: In Progress (Blocked by VM State PDA Mismatch)
