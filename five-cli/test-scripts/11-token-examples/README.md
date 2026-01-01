# Five Token Examples

This directory contains comprehensive examples of token systems built on Five DSL, demonstrating native token implementation and AMM (Automated Market Maker) integration with SPL tokens.

## Files

### 1. `five_native_token.v`

A complete fungible token system implementation in Five DSL, similar to the SPL Token program on Solana but built natively in Five.

#### Key Features

- **Mint Management**: Initialize mints with configurable decimals and authorities
- **Token Accounts**: Create and manage user token balances
- **Minting**: Controlled token creation by authorized authorities
- **Burning**: Token destruction to reduce supply
- **Transfers**: Direct token transfers between accounts
- **Delegation**: Approve other accounts to transfer on your behalf
- **Freezing**: Freeze/thaw accounts to prevent transfers
- **Authority Management**: Change mint and freeze authorities
- **Query Functions**: Read-only functions to inspect token state

#### Account Types

```five
// Mint Account - holds token metadata
account Mint {
    authority: pubkey;           // Can mint new tokens
    freeze_authority: pubkey;    // Can freeze accounts
    supply: u64;                 // Total supply
    decimals: u8;                // Decimal places
    name: string;                // Token name
}

// TokenAccount - holds user balances
account TokenAccount {
    owner: pubkey;               // Account owner
    mint: pubkey;                // Token mint
    balance: u64;                // Token balance
    is_frozen: bool;             // Frozen status
    delegated_amount: u64;       // Approved delegation amount
    delegate: pubkey;            // Delegated authority
}
```

#### Core Functions

**Initialization**
- `pub init_mint()` - Create a new token mint
- `pub init_token_account()` - Create a token account for a user

**Minting & Burning**
- `pub mint_to()` - Mint tokens (authority only)
- `pub burn()` - Burn tokens from your account

**Transfers**
- `pub transfer()` - Transfer tokens to another account
- `pub transfer_from()` - Transfer with delegation support

**Delegation**
- `pub approve()` - Approve a delegate to transfer on your behalf
- `pub revoke()` - Revoke delegation

**Freezing**
- `pub freeze_account()` - Freeze an account (freeze authority only)
- `pub thaw_account()` - Unfreeze an account

**Authority Management**
- `pub set_mint_authority()` - Change mint authority
- `pub set_freeze_authority()` - Change freeze authority

**Queries**
- `pub get_supply()` - Get total token supply
- `pub get_balance()` - Get account balance
- `pub is_frozen()` - Check if account is frozen
- `pub get_decimals()` - Get token decimals
- And more...

#### Example Usage

```v
// Initialize a token
let mint = init_mint(
    mint_account,
    payer,
    freeze_authority,
    8,                    // 8 decimals
    "MyToken"
);

// Create a token account
let token_account = init_token_account(
    token_acct,
    owner,
    mint
);

// Mint tokens
mint_to(
    mint_account,
    token_account,
    authority,
    1_000_000  // 1 million tokens with 8 decimals = 0.01 actual tokens
);

// Transfer tokens
transfer(
    from_account,
    to_account,
    owner,
    500_000
);
```

---

### 2. `five_spl_token_amm.v`

An Automated Market Maker (AMM) implementation that enables swaps between native Five tokens and SPL tokens through the constant-product formula (x*y=k).

#### Key Features

- **Constant Product AMM**: Uses x*y=k formula for fair pricing
- **Dual Token Support**: Swaps between Five tokens and SPL tokens
- **Liquidity Pools**: Users can provide liquidity and earn fees
- **LP Tokens**: Liquidity providers receive LP shares
- **SPL Token Integration**: Full CPI (Cross-Program Invocation) support
- **Fee Management**: Configurable fee structure in basis points
- **Slippage Protection**: Quote functions for safe swaps

#### Account Types

```five
// AMMPool - holds token reserves and LP information
account AMMPool {
    token_a_reserve: u64;        // Reserve of first token
    token_b_reserve: u64;        // Reserve of second token
    total_lp_shares: u64;        // Total LP shares issued
    fee_bps: u64;                // Fee in basis points (e.g., 30 = 0.3%)
    token_a_mint: pubkey;        // Token A mint (e.g., Five)
    token_b_mint: pubkey;        // Token B mint (e.g., SPL)
    pool_authority: pubkey;      // Pool manager
    lp_token_mint: pubkey;       // LP token mint
    last_k: u64;                 // Invariant for validation
}

// LPAccount - tracks liquidity provider shares
account LPAccount {
    owner: pubkey;               // LP owner
    pool: pubkey;                // Pool reference
    lp_shares: u64;              // Shares owned
}
```

#### Core Functions

**Pool Initialization**
- `pub init_pool()` - Create a new AMM pool
- `pub init_lp_account()` - Create an LP account for a provider

**Liquidity Operations**
- `pub add_liquidity()` - Add tokens and receive LP shares
- `pub remove_liquidity()` - Burn LP shares and receive tokens

**Swaps**
- `pub swap_a_to_b()` - Swap token A for token B
- `pub swap_b_to_a()` - Swap token B for token A

**SPL Token Integration (CPI)**
- `pub create_spl_mint()` - Initialize an SPL Token mint
- `pub mint_spl_tokens()` - Mint SPL tokens
- `pub transfer_spl_tokens()` - Transfer SPL tokens
- `pub burn_spl_tokens()` - Burn SPL tokens

**Price Quotes**
- `pub quote_swap_a_to_b()` - Get expected output for swap
- `pub quote_swap_b_to_a()` - Get expected output for reverse swap
- `pub quote_add_liquidity()` - Estimate LP shares for liquidity

**Pool Info**
- `pub get_reserves()` - Get current reserves
- `pub get_spot_price()` - Get current token price ratio
- `pub get_lp_balance()` - Get LP share balance
- `pub get_fee_bps()` - Get fee configuration

#### AMM Formula Explanation

**Constant Product Formula**: x * y = k

Where:
- x = reserve of token A
- y = reserve of token B
- k = invariant (product remains constant)

When swapping amount `A_in` of token A for token B:

```
Fee = A_in * fee_percentage
A_net = A_in - Fee
B_out = (y * A_net) / (x + A_net)
```

The new reserves become:
- x' = x + A_in
- y' = y - B_out

This ensures x' * y' ≥ k (invariant is maintained or increases due to fees)

#### Liquidity Provider Example

```v
// Initial liquidity provision
let lp_shares = add_liquidity(
    pool,
    lp_account,
    provider,
    1_000_000,  // 1M token A
    2_000_000,  // 2M token B
    0           // no minimum share requirement
);

// Provider now owns lp_shares and earns portion of swap fees

// Later, provider can withdraw liquidity
let (tokens_a, tokens_b) = remove_liquidity(
    pool,
    lp_account,
    provider,
    lp_shares / 2,  // Remove half the liquidity
    0,              // no minimum for token A
    0               // no minimum for token B
);
```

#### Swap Example

```v
// User wants to swap 100 tokens of type A for type B
// First, quote the expected output
let expected_b = quote_swap_a_to_b(pool, 100_000_000);

// Perform swap with slippage protection (require at least 99% of expected)
let min_output = (expected_b * 99) / 100;
let actual_b = swap_a_to_b(pool, 100_000_000, min_output);
```

#### SPL Token Integration Example

```v
// Create an SPL mint
let spl_mint = create_spl_mint(payer, mint_account, 6);

// Mint SPL tokens
mint_spl_tokens(mint_account, destination, authority, 1_000_000);

// Transfer SPL tokens
transfer_spl_tokens(source, destination, owner, 500_000);

// Use in AMM with token A being Five tokens and token B being SPL tokens
let amm_pool = init_pool(
    pool,
    authority,
    five_token_mint,
    spl_token_mint,
    lp_token_mint,
    30  // 0.3% fee (30 basis points)
);
```

---

## Compilation and Testing

### Compile Scripts

```bash
# Compile Five native token
five compile five_native_token.v

# Compile AMM with SPL token integration
five compile five_spl_token_amm.v
```

### Local Testing

```bash
# Test token script locally
five local execute five_native_token.v 0

# Test AMM script locally
five local execute five_spl_token_amm.v 0
```

### On-Chain Testing

```bash
# Deploy and test on devnet
five deploy five_native_token.v --network devnet
five execute <SCRIPT_ACCOUNT> --function 0 --params "[...]" --network devnet

# Deploy AMM on localnet
five deploy five_spl_token_amm.v --network localnet
```

---

## Architecture & Design

### Token Account Model

Both scripts follow Solana's account model:

1. **Mint Account**: Single mint per token type, holds supply and metadata
2. **Token Accounts**: One per user per token, holds balance
3. **Authority Pattern**: Separate mint and freeze authorities for granular control
4. **Delegation**: Allow accounts to approve spending up to a limit

### AMM Design

The AMM follows standard DeFi patterns:

1. **Constant Product**: x*y=k maintains fair pricing
2. **Slippage**: Fee percentage adjustable per pool
3. **LP Tokens**: Liquidity providers get shares proportional to their contribution
4. **CPI Integration**: Can call SPL Token program for cross-token operations
5. **Invariant Checking**: Pool state validates the k invariant

### Security Considerations

- **Owner Validation**: All operations verify correct ownership
- **Balance Checks**: Prevent spending more than owned
- **Frozen Accounts**: Can be frozen to prevent transfers
- **Invariant Validation**: AMM checks that k ≥ k_previous
- **Slippage Protection**: Swaps check minimum output amounts

---

## Comparison with SPL Token

| Feature | Five Token | SPL Token |
|---------|-----------|-----------|
| Bytecode Size | Ultra-lightweight | Standard |
| Compute Units | Sub-50 CU | Standard |
| Mint Authority | Native support | Native support |
| Freeze Authority | Native support | Native support |
| Delegation | Built-in | Requires approval account |
| Decimals | Configurable | Configurable |

---

## Extensions & Improvements

You can extend these scripts for:

1. **Stake Delegation**: Implement voting power delegation
2. **Flash Loans**: Add uncollateralized lending to AMM
3. **Multi-Token Routing**: Swap through multiple pools
4. **Governance Tokens**: Add voting mechanisms
5. **Yield Farming**: Combine with staking for rewards
6. **Options Trading**: Add strike price and expiry
7. **Cross-Chain Bridges**: SPL token wrapping

---

## Resources

- [Five DSL Programming Guide](../../FIVE_DSL_PROGRAMMING_GUIDE.md)
- [Solana Token Program](https://docs.rs/spl-token/)
- [Constant Product AMM Explanation](https://docs.uniswap.org/protocol/V2/concepts/core-concepts/swaps)
- [Five CLI Documentation](../../../five-cli/CLAUDE.md)

---

## Testing with Real Data

To test with real token mints and SPL tokens:

```v
// Use real Solana token addresses
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    // SPL Token program on Solana mainnet
}

// Example: Wrap USDC (real SPL token)
let usdc_mint = 0xEPjFWaLb3fqIvIv6cXvfB7mXL4UMvVaHhMPqGmPQ4UqE;  // USDC on mainnet

// Create AMM pool: Five <-> USDC
let pool = init_pool(
    pool_account,
    authority,
    five_mint,        // Native Five token
    usdc_mint,        // Real USDC
    lp_mint,
    30                // 0.3% fee
);
```

---

Generated for Five Protocol - Ultra-lightweight smart contract platform
