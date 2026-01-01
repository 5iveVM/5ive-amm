# Five Account Testing Infrastructure - Summary

**For: Builders creating account-based Five scripts**

This document summarizes the complete, reusable account testing infrastructure for Five VM that works for both local and on-chain testing.

---

## What Is This?

A **production-ready framework for testing account-system scripts in Five**, providing:

✅ **AccountTestFixture** - Fluent builder API for defining test accounts
✅ **FixtureTemplates** - Predefined setups for 6 common patterns
✅ **AccountTestExecutor** - Bind fixtures to function execution
✅ **Local & On-Chain Testing** - Same fixtures work for both modes
✅ **Comprehensive Documentation** - Full guide with 10 production examples

---

## The Problem It Solves

Previously, builders had to manually create and manage test accounts:

```typescript
// ❌ Old way: Manual, error-prone
const accounts = [
  { pubkey: 'xyz...', isSigner: true, isWritable: false },
  { pubkey: 'abc...', isSigner: false, isWritable: true },
  // Did I get the constraints right?
];

// Hard to test account initialization
// No validation before execution
// Not reusable across tests
```

Now, builders use a **declarative, reusable pattern**:

```typescript
// ✅ New way: Clear, composable, validated
const fixture = new AccountTestFixture()
  .addSignerAccount('authority')
  .addMutableAccount('data', { count: 0 })
  .addStateAccount('state', { admin: '...' })
  .build({ debug: true });  // Automatic validation
```

---

## Three Core Components

### 1. AccountTestFixture (Builder Pattern)

**Location**: `src/sdk/testing/AccountTestFixture.ts`

**What it does**: Fluent API for constructing test accounts

```typescript
const fixture = new AccountTestFixture()
  // Add individual accounts
  .addSignerAccount('authority')
  .addMutableAccount('data', { field: 0 })
  .addStateAccount('state', { admin: 'key' })
  .addReadOnlyAccount('reference')
  .addInitAccount('new_account')

  // Build and validate
  .build({ debug: true });

// Access built accounts
fixture.accountsByName.get('authority')  // Get specific account
fixture.accounts  // Array of all accounts
fixture.metadata  // Signer count, writable count, etc.
```

**Key Methods**:
- `addSignerAccount()` - @signer constraint
- `addMutableAccount()` - @mut constraint
- `addStateAccount()` - Program state
- `addReadOnlyAccount()` - Read-only reference
- `addInitAccount()` - Account creation pattern
- `addPattern()` - Quick preset patterns
- `validateAgainstABI()` - Validate vs function requirements
- `getSummary()` - Human-readable summary

**Export**: `src/sdk/testing/index.ts`

---

### 2. FixtureTemplates (Template Pattern)

**Location**: `src/sdk/testing/AccountTestFixture.ts` (FixtureTemplates class)

**What it does**: Predefined account setups for common patterns

```typescript
// 6 built-in templates
const c1 = FixtureTemplates.stateCounter().build();        // Simple counter
const c2 = FixtureTemplates.authorization().build();       // @signer auth
const c3 = FixtureTemplates.accountCreation().build();     // @init pattern
const c4 = FixtureTemplates.batchOperation().build();      // Multiple @mut
const c5 = FixtureTemplates.multiSigPattern().build();     // Multi-signer
const c6 = FixtureTemplates.pdaPattern().build();          // PDA derivation
```

**When to use**:
- Quick setup for standard patterns
- Learning by example
- Basis for custom extensions

**Easy to customize**:
```typescript
const custom = FixtureTemplates.authorization()
  .addMutableAccount('extra_data')  // Add to template
  .build();
```

---

### 3. AccountTestExecutor (Execution Pattern)

**Location**: `src/sdk/testing/AccountTestFixture.ts` (AccountTestExecutor class)

**What it does**: Bind fixtures to function execution and validate

```typescript
// Create fixture
const fixture = await new AccountTestFixture()
  .addSignerAccount('caller')
  .addStateAccount('state', { count: 0 })
  .build();

// Bind to function with parameters
const context = AccountTestExecutor.bindFixture(
  fixture,
  'increment',           // function name
  [100]                  // parameters
);

// Validate before execution
const validation = AccountTestExecutor.validateContext(context);
if (!validation.valid) {
  validation.errors.forEach(e => console.error(`❌ ${e}`));
}

// Show execution details
console.log(AccountTestExecutor.getSummary(context));
```

**Key Methods**:
- `bindFixture()` - Attach accounts to function call
- `validateContext()` - Check execution safety
- `getSummary()` - Human-readable execution plan

---

## Usage Patterns

### Pattern 1: Simple Setup

```typescript
// 1. Define accounts
const fixture = await new AccountTestFixture()
  .addStateAccount('counter', { count: 0 })
  .build();

// 2. Execute function
const result = await executeFunction(fixture, 'increment', []);

// 3. Verify result
console.log(`Counter incremented to: ${result}`);
```

### Pattern 2: Validation Before Execution

```typescript
const fixture = await new AccountTestFixture()
  .addSignerAccount('admin')
  .addStateAccount('config', { admin: '<admin-key>' })
  .build();

// Validate constraints match function ABI
const validation = fixture.validateAgainstABI(abiFunc);
if (!validation.valid) throw new Error(validation.errors[0]);

// Safe to execute
const result = executeFunction(fixture, 'admin_only_function', []);
```

### Pattern 3: Template Extension

```typescript
// Start from template
const fixture = await FixtureTemplates.authorization()
  // Customize
  .addMutableAccount('custom_data', { value: 42 })
  .build();

// Now has: authority signer + state + custom_data
```

### Pattern 4: Reusable Factory

```typescript
class MyScriptFixtures {
  static async admin() {
    return await new AccountTestFixture()
      .addSignerAccount('admin')
      .addStateAccount('config', { admin_set: false })
      .build();
  }

  static async user(userId: string) {
    return await new AccountTestFixture()
      .addSignerAccount('user')
      .addMutableAccount('user_data', {
        id: userId,
        balance: 0
      })
      .build();
  }
}

// Usage
const admin = await MyScriptFixtures.admin();
const user = await MyScriptFixtures.user('user-123');
```

---

## Testing Workflow

### Local Testing (WASM)

```bash
# 1. Use fixture to create accounts
const fixture = await new AccountTestFixture()
  .addSignerAccount('payer')
  .addStateAccount('state', { ... })
  .build();

# 2. Execute locally (instant, no blockchain)
const result = await executeLocally(fixture, 'function_name', params);

# 3. Verify result
assert(result === expected);
```

**Benefits**:
- Instant feedback (milliseconds)
- No blockchain needed
- Perfect for development iteration

### On-Chain Testing (Solana)

```bash
# Same fixture used for on-chain testing
./test-runner.sh --onchain --network localnet --category 04-account-system

# Accounts created on actual Solana
# Constraints validated by Solana runtime
# Real account fees apply
```

**Benefits**:
- Production-ready validation
- Real account constraints enforced
- Identical fixture for both modes

---

## File Structure

```
five-cli/
├── src/sdk/
│   ├── testing/
│   │   ├── AccountTestFixture.ts      (NEW) Fixture framework
│   │   ├── AccountMetaGenerator.ts    (EXISTING) Account generation
│   │   ├── index.ts                   (UPDATED) Exports
│   │   └── ...
│   ├── examples/
│   │   ├── account-testing-examples.ts (NEW) 10 production examples
│   │   └── ...
│   └── ...
│
├── docs/
│   ├── ACCOUNT_TESTING_GUIDE.md       (NEW) Complete guide
│   └── ACCOUNT_TESTING_SUMMARY.md     (NEW) This file
│
├── test-scripts/
│   ├── 04-account-system/             (13 examples)
│   │   ├── signer-constraint.v
│   │   ├── mut-constraint.v
│   │   ├── init-constraint.v
│   │   ├── combined-constraints.v
│   │   └── ... (9 more examples)
│   └── ...
```

---

## Quick Start for Builders

### 1. Install Five CLI
```bash
npm install @five-vm/cli
```

### 2. Write Five Script
```v
// my-script.v
account State { count: u64; }

pub increment(state: State @mut) -> u64 {
    state.count = state.count + 1;
    return state.count;
}
```

### 3. Create Test
```typescript
import { AccountTestFixture } from '@five-vm/sdk/testing';

const fixture = await new AccountTestFixture()
  .addStateAccount('state', { count: 0 })
  .build();

// Test locally
const result = await executeLocally(fixture, 'increment', []);
assert.equal(result, 1);
```

### 4. Test Locally
```bash
five local execute my-script.v 0
```

### 5. Test On-Chain
```bash
./test-runner.sh --onchain --network devnet --category my-category
```

---

## API Reference

### AccountTestFixture

```typescript
class AccountTestFixture {
  // Building
  addSignerAccount(name, options?): this
  addMutableAccount(name, state?, options?): this
  addReadOnlyAccount(name, state?, options?): this
  addStateAccount(name, state?, options?): this
  addInitAccount(name, options?): this
  addPattern(pattern: 'authorization' | 'state-mutation' | 'batch-operation'): this

  // Compilation
  async build(options?: { debug?: boolean }): Promise<CompiledFixture>

  // Validation
  validateAgainstABI(abiFunction): ConstraintValidationResult
  getSummary(): string
}
```

### FixtureTemplates

```typescript
class FixtureTemplates {
  static stateCounter(): AccountTestFixture
  static authorization(): AccountTestFixture
  static accountCreation(): AccountTestFixture
  static batchOperation(): AccountTestFixture
  static multiSigPattern(): AccountTestFixture
  static pdaPattern(): AccountTestFixture
}
```

### AccountTestExecutor

```typescript
class AccountTestExecutor {
  static bindFixture(
    fixture: CompiledFixture,
    functionName: string,
    parameters?: any[]
  ): AccountExecutionContext

  static validateContext(context: AccountExecutionContext): ConstraintValidationResult

  static getSummary(context: AccountExecutionContext): string
}
```

### CompiledFixture

```typescript
interface CompiledFixture {
  accounts: GeneratedAccountMeta[];
  accountsByName: Map<string, GeneratedAccountMeta>;
  stateData: Map<string, any>;
  specs: FixtureAccountSpec[];
  metadata: {
    signerCount: number;
    mutableCount: number;
    readonlyCount: number;
    stateCount: number;
  };
}
```

---

## Example: Complete Test

```typescript
import { AccountTestFixture, AccountTestExecutor } from '@five-vm/sdk/testing';

// Create fixture with authorization pattern
const fixture = await new AccountTestFixture()
  .addSignerAccount('authority')
  .addStateAccount('state', {
    admin: '<authority-pubkey>',
    operation_count: 0
  })
  .addMutableAccount('target')
  .build({ debug: true });

// Bind to execution
const context = AccountTestExecutor.bindFixture(
  fixture,
  'perform_operation',
  [42]  // amount parameter
);

// Validate
const validation = AccountTestExecutor.validateContext(context);
if (!validation.valid) {
  console.error('Execution validation failed:');
  validation.errors.forEach(e => console.error(`  - ${e}`));
  process.exit(1);
}

// Execute
console.log('\nExecution Plan:');
console.log(AccountTestExecutor.getSummary(context));

const result = await executeWithFixture(context);
console.log(`Result: ${result}`);
```

---

## Key Design Decisions

### 1. Fluent Builder API
- **Why**: Readable, chainable, self-documenting
- **Benefit**: Code is clear about what accounts are being set up

### 2. Automatic Keypair Generation
- **Why**: Signers need real keypairs for transaction signing
- **Benefit**: Builders don't need to manage key generation

### 3. Same Fixtures for Local & On-Chain
- **Why**: No inconsistency between test modes
- **Benefit**: Tests that pass locally will work on-chain (if logic is correct)

### 4. Predefined Templates
- **Why**: Common patterns are reusable across projects
- **Benefit**: New builders can learn by example

### 5. Validation Before Execution
- **Why**: Catch constraint mismatches early
- **Benefit**: Clear error messages before expensive on-chain testing

---

## Testing the Framework

The framework is tested via:

1. **Unit Tests** in `src/sdk/__tests__/`
   - AccountTestFixture creation
   - Constraint validation
   - Account generation

2. **Integration Tests** in `test-scripts/04-account-system/`
   - 13 real account-system scripts
   - Both local and on-chain execution
   - All constraint combinations

3. **Example Scripts** in `src/sdk/examples/`
   - 10 production-ready patterns
   - Runnable examples builders can copy

---

## Extensibility

Builders can extend this framework:

### Custom Templates
```typescript
class GameFixtures {
  static async playerSetup(id: string) {
    return await new AccountTestFixture()
      .addSignerAccount(`player_${id}`)
      .addMutableAccount(`player_data_${id}`, { score: 0 })
      .build();
  }
}
```

### Custom Validators
```typescript
function validateGameSetup(fixture: CompiledFixture) {
  // Custom validation logic
  if (fixture.accounts.length < 3) {
    throw new Error('Game needs at least 3 accounts');
  }
}
```

### Custom Fixtures
```typescript
const nftMintingFixture = new AccountTestFixture()
  .addSignerAccount('creator')
  .addSignerAccount('authority')
  .addInitAccount('mint')
  .addStateAccount('metadata', { ... })
  .build();
```

---

## Compatibility

- **Node.js**: 18.0.0+
- **TypeScript**: 4.5+
- **Networks**: All (localnet, devnet, mainnet)
- **Five SDK**: Latest version

---

## Related Documentation

- **ACCOUNT_TESTING_GUIDE.md** - Complete guide with 5 patterns and best practices
- **test-scripts/04-account-system/** - 13 working examples
- **src/sdk/examples/account-testing-examples.ts** - 10 production examples
- **CLAUDE.md** - Developer reference

---

## Summary

This infrastructure provides **builders with a production-ready, reusable framework for testing Five account-system scripts** that:

✅ Works for both local (WASM) and on-chain (Solana) testing
✅ Uses a clear, fluent builder API
✅ Includes 6 predefined templates for common patterns
✅ Validates constraints before execution
✅ Is fully documented with 10 production examples
✅ Is extensible for custom needs

**Result**: Builders can write robust account-based scripts with confidence, knowing their tests will work reliably in production.
