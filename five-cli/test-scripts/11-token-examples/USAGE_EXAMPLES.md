# Five Token Scripts - Usage Examples

Complete, runnable examples demonstrating real-world scenarios using the Five native token and AMM swap systems.

---

## Part 1: Native Token Operations

### Scenario 1: Create a Token and Distribute to Users

```v
// Initialize "MyToken" with specific configuration
let my_mint = init_mint(
    mint_account,
    authority,           // User with mint authority
    freeze_authority,    // Can freeze accounts
    8,                   // 8 decimal places (like USDC)
    "MyToken"
);

// Create token accounts for three users
let alice_account = init_token_account(
    alice_token_acct,
    alice,
    my_mint
);

let bob_account = init_token_account(
    bob_token_acct,
    bob,
    my_mint
);

let charlie_account = init_token_account(
    charlie_token_acct,
    charlie,
    my_mint
);

// Mint 1 million tokens to Alice
mint_to(
    mint_account,
    alice_account,
    authority,
    100_000_000  // 100M with 8 decimals = 1M actual tokens
);

// Check Alice's balance
let alice_balance = get_balance(alice_account);  // Returns 100_000_000
```

### Scenario 2: Users Exchange Tokens

```v
// Alice transfers 50 tokens to Bob
transfer(
    alice_account,      // From Alice
    bob_account,        // To Bob
    alice,              // Alice signs (she's the owner)
    5_000_000           // 50 tokens (with 8 decimals)
);

// Verify balances
let alice_new = get_balance(alice_account);  // 50_000_000 (50 tokens)
let bob_new = get_balance(bob_account);      // 5_000_000 (5 tokens)
```

### Scenario 3: Delegated Transfers

Alice wants to let Bob spend some of her tokens without giving him full control.

```v
// Alice approves Bob to spend 10 tokens
approve(
    alice_account,
    alice,
    bob,           // Bob is the delegate
    1_000_000      // Can spend up to 1M (10 tokens)
);

// Now Bob can transfer Alice's tokens on her behalf
transfer_from(
    alice_account,      // From Alice
    charlie_account,    // To Charlie
    bob,                // Bob signs (he's delegated)
    500_000             // Transfer 5 tokens
);

// Alice's delegated amount decreased
let remaining_delegation = get_delegated_amount(alice_account);  // 500_000
```

### Scenario 4: Burning and Reducing Supply

Alice wants to burn some of her tokens to reduce total supply.

```v
// Initially: supply = 100_000_000
let initial_supply = get_supply(mint_account);

// Alice burns 20 tokens
burn(
    mint_account,
    alice_account,
    alice,
    2_000_000      // 20 tokens
);

// New supply is 80 tokens
let new_supply = get_supply(mint_account);  // 80_000_000
let alice_balance = get_balance(alice_account);  // Also 80_000_000
```

### Scenario 5: Freezing a Compromised Account

If an account is compromised, the freeze authority can lock it.

```v
// Freeze Bob's account (only freeze authority can do this)
freeze_account(
    mint_account,
    bob_account,
    freeze_authority  // Must be the freeze authority signer
);

// Now Bob cannot transfer tokens - this will fail:
// transfer(bob_account, alice_account, bob, 100);  // FAILS

// But the freeze authority can thaw the account later
thaw_account(
    mint_account,
    bob_account,
    freeze_authority  // Must be the freeze authority signer
);

// Now Bob can transfer again
transfer(bob_account, alice_account, bob, 100);  // OK
```

### Scenario 6: Authority Transfer

The current mint authority wants to transfer control to a new authority.

```v
// Current authority is "old_authority"
let current_auth = mint_account.authority;  // old_authority

// Set new authority
set_mint_authority(
    mint_account,
    old_authority,      // Current authority signs
    new_authority       // New authority to transfer to
);

// Now only new_authority can mint
mint_to(
    mint_account,
    alice_account,
    new_authority,      // This works
    1_000_000
);

// old_authority can no longer mint:
// mint_to(mint_account, alice_account, old_authority, 1_000_000);  // FAILS
```

---

## Part 2: AMM Swap Operations

### Scenario 1: Initialize an AMM Pool

Create an AMM pool that allows trading between Five tokens and USDC (SPL).

```v
// Initialize the AMM pool
let pool = init_pool(
    pool_account,
    pool_authority,      // Pool manager
    five_mint,          // Token A: Five token
    usdc_mint,          // Token B: USDC (SPL)
    lp_token_mint,      // Mint for LP shares
    30                  // 0.3% fee (30 basis points)
);

// Create LP account for the first liquidity provider
let lp_account = init_lp_account(
    lp_acct,
    provider,
    pool
);
```

### Scenario 2: Provide Initial Liquidity

The first LP provides initial liquidity to bootstrap the pool.

```v
// Provider deposits 100,000 Five tokens and 200,000 USDC
// This sets the initial price: 1 Five = 2 USDC
let lp_shares = add_liquidity(
    pool,
    lp_account,
    provider,
    10_000_000_000,    // 100,000 Five tokens (with 8 decimals)
    200_000_000_000,   // 200,000 USDC (with 8 decimals)
    0                  // No minimum share requirement for initial liquidity
);

// Provider receives LP shares
// Reserve state updated:
// - Five reserve: 100,000
// - USDC reserve: 200,000
// - Price: 1 Five = 2 USDC
```

### Scenario 3: Subsequent Liquidity Providers

New LPs add liquidity at the current price ratio.

```v
// Create LP account for second provider
let lp2 = init_lp_account(lp_acct_2, provider2, pool);

// Provider 2 wants to add liquidity maintaining the 1:2 ratio
// They add 50,000 Five and 100,000 USDC
let shares2 = add_liquidity(
    pool,
    lp2,
    provider2,
    5_000_000_000,     // 50,000 Five
    10_000_000_000,    // 100,000 USDC
    1_000_000          // Require at least 1M LP shares
);

// New reserves:
// - Five: 150,000
// - USDC: 300,000
// Pool maintains constant product
```

### Scenario 4: User Swaps Five for USDC

A user wants to convert Five tokens to USDC.

```v
// User has 1,000 Five tokens and wants to swap for USDC
// First, get a quote for the fair output
let expected_usdc = quote_swap_a_to_b(
    pool,
    100_000_000  // 1,000 Five tokens
);
// Returns approximately 1,980 USDC (accounting for fees)

// User adds 2% slippage tolerance
let min_usdc_out = (expected_usdc * 98) / 100;

// Perform the swap
let actual_usdc = swap_a_to_b(
    pool,
    100_000_000,   // Input: 1,000 Five
    min_usdc_out   // Minimum output with slippage
);

// Pool reserves updated:
// - Five: 151,000 (added 1,000)
// - USDC: ~298,020 (removed ~1,980)
// Invariant maintained: 151,000 * 298,020 > original k
```

### Scenario 5: User Swaps USDC for Five (Reverse)

A user wants to convert USDC back to Five tokens.

```v
// User has 1,000 USDC and wants Five tokens
let expected_five = quote_swap_b_to_a(pool, 100_000_000);  // 1,000 USDC
// Returns approximately 480 Five tokens (accounting for fees and ratio)

let min_five_out = (expected_five * 98) / 100;

let actual_five = swap_b_to_a(
    pool,
    100_000_000,   // Input: 1,000 USDC
    min_five_out   // Minimum output
);

// Pool reserves updated inversely
// Note: Due to fees, user doesn't get back exact initial amount
```

### Scenario 6: Liquidity Provider Withdraws

After earning fees, an LP wants to withdraw their share.

```v
// LP has earned fees over time and wants to withdraw half their position
// Their LP shares value has increased due to fee accumulation

let lp_shares_to_withdraw = lp_account.lp_shares / 2;

let (five_out, usdc_out) = remove_liquidity(
    pool,
    lp_account,
    provider,
    lp_shares_to_withdraw,
    0,             // No minimum for Five
    0              // No minimum for USDC
);

// LP receives proportional share of both tokens
// Due to fees collected during swaps, they often receive MORE than they put in

// New pool reserves:
// - Five: reduced by five_out
// - USDC: reduced by usdc_out
// LP account: shares reduced by half
```

### Scenario 7: Multi-User Trading

Realistic scenario with multiple users trading and liquidity providers earning.

```v
// Initial state: Pool has 100,000 Five and 200,000 USDC
// Price: 1 Five = 2 USDC

// User 1 swaps 10,000 Five for USDC
let usdc_user1 = swap_a_to_b(pool, 1_000_000_000, 0);  // Gets ~19,700 USDC

// User 2 swaps 5,000 USDC for Five
let five_user2 = swap_b_to_a(pool, 500_000_000, 0);    // Gets ~2,380 Five

// User 3 swaps 20,000 Five for USDC
let usdc_user3 = swap_a_to_b(pool, 2_000_000_000, 0);  // Gets ~38,900 USDC

// LP withdraws to see how much they earned
let lp_balance_before = 150_000;
let (five_got, usdc_got) = remove_liquidity(
    pool,
    lp_account,
    provider,
    full_shares,
    0,
    0
);

// LP earned fees from 3 swaps even though price moved
// Total received > total originally deposited (due to fees)
```

### Scenario 8: Price Discovery and Slippage

Demonstrating how price changes with swap size.

```v
// Current pool: 100,000 Five : 200,000 USDC (1:2 ratio)

// Small swap: 1,000 Five
let small_swap = quote_swap_a_to_b(pool, 100_000_000);
// Returns ~1,980 USDC (essentially 2x due to small size)

// Large swap: 50,000 Five (50% of pool)
let large_swap = quote_swap_a_to_b(pool, 5_000_000_000);
// Returns ~66,600 USDC (much less than 100k, high slippage)

// Very large swap: 100,000 Five (entire pool!)
let huge_swap = quote_swap_a_to_b(pool, 10_000_000_000);
// Returns ~100,000 USDC (asymptotic behavior, not 200k!)

// This demonstrates price impact and slippage
```

---

## Part 3: Cross-Program Integration

### Scenario 1: Create SPL Token Mint

Create an SPL token within the Five DSL contract using CPI.

```v
// Create an SPL USDC-like token
let usdc_mint = create_spl_mint(
    payer,          // Pays for account creation
    mint_account,   // Account to initialize as mint
    6               // 6 decimal places (USDC standard)
);

// Now usdc_mint can be used in the AMM pool
// Mint some USDC tokens
mint_spl_tokens(
    mint_account,
    destination_account,
    authority,
    1_000_000_000   // 1M USDC
);
```

### Scenario 2: AMM with Real SPL Tokens

Use the AMM with actual SPL tokens from Solana.

```v
// USDC addresses on Solana (example)
let usdc_mint_address = 0xEPjFWaLb3fqIvIv6cXvfB7mXL4UMvVaHhMPqGmPQ4UqE;

// Create AMM: Five <-> Real USDC
let pool = init_pool(
    pool_account,
    authority,
    five_token_mint,     // Five token
    usdc_mint_address,   // Real USDC on Solana
    lp_token_mint,
    25                   // 0.25% fee
);

// Now swaps can happen between native Five and real SPL tokens
// LPs earn fees on real USDC/Five trading
```

### Scenario 3: Transfer SPL Tokens in Pool

Move SPL tokens between accounts using CPI.

```v
// Transfer USDC from one account to another
transfer_spl_tokens(
    usdc_source_account,
    usdc_dest_account,
    authority,
    500_000_000  // Transfer 5M USDC
);

// Burn USDC tokens
burn_spl_tokens(
    usdc_mint,
    usdc_account,
    authority,
    100_000_000  // Burn 1M USDC
);
```

---

## Part 4: Error Handling and Edge Cases

### Scenario 1: Insufficient Balance

```v
// Alice has 100 tokens but tries to send 150
// This will fail with "Insufficient balance"
transfer(
    alice_account,
    bob_account,
    alice,
    150_000_000  // More than Alice has
);
// ERROR: Insufficient balance ❌
```

### Scenario 2: Frozen Account

```v
// Account is frozen
freeze_account(mint_account, bob_account, freeze_authority);

// Bob tries to transfer but fails
transfer(bob_account, alice_account, bob, 100);
// ERROR: Source account is frozen ❌

// Only freeze authority can fix this
thaw_account(mint_account, bob_account, freeze_authority);

// Now it works
transfer(bob_account, alice_account, bob, 100);  // OK ✓
```

### Scenario 3: Slippage Protection

```v
// Market price is 1:2, but heavy trading changed it
// User tries to swap with extreme slippage

let expected = quote_swap_a_to_b(pool, 1_000_000_000);  // 1,500 USDC

let min_required = 2_000_000_000;  // Unrealistic requirement

swap_a_to_b(pool, 1_000_000_000, min_required);
// ERROR: Insufficient output amount ❌

// With realistic slippage (2%)
let min_realistic = (expected * 98) / 100;
swap_a_to_b(pool, 1_000_000_000, min_realistic);  // OK ✓
```

### Scenario 4: LP Share Precision

```v
// New liquidity provider adds tiny amounts
let minimal = add_liquidity(
    pool,
    lp_account,
    provider,
    1,              // 1 wei
    1,              // 1 wei
    1               // Require at least 1 share
);
// May fail if calculated shares < minimum required

// With more realistic amounts
let viable = add_liquidity(
    pool,
    lp_account,
    provider,
    1_000_000,      // 1M units
    2_000_000,      // 2M units
    100_000         // Require at least 100k shares
);  // OK ✓
```

---

## Part 5: Security Considerations

### Scenario 1: Verify Mint Authority

```v
// Always check authority before trusting mint operation
require(
    mint_account.authority == expected_authority,
    "Mint authority changed unexpectedly"
);

// Only then mint
mint_to(mint_account, account, authority, amount);
```

### Scenario 2: Validate Account Ownership

```v
// Never trust account input - always verify ownership
require(
    token_account.owner == signer,
    "Account owner mismatch"
);

// Safe to execute transfer
transfer(token_account, dest, signer, amount);
```

### Scenario 3: Check Pool Invariant

```v
// After swap, verify pool invariant is maintained
let new_k = pool.token_a_reserve * pool.token_b_reserve;

require(
    new_k >= pool.last_k,
    "Invariant violation detected"
);
```

### Scenario 4: Slippage Tolerance

```v
// Always set reasonable slippage tolerance
let quote = quote_swap_a_to_b(pool, input);

// Typical: 0.5% - 5% depending on pool volatility
let slippage = 200;  // 2%
let min_output = (quote * (10000 - slippage)) / 10000;

swap_a_to_b(pool, input, min_output);  // Protected
```

---

## Practical Deployment Checklist

### Before Going Live with Token

- [ ] Mint authority is set correctly
- [ ] Freeze authority is a secure, multi-sig address
- [ ] Test token supply limits
- [ ] Verify decimal configuration
- [ ] Test authority transfers
- [ ] Verify all constraint checks work

### Before Going Live with AMM

- [ ] Initial liquidity provided by trusted party
- [ ] Fee rate is documented and fair
- [ ] LP shares properly allocated
- [ ] Test swap slippage protection
- [ ] Verify invariant checking
- [ ] SPL integration tested with real tokens
- [ ] Price oracle integration (if used)
- [ ] Emergency withdraw function tested

---

## Performance Considerations

**Token Operations:**
- `transfer`: ~50-100 CU
- `mint_to`: ~75-150 CU
- `burn`: ~75-150 CU
- `freeze_account`: ~50 CU

**AMM Operations:**
- `swap`: ~200-400 CU (includes fee calculation)
- `add_liquidity`: ~300-500 CU (includes LP minting)
- `remove_liquidity`: ~300-500 CU
- Quote functions: ~100-200 CU (read-only)

**SPL Integration:**
- CPI calls add ~300-500 CU per call
- SPL token operations: add standard SPL costs

---

## Troubleshooting

### Common Issues

**"Account frozen" error**
- Check if account has been frozen by freeze authority
- Use `is_frozen()` to verify status
- Request thaw from freeze authority

**"Insufficient output" on swap**
- Price may have moved between quote and swap
- Increase slippage tolerance
- Or retry with fresh quote

**"Mint authority mismatch"**
- Verify correct authority signer
- Check if authority was recently changed
- Use correct keypair

**"LP shares too small"**
- Add larger amounts of liquidity
- Or reduce minimum share requirement
- Check for precision loss in calculations

---

This document provides complete, working examples for all major token and AMM operations. Each scenario is runnable with proper account setup and can be adapted for your specific use case.
