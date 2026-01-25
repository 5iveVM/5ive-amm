# Five CPI On-Chain Integration Tests

This directory contains integration tests that verify Five CPI functionality by executing against real Solana programs on localnet or devnet.

## What These Tests Verify

1. **CPI to SPL Token Program**
   - Mint tokens via CPI
   - Burn tokens via CPI
   - Transfer tokens via CPI
   - Verify SPL Token state changes

2. **INVOKE_SIGNED with PDA Authority**
   - Execute instructions signed by PDA
   - Verify PDA derivation and seed validation
   - Test delegated authority patterns

3. **Import Verification Security**
   - Verify program IDs match expectations
   - Prevent bytecode substitution attacks
   - Validate stack contract format

4. **Instruction Data Serialization**
   - Verify Borsh encoding matches SPL Token format
   - Verify discriminator and parameter encoding
   - Validate account ordering

## Test Scenarios

### Scenario 1: SPL Token Mint via CPI

```
Five Contract → SPL Token.mint_to → Mint Authority validates → Tokens minted
```

**Setup:**
1. Create token mint
2. Create destination token account
3. Make Five contract the mint authority
4. Call mint_to via Five CPI

**Verification:**
- Destination account token balance increased
- Mint supply increased
- Transaction succeeded

### Scenario 2: SPL Token Burn via INVOKE_SIGNED

```
Five Contract (PDA) → SPL Token.burn → PDA authority validated → Tokens burned
```

**Setup:**
1. Create token mint
2. Create PDA-owned token account
3. Mint tokens to PDA account
4. Make PDA the burn authority
5. Call burn via Five INVOKE_SIGNED

**Verification:**
- Token account balance decreased
- Mint supply decreased
- Transaction succeeded

### Scenario 3: Import Verification Security

```
Bytecode with wrong program IDs → Stack contract validation → Rejected
```

**Setup:**
1. Compile contract with one program ID
2. Try to execute with different program ID in stack contract
3. Verify rejection

**Verification:**
- Import verification blocks execution
- Error message indicates program ID mismatch

## Running Tests

### Prerequisites

```bash
# Install dependencies
npm install

# Start localnet (one terminal)
solana-test-validator

# In another terminal, deploy Five VM program
cd five-solana
cargo build --release
solana program deploy target/deploy/five.so --url http://127.0.0.1:8899

# Get program ID and update tests
solana address --program target/deploy/five.so --url http://127.0.0.1:8899
```

### Run Localnet Tests

```bash
npm run test:localnet
```

This tests CPI against the actual Solana localnet with real SPL Token program.

**What it does:**
1. Compiles test contracts
2. Deploys Five contracts
3. Creates token mint and accounts
4. Executes CPI calls
5. Verifies state changes

### Run Devnet Tests

```bash
npm run test:devnet
```

**Setup for devnet:**
```bash
# Configure for devnet
solana config set -u devnet

# Ensure you have devnet SOL (airdrop or transfer)
solana airdrop 10
```

**What it does:**
1. Deploys to devnet
2. Uses actual devnet SPL Token program
3. Verifies real on-chain CPI execution
4. Validates against live state

## Test Files

### test-spl-token-mint.v
Tests minting tokens via CPI.

```five
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (mint: pubkey, to: pubkey, authority: pubkey, amount: u64);
}

pub mint_tokens(mint: account @mut, to: account @mut, authority: account @signer) {
    SPLToken.mint_to(mint, to, authority, 1000);
}
```

### test-pda-burn.v
Tests burning tokens via INVOKE_SIGNED with PDA authority.

```five
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    burn @discriminator(8) (account: pubkey, mint: pubkey, authority: pubkey, amount: u64);
}

pub burn_from_pda(account: account @mut, mint: account @mut, pda: account) {
    SPLToken.burn(account, mint, pda, 1000);
}
```

### test-localnet.mjs
Runs full integration test suite on localnet.

**Steps:**
1. Connect to localnet
2. Compile test contracts
3. Deploy Five contracts
4. Create SPL Token infrastructure
5. Execute CPI calls
6. Verify results
7. Report test results

### test-devnet.mjs
Runs full integration test suite on devnet.

**Differences from localnet:**
- Uses deployed devnet SPL Token program
- Real on-chain state (persists)
- Slower execution (network latency)
- Requires devnet SOL funds

## Implementation Details

### Account Setup

**For mint test:**
```
┌─────────────────────┐
│   Token Mint        │
│   - Supply: 0       │
│   - Authority: Five │
└─────────────────────┘
           ↓
┌─────────────────────┐
│   Destination       │
│   Token Account     │
│   - Balance: 0      │
└─────────────────────┘
```

**For burn test (PDA authority):**
```
┌─────────────────────┐
│   Token Mint        │
│   - Supply: 10000   │
└─────────────────────┘
           ↓
┌─────────────────────┐
│   PDA Token Account │
│   - Owner: PDA      │
│   - Balance: 10000  │
└─────────────────────┘
           ↓
┌─────────────────────┐
│   PDA Authority     │
│   - Program: Five   │
└─────────────────────┘
```

### Instruction Serialization Verification

Tests verify that instruction data matches SPL Token format:

```
[Discriminator (u8)]
[Account 1 (32 bytes: mint)]
[Account 2 (32 bytes: to)]
[Account 3 (32 bytes: authority)]
[Amount (u64: 1000 in little-endian)]
```

### Import Verification Testing

Tests verify that stack contract prevents:
- Wrong program IDs
- Modified interfaces
- Bytecode substitution attacks

## Troubleshooting

### "Connection failed"
Verify solana-test-validator is running:
```bash
solana cluster-version --url http://127.0.0.1:8899
```

### "Five program not found"
Deploy Five program:
```bash
cd five-solana
cargo build --release
solana program deploy target/deploy/five.so --url http://127.0.0.1:8899
```

### "Insufficient funds"
For devnet, airdrop SOL:
```bash
solana airdrop 10 --url devnet
```

### "Compilation failed"
Verify Five DSL compiler is installed:
```bash
five --version
```

### "CPI instruction failed"
Check:
1. Account ownership (token program owns account)
2. Account type (must be TokenAccount, not Mint)
3. Authority validation (proper signer)
4. Token supply (can't mint more than mint allows)

## Output Example

```
================================================================================
Five CPI Integration Test Suite - Localnet
================================================================================

[INFO] Connected to Solana (localnet)
[INFO] Five VM Program: 9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH
[INFO] SPL Token Program: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA

================================================================================
Test 1: SPL Token Mint via CPI
================================================================================

[INFO] Creating token mint...
[PASS] Token mint created: 3HmXi...
[INFO] Creating destination token account...
[PASS] Token account created: 7xYjK...
[INFO] Deploying mint_tokens contract...
[PASS] Contract deployed: 4kL8P...
[INFO] Executing mint_tokens(1000)...
[PASS] Transaction succeeded: 2xM5K...
[INFO] Verifying token balance...
[PASS] Token balance increased to 1000

[SUCCESS] Test 1 Passed

================================================================================
Test 2: SPL Token Burn via INVOKE_SIGNED
================================================================================

[INFO] Creating PDA token account...
[PASS] PDA account created: 8mN3Q...
[INFO] Minting 10000 tokens to PDA...
[PASS] Tokens minted
[INFO] Deploying burn contract...
[PASS] Contract deployed: 5pR9T...
[INFO] Executing burn(1000)...
[PASS] Transaction succeeded: 3xP7U...
[INFO] Verifying token balance...
[PASS] Token balance decreased to 9000

[SUCCESS] Test 2 Passed

================================================================================
SUMMARY
================================================================================

Total Tests: 2
Passed: 2
Failed: 0

CPI Integration Tests: ✅ PASSED
```

## Known Limitations

- Tests are sequential (not parallelized)
- Localnet setup required for local testing
- Devnet tests require SOL funds
- PDA derivation validation is manual (not automated)

## Future Improvements

1. **Parallel test execution** - Run multiple scenarios concurrently
2. **Performance benchmarking** - Measure CPI execution costs
3. **Fuzzing tests** - Random parameter combinations
4. **Return data testing** - Once return value support is added
5. **Multi-program testing** - Test CPI chains (Five → Anchor → SPL)

## Related Documentation

- **CPI Guide:** `docs/CPI_GUIDE.md`
- **Examples:** `five-templates/cpi-examples/`
- **Compiler Source:** `five-dsl-compiler/src/interface_serializer.rs`
- **VM Handler:** `five-vm-mito/src/handlers/system/invoke.rs`
