# Five Token Scripts - Quick Reference

Fast lookup guide for common operations with Five tokens and AMM swaps.

## Five Native Token (`five_native_token.v`)

### Accounts
```v
account Mint {
    authority: pubkey;
    freeze_authority: pubkey;
    supply: u64;
    decimals: u8;
    name: string;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
    is_frozen: bool;
    delegated_amount: u64;
    delegate: pubkey;
}
```

### Key Functions

| Function | Purpose | Authority |
|----------|---------|-----------|
| `init_mint()` | Create token | Any |
| `init_token_account()` | Create account | Owner |
| `mint_to()` | Create tokens | Mint authority |
| `burn()` | Destroy tokens | Account owner |
| `transfer()` | Send tokens | Account owner |
| `transfer_from()` | Delegated send | Owner or delegate |
| `approve()` | Allow spending | Account owner |
| `revoke()` | Revoke delegate | Account owner |
| `freeze_account()` | Lock account | Freeze authority |
| `thaw_account()` | Unlock account | Freeze authority |
| `set_mint_authority()` | Change mint auth | Current authority |
| `set_freeze_authority()` | Change freeze auth | Current authority |

### Query Functions

| Function | Returns |
|----------|---------|
| `get_supply()` | Total token supply |
| `get_balance()` | Account balance |
| `get_owner()` | Account owner |
| `get_mint()` | Token mint |
| `is_frozen()` | Frozen status |
| `get_decimals()` | Token decimals |
| `get_delegated_amount()` | Approved amount |
| `get_delegate()` | Delegate address |

### Common Patterns

**Mint and distribute:**
```v
let mint = init_mint(mint_acct, auth, freeze_auth, 8, "MyToken");
let acct = init_token_account(token_acct, owner, mint);
mint_to(mint, acct, auth, amount);
```

**Transfer with delegation:**
```v
approve(account, owner, delegate, amount);
transfer_from(account, dest, delegate, amount);
revoke(account, owner);
```

**Freeze protocol:**
```v
freeze_account(mint, account, freeze_auth);
// Account frozen - no transfers
thaw_account(mint, account, freeze_auth);
// Account unfrozen
```

---

## AMM Swap (`five_spl_token_amm.v`)

### Accounts

```v
account AMMPool {
    token_a_reserve: u64;
    token_b_reserve: u64;
    total_lp_shares: u64;
    fee_bps: u64;           // 30 = 0.3%
    token_a_mint: pubkey;
    token_b_mint: pubkey;
    pool_authority: pubkey;
    lp_token_mint: pubkey;
    last_k: u64;
}

account LPAccount {
    owner: pubkey;
    pool: pubkey;
    lp_shares: u64;
}
```

### Core Functions

| Function | Input | Output |
|----------|-------|--------|
| `init_pool()` | fee_bps | pool_key |
| `init_lp_account()` | - | lp_key |
| `add_liquidity()` | amount_a, amount_b | lp_shares |
| `remove_liquidity()` | lp_shares | (token_a, token_b) |
| `swap_a_to_b()` | amount_a_in | token_b_out |
| `swap_b_to_a()` | amount_b_in | token_a_out |

### SPL Integration

```v
create_spl_mint(payer, mint, decimals)
mint_spl_tokens(mint, dest, auth, amount)
transfer_spl_tokens(from, to, auth, amount)
burn_spl_tokens(mint, account, auth, amount)
```

### Quote Functions

```v
quote_swap_a_to_b(pool, amount_a) -> amount_b
quote_swap_b_to_a(pool, amount_b) -> amount_a
quote_add_liquidity(pool, amount_a, amount_b) -> lp_shares
```

### Info Functions

```v
get_reserves(pool) -> (a_reserve, b_reserve)
get_spot_price(pool) -> price_scaled
get_lp_balance(lp_account) -> shares
get_total_lp_shares(pool) -> total_shares
get_fee_bps(pool) -> fee
```

### Common Patterns

**Initialize pool:**
```v
let pool = init_pool(pool_acct, auth, mint_a, mint_b, lp_mint, 30);
let lp = init_lp_account(lp_acct, provider, pool);
```

**Provide liquidity:**
```v
let shares = add_liquidity(pool, lp, provider, amt_a, amt_b, 0);
```

**Swap with protection:**
```v
let quote = quote_swap_a_to_b(pool, amount_a);
let min_out = (quote * 98) / 100;  // 2% slippage
let actual = swap_a_to_b(pool, amount_a, min_out);
```

**Withdraw liquidity:**
```v
let (amt_a, amt_b) = remove_liquidity(pool, lp, owner, shares, 0, 0);
```

---

## AMM Math

**Constant Product Formula:**
```
x * y = k
```

**Output Amount:**
```
fee = input * fee_bps / 10000
net_input = input - fee
output = (reserve_out * net_input) / (reserve_in + net_input)
```

**LP Share Calculation (Initial):**
```
shares = sqrt(amount_a * amount_b)
```

**LP Share Calculation (Non-Initial):**
```
shares = min(
    (amount_a * total_shares) / reserve_a,
    (amount_b * total_shares) / reserve_b
)
```

**Price Impact:**
```
price_impact = output_amount / (input * (reserve_out / reserve_in))
```

---

## Error Messages

| Error | Cause | Fix |
|-------|-------|-----|
| Insufficient balance | Not enough tokens | Transfer less |
| Account is frozen | Account locked | Request thaw |
| Authority mismatch | Wrong signer | Use correct authority |
| Insufficient output | Too much slippage | Increase slippage tolerance |
| Mint mismatch | Wrong token | Use correct mint |
| Invariant violation | Formula broken | Check pool state |
| Pool depleted | No reserves left | Add liquidity |

---

## Decimal Considerations

Remember to account for decimals when specifying amounts:

```
Actual tokens = Raw amount / 10^decimals

8 decimals (like USDC):  1,000,000 raw = 0.01 actual
6 decimals (like native SOL): 1,000,000 raw = 1.0 actual
```

### Common Decimals
- USDC: 6
- Wrapped SOL: 9
- Most SPL tokens: 6 or 8
- Five tokens: configurable (typically 8 or 9)

---

## Fee Structure

**Basis Points (BPS) = 1/100th of a percent**

| BPS | Percentage | Example |
|-----|-----------|---------|
| 10 | 0.10% | 10M in = 10k fee |
| 25 | 0.25% | 10M in = 25k fee |
| 30 | 0.30% | 10M in = 30k fee |
| 50 | 0.50% | 10M in = 50k fee |
| 100 | 1.00% | 10M in = 100k fee |

---

## Typical Workflows

### User swaps 1,000 Five → USDC

```v
// 1. Get quote
let expected = quote_swap_a_to_b(pool, 100_000_000);

// 2. Set slippage (2%)
let min_out = (expected * 98) / 100;

// 3. Execute swap
let usdc_received = swap_a_to_b(pool, 100_000_000, min_out);
```

### LP earns fees

```v
// 1. Add liquidity
let shares = add_liquidity(pool, lp, owner, amt_a, amt_b, min_shares);

// 2. Users trade (LP earns fees)
// ... swaps happen ...

// 3. Withdraw (receive more due to fees)
let (amt_a_out, amt_b_out) = remove_liquidity(pool, lp, owner, shares, 0, 0);
// amt_a_out > amt_a_in (earned fees!)
```

### Create new token

```v
// 1. Initialize mint
let mint = init_mint(mint_acct, auth, freeze_auth, 8, "NewToken");

// 2. Create accounts
let alice = init_token_account(acct1, alice_key, mint);
let bob = init_token_account(acct2, bob_key, mint);

// 3. Mint initial supply
mint_to(mint, alice, auth, 1_000_000_000);  // 10M tokens

// 4. Users can transfer
transfer(alice, bob, alice_key, 500_000_000);
```

---

## Security Checklist

- [ ] Validate all pubkey inputs
- [ ] Check account ownership for signers
- [ ] Verify authority before privileged operations
- [ ] Set reasonable slippage tolerance (0.5-5%)
- [ ] Check pool invariant after swaps
- [ ] Use minimum amount requirements
- [ ] Verify no account freezing
- [ ] Validate token mints match

---

## Compilation

```bash
# Compile token script
five compile five_native_token.v

# Compile AMM script
five compile five_spl_token_amm.v
```

## Local Testing

```bash
# Execute token script locally
five local execute five_native_token.v 0

# Execute AMM script locally
five local execute five_spl_token_amm.v 0
```

## On-Chain Deployment

```bash
# Deploy token to devnet
five deploy five_native_token.v --network devnet

# Deploy AMM to localnet
five deploy five_spl_token_amm.v --network localnet

# Execute on-chain
five execute <SCRIPT_ACCOUNT> --function 0 --network devnet
```

---

## Links

- Full [README.md](README.md) - Complete feature documentation
- [USAGE_EXAMPLES.md](USAGE_EXAMPLES.md) - Real-world scenarios
- [Five DSL Guide](../../FIVE_DSL_PROGRAMMING_GUIDE.md)
- [Five CLI Reference](../../../five-cli/CLAUDE.md)

---

**Last Updated:** 2025-12-11
**Five Protocol Version:** Latest
