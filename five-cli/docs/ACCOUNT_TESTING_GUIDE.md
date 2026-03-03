# Five Account System Testing Guide

For Builders: How to Test Account-Based Scripts in Five

## Overview

This guide explains how to write and test Five scripts that work with accounts - the most powerful feature of Five VM.

**Key Insight**: Five's account system has both **local and on-chain** testing modes:
- **Local tests** validate your function logic quickly (milliseconds)
- **On-chain tests** enforce real account constraints via Solana (production-ready)

This guide covers building tests that work in both modes seamlessly.

---

## Quick Start: Your First Account Test

### 1. Define Your Account in Five DSL

```v
// my-counter.v
account CounterState {
    owner: pubkey;
    count: u64;
}

pub increment(state: CounterState @mut) -> u64 {
    state.count = state.count + 1;
    return state.count;
}

pub set_count(state: CounterState @mut, new_value: u64) {
    state.count = new_value;
}
```

### 2. Create Test Fixtures (SDK/Node.js)

```typescript
import { AccountTestFixture, FixtureTemplates } from '@five-vm/sdk/testing';

// Simple counter fixture
const fixture = new AccountTestFixture()
    .addStateAccount('state', {
        owner: 'wallet address',
        count: 0
    })
    .build();

// Or use a predefined template
const counterFixture = FixtureTemplates.stateCounter()
    .build();
```

### 3. Execute Tests

```bash
# Local WASM execution (instant feedback)
five local execute my-counter.v 0

# On-chain execution (real constraints)
./test-runner.sh --onchain --network localnet --category 04-account-system
```

---

## Account Constraint Patterns

### Pattern 1: State Mutation (@mut)

**Use case**: Modify account data (counters, flags, tracking)

```v
account TokenState {
    supply: u64;
    minted: u64;
}

pub mint_tokens(state: TokenState @mut, amount: u64) -> u64 {
    require(amount > 0);
    state.minted = state.minted + amount;
    require(state.minted <= state.supply);
    return state.minted;
}
```

**Test Setup**:
```typescript
const fixture = new AccountTestFixture()
    .addStateAccount('state', {
        supply: 1000000,
        minted: 0
    })
    .build();

// Execute: mint_tokens(state, 100)
// Expected: state.minted = 100
```

**Key Points**:
- `@mut` allows writing to account fields
- Validation happens with `require()` checks
- Local tests verify logic, on-chain tests enforce constraints

---

### Pattern 2: Authorization (@signer)

**Use case**: Check transaction signer permissions

```v
account AdminState {
    admin: pubkey;
    authorized_count: u64;
}

pub authorize_user(
    admin: account @signer,
    state: AdminState @mut,
    new_user: pubkey
) -> bool {
    // @signer constraint: transaction must be signed by 'admin'
    require(admin.ctx.key == state.admin);

    state.authorized_count = state.authorized_count + 1;
    return true;
}
```

**Test Setup**:
```typescript
const fixture = new AccountTestFixture()
    .addSignerAccount('admin')  // Has keypair for signing
    .addStateAccount('state', {
        admin: '<admin-pubkey>',
        authorized_count: 0
    })
    .build();

// Local test: Checks admin.ctx.key == state.admin
// On-chain: Solana validates @signer constraint
```

**Key Points**:
- `@signer` account MUST sign the transaction
- Access pubkey via `account.ctx.key`
- Use `require()` to validate permissions
- Multiple signers allowed in one transaction

---

### Pattern 3: Account Creation (@init)

**Use case**: Create new blockchain accounts

```v
account VaultState {
    created_vaults: u64;
}

pub create_vault(
    payer: account @signer,
    vault: account @init,
    state: VaultState @mut
) -> pubkey {
    // @init ensures 'vault' doesn't already exist
    // payer pays for account creation

    state.created_vaults = state.created_vaults + 1;
    return vault.ctx.key;
}
```

**Test Setup**:
```typescript
const fixture = new AccountTestFixture()
    .addSignerAccount('payer')
    .addInitAccount('vault')
    .addStateAccount('state', {
        created_vaults: 0
    })
    .build();

// Local: Verifies return value is vault address
// On-chain: Creates actual account on Solana
```

**Key Points**:
- `@init` is only valid with a `@signer` payer
- Account creation requires fees (paid by signer)
- Returns new vault address for confirmation
- On-chain only: full account initialization

---

### Pattern 4: Multi-Account Transactions (@mut + @mut)

**Use case**: Atomic operations across multiple accounts

```v
account TransferState {
    transfer_count: u64;
}

pub batch_transfer(
    authority: account @signer,
    from: account @mut,
    to: account @mut,
    state: TransferState @mut,
    amount: u64
) -> bool {
    // Multiple @mut accounts in one transaction
    require(authority.ctx.key != from.ctx.key);
    require(from.ctx.key != to.ctx.key);

    // Atomic: both accounts modified together
    state.transfer_count = state.transfer_count + 1;
    return true;
}
```

**Test Setup**:
```typescript
const fixture = new AccountTestFixture()
    .addSignerAccount('authority')
    .addMutableAccount('from')
    .addMutableAccount('to')
    .addStateAccount('state', {
        transfer_count: 0
    })
    .build();

// Local: Checks account validation logic
// On-chain: Atomic account updates or rollback
```

**Key Points**:
- Multiple accounts can be modified in one call
- Accounts must be distinct (no self-transfers)
- All modifications are atomic (succeed together or fail together)
- Solana validates all constraints before execution

---

### Pattern 5: PDA (Program Derived Address)

**Use case**: Deterministic account generation

```v
account VaultState {
    vault_bump: Option<u8>;
}

pub create_user_vault(
    user: pubkey,
    seed: u64,
    vault: account @init,
    state: VaultState @mut
) -> pubkey {
    let (expected_vault, bump) = derive_pda(user, "vault", seed);
    require(vault.ctx.key == expected_vault);
    require(vault.ctx.bump == bump);

    state.vault_bump = Some(vault.ctx.bump);
    return vault.ctx.key;
}
```

**Test Setup**:
```typescript
const fixture = new AccountTestFixture()
    .addInitAccount('vault')
    .addStateAccount('state', {
        vault_bump: 255
    })
    .build();

// PDA: derive_pda() generates deterministic address
// bump: collision avoidance seed (0-255)
```

**Key Points**:
- `derive_pda()` is deterministic - same inputs = same address
- bump parameter ensures address uniqueness
- Commonly used for per-user or per-program accounts
- On-chain: account must match derived address

---

## Using Fixture Templates

The SDK includes predefined templates for common patterns:

### State Counter Template
```typescript
const fixture = FixtureTemplates.stateCounter().build();
// Creates: state account with count=0, modification_count=0
```

### Authorization Template
```typescript
const fixture = FixtureTemplates.authorization().build();
// Creates: authority signer + state account with admin and authorized_users
```

### Account Creation Template
```typescript
const fixture = FixtureTemplates.accountCreation().build();
// Creates: payer signer + init account + state with created count
```

### Batch Operation Template
```typescript
const fixture = FixtureTemplates.batchOperation().build();
// Creates: authority + 2 mutable accounts + state tracking
```

### Multi-Signature Template
```typescript
const fixture = FixtureTemplates.multiSigPattern().build();
// Creates: primary + secondary signers + state
```

### PDA Template
```typescript
const fixture = FixtureTemplates.pdaPattern().build();
// Creates: payer + vault account + state with bump seeds
```

---

## Building Custom Fixtures

### Fluent Builder API

```typescript
const fixture = new AccountTestFixture()
    // Add individual accounts
    .addSignerAccount('payer', { description: 'Pays for creation' })
    .addSignerAccount('authority')
    .addMutableAccount('data_account', { count: 0, owner: '' })
    .addStateAccount('program_state', { total_operations: 0 })
    .addReadOnlyAccount('reference')
    .addInitAccount('new_account')
    // Build all at once
    .build({ debug: true });
```

### With State Initialization

```typescript
const fixture = new AccountTestFixture()
    .addStateAccount('game_state', {
        players: 2,
        round: 0,
        winner: '11111111111111111111111111111111'
    })
    .addSignerAccount('player1')
    .addSignerAccount('player2')
    .build();
```

### Validate Against ABI

```typescript
const fixture = new AccountTestFixture()
    .addSignerAccount('authority')
    .addMutableAccount('target');

const compiled = await fixture.build();
const validation = fixture.validateAgainstABI(abiFunction);

if (!validation.valid) {
    validation.errors.forEach(err => console.error(`❌ ${err}`));
}
```

---

## Testing Workflow

### Step 1: Write Script in Five DSL (.v file)

```v
// transfer.v
account TransferLog {
    transfers: u64;
}

pub transfer(
    from: account @signer,
    to: pubkey,
    amount: u64,
    log: TransferLog @mut
) -> bool {
    require(amount > 0);
    require(amount <= 1000);

    log.transfers = log.transfers + 1;
    return true;
}
```

### Step 2: Create Fixture in Test Code

```typescript
import { AccountTestFixture, FixtureTemplates } from '@five-vm/sdk/testing';

const fixture = new AccountTestFixture()
    .addSignerAccount('from')
    .addStateAccount('log', {
        transfers: 0
    })
    .build({ debug: true });
```

### Step 3: Execute Locally (Quick Feedback)

```bash
# Verify function logic works
five local execute transfer.v 0 --params "[100]"
```

### Step 4: Deploy on-chain (Final Validation)

```bash
# Real account constraints enforced by Solana
./test-runner.sh --onchain --network devnet --category 04-account-system
```

### Step 5: Iterate

- ✅ Logic issue? Fix `.v` file
- ✅ Account setup wrong? Update fixture
- ✅ Constraint mismatch? Add validation
- ❌ On-chain failure? Check Solana logs

---

## Best Practices for Account Tests

### 1. One Test = One Pattern
```typescript
// ✅ Good: Clear, focused test
const fixture = FixtureTemplates.authorization().build();

// ❌ Bad: Mixing multiple patterns
const fixture = new AccountTestFixture()
    .addSignerAccount('auth1')
    .addSignerAccount('auth2')
    .addMutableAccount('data1')
    .addMutableAccount('data2')
    .addInitAccount('new1')
    .build();
```

### 2. Name Accounts Clearly
```typescript
// ✅ Clear names
.addSignerAccount('payer')
.addSignerAccount('authority')
.addMutableAccount('user_data')
.addStateAccount('program_state')

// ❌ Generic names
.addSignerAccount('signer1')
.addSignerAccount('signer2')
.addMutableAccount('account1')
.addMutableAccount('account2')
```

### 3. Initialize State with Realistic Values
```typescript
// ✅ Realistic defaults
.addStateAccount('state', {
    admin: 'SystemProgram',
    total_users: 0,
    bump: 255
})

// ❌ Placeholder values everywhere
.addStateAccount('state', {
    admin: '11111111111111111111111111111111',
    total_users: 0,
    bump: 0
})
```

### 4. Document Complex Fixtures
```typescript
// ✅ Clear documentation
const fixture = new AccountTestFixture()
    // Stores protocol configuration
    .addStateAccount('config', {
        fee_basis_points: 100,
        emergency_pause: false
    })
    // Protocol authority for config updates
    .addSignerAccount('upgrade_authority')
    // Data submitted by users
    .addMutableAccount('submission')
    .build();

// ❌ No context
const fixture = new AccountTestFixture()
    .addStateAccount('s1', { a: 100, b: false })
    .addSignerAccount('auth')
    .addMutableAccount('m1')
    .build();
```

### 5. Test Constraint Combinations
```typescript
// ✅ Test each constraint separately first
const readOnlyFixture = FixtureTemplates.stateCounter().build();
const signerFixture = FixtureTemplates.authorization().build();

// Then test combinations
const multiSigFixture = FixtureTemplates.multiSigPattern().build();

// ❌ Try all combinations at once
const fixture = new AccountTestFixture()
    .addSignerAccount('a').addSignerAccount('b').addSignerAccount('c')
    .addMutableAccount('m1').addMutableAccount('m2')
    .addInitAccount('i1').addInitAccount('i2')
    .build();
```

---

## Debugging Account Tests

### Enable Debug Mode

```typescript
const fixture = new AccountTestFixture()
    .addSignerAccount('payer')
    .addStateAccount('state', { count: 0 })
    .build({ debug: true });

// Output:
// [AccountTestFixture] Building fixture with 2 accounts:
//   payer (signer): 8gKXm...
//   state (state): 7aBmXq...
```

### Print Fixture Summary

```typescript
console.log(fixture.getSummary());

// Output:
// Account Test Fixture (2 accounts)
// ────────────────────────────────────
//   • payer (signer)
//   • state (state) with state: {"count":0}
// ────────────────────────────────────
// Signers: 1, Writable: 1
```

### Validate Constraints

```typescript
const validation = fixture.validateAgainstABI(abiFunc);

if (!validation.valid) {
    console.error('❌ Constraint mismatch:');
    validation.errors.forEach(e => console.error(`  - ${e}`));
}

if (validation.warnings.length > 0) {
    console.warn('⚠️  Warnings:');
    validation.warnings.forEach(w => console.warn(`  - ${w}`));
}
```

### Check Execution Context

```typescript
import { AccountTestExecutor } from '@five-vm/sdk/testing';

const context = AccountTestExecutor.bindFixture(fixture, 'transfer', [100]);
const validation = AccountTestExecutor.validateContext(context);

if (!validation.valid) {
    console.log(AccountTestExecutor.getSummary(context));
}
```

---

## Common Issues & Solutions

### Issue: "Account count mismatch"
**Problem**: Fixture accounts don't match function parameters
```v
pub process(
    authority: account @signer,
    data: account @mut,
    state: account @mut
)
```
**Solution**: Fixture must have 3 accounts
```typescript
const fixture = new AccountTestFixture()
    .addSignerAccount('authority')       // 1st
    .addMutableAccount('data')           // 2nd
    .addStateAccount('state', { ... })   // 3rd
    .build();
```

### Issue: "Signer account missing keypair"
**Problem**: Signer account generated without keypair
```typescript
const fixture = new AccountTestFixture()
    .addSignerAccount('payer')
    .build();
// ❌ payer doesn't have keypair
```
**Solution**: `addSignerAccount` automatically generates keypairs
```typescript
// ✅ Keypair generated automatically
const fixture = new AccountTestFixture()
    .addSignerAccount('payer')
    .build();
```

### Issue: "require() check failed"
**Problem**: Account state doesn't satisfy validation
```v
pub transfer(authority: account @signer, ...) {
    require(authority.ctx.key == state.admin);  // Fails
}
```
**Solution**: Set state with correct pubkey
```typescript
const fixture = new AccountTestFixture()
    .addSignerAccount('authority')  // Generates pubkey
    .addStateAccount('state', {
        admin: '<authority-pubkey>'  // Match!
    })
    .build();
```

### Issue: Local test passes but on-chain fails
**Problem**: Logic works but Solana constraint fails
**Solution**:
1. Check `@signer` accounts are actually signing
2. Verify `@mut` accounts have correct permissions
3. Confirm `@init` accounts don't already exist
4. Check on-chain logs: `solana logs --follow`

---

## Advanced Patterns

### Dynamic State Based on Function

```typescript
function createFixtureForFunction(functionName: string) {
    switch (functionName) {
        case 'create_user':
            return FixtureTemplates.accountCreation().build();
        case 'authorize':
            return FixtureTemplates.authorization().build();
        case 'batch_transfer':
            return FixtureTemplates.batchOperation().build();
        default:
            return FixtureTemplates.stateCounter().build();
    }
}
```

### Fixture Composition (Combining Patterns)

```typescript
const fixture = new AccountTestFixture()
    // Authorization pattern
    .addSignerAccount('admin')
    .addStateAccount('admin_state', { admin: '<admin-key>' })
    // State mutation pattern
    .addMutableAccount('user_data', { balance: 1000 })
    // Creation pattern
    .addInitAccount('new_account')
    .build();
```

### Reusable Fixture Factory

```typescript
class GameFixtures {
    static setup() {
        return new AccountTestFixture()
            .addSignerAccount('player1')
            .addSignerAccount('player2')
            .addStateAccount('game_state', {
                p1_score: 0,
                p2_score: 0,
                round: 0
            });
    }

    static withScores(p1: u64, p2: u64) {
        return this.setup()
            .addStateAccount('game_state', {
                p1_score: p1,
                p2_score: p2,
                round: 1
            });
    }
}

// Usage
const fixture1 = GameFixtures.setup().build();
const fixture2 = GameFixtures.withScores(100, 50).build();
```

---

## Next Steps

### Learn More
- [Five DSL Language Reference](./LANGUAGE.md)
- [Account Constraints Documentation](./CONSTRAINTS.md)
- [SDK API Reference](./SDK.md)

### Example Scripts
See `/test-scripts/04-account-system/` for 13 working examples:
- `signer-constraint.v` - Basic @signer pattern
- `mut-constraint.v` - Basic @mut pattern
- `init-constraint.v` - Basic @init pattern
- `combined-constraints.v` - Multiple constraints
- And 9 more comprehensive examples

### Testing Command Reference
```bash
# Local quick test
five local execute script.v 0

# Category test
./test-runner.sh --category 04-account-system

# Verbose with debug info
./test-runner.sh --category 04-account-system --verbose

# On-chain with real accounts
./test-runner.sh --onchain --network localnet --category 04-account-system
```

---

## Summary

**Three Key Patterns for Account Testing**:

1. **Builder Pattern** (AccountTestFixture)
   - Fluent API for defining test accounts
   - Clear, readable, composable

2. **Template Pattern** (FixtureTemplates)
   - Predefined setups for common patterns
   - Start from proven configurations

3. **Validation Pattern** (Constraint checking)
   - Validate before executing
   - Catch mismatches early

**Testing Workflow**:
1. Define account in Five DSL (.v)
2. Create fixture with AccountTestFixture
3. Test locally with `five local execute`
4. Deploy on-chain with `./test-runner.sh --onchain`
5. Iterate based on results

**This pattern is reusable** for all your account-based Five scripts!
