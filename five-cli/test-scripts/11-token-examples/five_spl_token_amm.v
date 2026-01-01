// Five DSL - AMM Swap between Five Native Token and SPL Token
// Implements a Constant Product AMM (x*y=k) with support for both Five native tokens
// and SPL tokens via CPI calls. This enables native token holders to swap with SPL tokens.
// @test-params

// ============================================================================
// EXTERNAL INTERFACES
// ============================================================================

// SPL Token interface for CPI (Cross-Program Invocation) calls
// This allows our contract to call the official SPL Token program
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    // Initialize a new mint
    initialize_mint @discriminator(0) (
        mint: pubkey,
        decimals: u8,
        authority: pubkey,
        freeze_authority: pubkey
    );

    // Mint tokens to a destination account
    mint_to @discriminator(7) (
        mint: pubkey,
        to: pubkey,
        authority: pubkey,
        amount: u64
    );

    // Transfer tokens between accounts
    transfer @discriminator(3) (
        source: pubkey,
        destination: pubkey,
        authority: pubkey,
        amount: u64
    );

    // Burn tokens from an account
    burn @discriminator(8) (
        mint: pubkey,
        account: pubkey,
        authority: pubkey,
        amount: u64
    );
}

// ============================================================================
// ACCOUNT DEFINITIONS
// ============================================================================

// AMM Pool state - holds both token reserves and liquidity provider shares
account AMMPool {
    // Token reserves
    token_a_reserve: u64;        // Reserve of first token (typically Five)
    token_b_reserve: u64;        // Reserve of second token (typically SPL)

    // Liquidity provider tracking
    total_lp_shares: u64;        // Total LP (Liquidity Provider) shares issued

    // Fee configuration
    fee_bps: u64;                // Fee in basis points (e.g., 30 = 0.3%)

    // Token mint addresses
    token_a_mint: pubkey;        // Mint address for token A (Five)
    token_b_mint: pubkey;        // Mint address for token B (SPL)

    // Authority
    pool_authority: pubkey;      // Authority that can manage the pool

    // LP Token mint
    lp_token_mint: pubkey;       // Mint address for LP shares

    // Invariant tracking for validation
    last_k: u64;                 // Last k value (x * y) for slippage checking
}

// Liquidity Provider account - tracks LP shares for individual providers
account LPAccount {
    owner: pubkey;               // Owner of the LP shares
    pool: pubkey;                // Reference to the pool
    lp_shares: u64;              // Number of LP shares owned
}

// ============================================================================
// POOL INITIALIZATION
// ============================================================================

// Initialize a new AMM pool with two tokens
pub init_pool(
    pool: AMMPool @mut @init,
    authority: account @signer,
    token_a_mint: pubkey,
    token_b_mint: pubkey,
    lp_token_mint: pubkey,
    fee_bps: u64
) -> pubkey {
    // Verify fee is reasonable (max 10% = 1000 bps)
    require(fee_bps <= 1000, "Fee too high");

    pool.token_a_reserve = 0;
    pool.token_b_reserve = 0;
    pool.total_lp_shares = 0;
    pool.fee_bps = fee_bps;
    pool.token_a_mint = token_a_mint;
    pool.token_b_mint = token_b_mint;
    pool.pool_authority = authority.key;
    pool.lp_token_mint = lp_token_mint;
    pool.last_k = 0;

    return get_key(pool);
}

// Initialize an LP account for tracking LP shares
pub init_lp_account(
    lp_account: LPAccount @mut @init,
    owner: account @signer,
    pool: pubkey
) -> pubkey {
    lp_account.owner = owner.key;
    lp_account.pool = pool;
    lp_account.lp_shares = 0;

    return get_key(lp_account);
}

// ============================================================================
// LIQUIDITY OPERATIONS
// ============================================================================

// Add liquidity to the pool - deposit both tokens and receive LP shares
// This function accepts equal value amounts of both tokens
pub add_liquidity(
    pool: AMMPool @mut,
    lp_account: LPAccount @mut,
    owner: account @signer,
    amount_a: u64,
    amount_b: u64,
    min_lp_shares: u64
) -> u64 {
    // Verify amounts are positive
    require(amount_a > 0, "Amount A must be greater than zero");
    require(amount_b > 0, "Amount B must be greater than zero");

    // Verify LP account ownership
    require(lp_account.owner == owner.key, "LP account owner mismatch");

    // Verify LP account belongs to this pool
    require(lp_account.pool == get_key(pool), "LP account pool mismatch");

    let lp_shares: u64;

    // Calculate LP shares based on whether this is initial liquidity
    if (pool.total_lp_shares == 0) {
        // Initial liquidity: LP shares = sqrt(amount_a * amount_b)
        // For simplicity, use geometric mean approach: sqrt(x*y)
        lp_shares = sqrt_product(amount_a, amount_b);

        // Verify minimum shares for initial liquidity
        require(lp_shares >= min_lp_shares, "Insufficient LP shares minted");
    } else {
        // Subsequent liquidity: maintain proportional share
        // New shares = (amount_a / reserve_a) * existing_shares
        // or equivalently = (amount_b / reserve_b) * existing_shares
        let shares_from_a = (amount_a * pool.total_lp_shares) / pool.token_a_reserve;
        let shares_from_b = (amount_b * pool.total_lp_shares) / pool.token_b_reserve;

        // Use the minimum to maintain constant product
        if (shares_from_a < shares_from_b) {
            lp_shares = shares_from_a;
        } else {
            lp_shares = shares_from_b;
        }

        // Verify minimum shares
        require(lp_shares >= min_lp_shares, "Insufficient LP shares minted");
    }

    // Update pool reserves
    pool.token_a_reserve = pool.token_a_reserve + amount_a;
    pool.token_b_reserve = pool.token_b_reserve + amount_b;

    // Update total LP shares
    pool.total_lp_shares = pool.total_lp_shares + lp_shares;

    // Update invariant
    pool.last_k = pool.token_a_reserve * pool.token_b_reserve;

    // Assign LP shares to the provider
    lp_account.lp_shares = lp_account.lp_shares + lp_shares;

    return lp_shares;
}

// Remove liquidity from the pool - burn LP shares and receive both tokens back
pub remove_liquidity(
    pool: AMMPool @mut,
    lp_account: LPAccount @mut,
    owner: account @signer,
    lp_shares: u64,
    min_amount_a: u64,
    min_amount_b: u64
) -> (u64, u64) {
    // Verify shares are positive
    require(lp_shares > 0, "LP shares must be greater than zero");

    // Verify ownership
    require(lp_account.owner == owner.key, "LP account owner mismatch");

    // Verify sufficient shares
    require(lp_account.lp_shares >= lp_shares, "Insufficient LP shares");

    // Calculate tokens returned based on proportional share
    let amount_a = (lp_shares * pool.token_a_reserve) / pool.total_lp_shares;
    let amount_b = (lp_shares * pool.token_b_reserve) / pool.total_lp_shares;

    // Verify minimum amounts
    require(amount_a >= min_amount_a, "Insufficient token A output");
    require(amount_b >= min_amount_b, "Insufficient token B output");

    // Update pool state
    pool.token_a_reserve = pool.token_a_reserve - amount_a;
    pool.token_b_reserve = pool.token_b_reserve - amount_b;
    pool.total_lp_shares = pool.total_lp_shares - lp_shares;

    // Update invariant
    pool.last_k = pool.token_a_reserve * pool.token_b_reserve;

    // Burn LP shares
    lp_account.lp_shares = lp_account.lp_shares - lp_shares;

    return (amount_a, amount_b);
}

// ============================================================================
// SWAP OPERATIONS
// ============================================================================

// Swap token A for token B using the constant product formula (x*y=k)
pub swap_a_to_b(
    pool: AMMPool @mut,
    amount_a_in: u64,
    min_b_out: u64
) -> u64 {
    // Verify input is positive
    require(amount_a_in > 0, "Input amount must be greater than zero");

    // Verify sufficient reserves
    require(pool.token_a_reserve > 0, "Pool has no token A reserve");
    require(pool.token_b_reserve > 0, "Pool has no token B reserve");

    // Calculate fee
    let fee_amount = (amount_a_in * pool.fee_bps) / 10000;
    let amount_a_net = amount_a_in - fee_amount;

    // Apply constant product formula: x*y = k
    // Output = (y * x_in) / (x + x_in)
    let denominator = pool.token_a_reserve + amount_a_net;
    let amount_b_out = (pool.token_b_reserve * amount_a_net) / denominator;

    // Verify output meets minimum
    require(amount_b_out >= min_b_out, "Insufficient output amount");

    // Update reserves
    pool.token_a_reserve = pool.token_a_reserve + amount_a_in;
    pool.token_b_reserve = pool.token_b_reserve - amount_b_out;

    // Verify invariant is maintained (k can only increase due to fees)
    let new_k = pool.token_a_reserve * pool.token_b_reserve;
    require(new_k >= pool.last_k, "Invariant violation");

    // Update last_k
    pool.last_k = new_k;

    return amount_b_out;
}

// Swap token B for token A using the constant product formula
pub swap_b_to_a(
    pool: AMMPool @mut,
    amount_b_in: u64,
    min_a_out: u64
) -> u64 {
    // Verify input is positive
    require(amount_b_in > 0, "Input amount must be greater than zero");

    // Verify sufficient reserves
    require(pool.token_a_reserve > 0, "Pool has no token A reserve");
    require(pool.token_b_reserve > 0, "Pool has no token B reserve");

    // Calculate fee
    let fee_amount = (amount_b_in * pool.fee_bps) / 10000;
    let amount_b_net = amount_b_in - fee_amount;

    // Apply constant product formula: x*y = k
    // Output = (x * y_in) / (y + y_in)
    let denominator = pool.token_b_reserve + amount_b_net;
    let amount_a_out = (pool.token_a_reserve * amount_b_net) / denominator;

    // Verify output meets minimum
    require(amount_a_out >= min_a_out, "Insufficient output amount");

    // Update reserves
    pool.token_b_reserve = pool.token_b_reserve + amount_b_in;
    pool.token_a_reserve = pool.token_a_reserve - amount_a_out;

    // Verify invariant is maintained
    let new_k = pool.token_a_reserve * pool.token_b_reserve;
    require(new_k >= pool.last_k, "Invariant violation");

    // Update last_k
    pool.last_k = new_k;

    return amount_a_out;
}

// ============================================================================
// SPL TOKEN INTEGRATION (CPI CALLS)
// ============================================================================

// Call SPL Token program to initialize an SPL mint
pub create_spl_mint(
    payer: account @signer,
    mint: account @init,
    decimals: u8
) -> pubkey {
    // Call the SPL Token program's initialize_mint instruction
    SPLToken.initialize_mint(
        get_key(mint),
        decimals,
        payer.key,
        payer.key
    );

    return get_key(mint);
}

// Call SPL Token program to mint tokens
pub mint_spl_tokens(
    mint: account @mut,
    destination: account @mut,
    authority: account @signer,
    amount: u64
) {
    // Call the SPL Token program's mint_to instruction
    SPLToken.mint_to(
        get_key(mint),
        get_key(destination),
        authority.key,
        amount
    );
}

// Call SPL Token program to transfer tokens
pub transfer_spl_tokens(
    source: account @mut,
    destination: account @mut,
    authority: account @signer,
    amount: u64
) {
    // Call the SPL Token program's transfer instruction
    SPLToken.transfer(
        get_key(source),
        get_key(destination),
        authority.key,
        amount
    );
}

// Call SPL Token program to burn tokens
pub burn_spl_tokens(
    mint: account @mut,
    token_account: account @mut,
    authority: account @signer,
    amount: u64
) {
    // Call the SPL Token program's burn instruction
    SPLToken.burn(
        get_key(mint),
        get_key(token_account),
        authority.key,
        amount
    );
}

// ============================================================================
// PRICE QUOTE FUNCTIONS (READ-ONLY)
// ============================================================================

// Quote the amount of token B you'd receive for swapping amount_a
pub quote_swap_a_to_b(pool: AMMPool, amount_a_in: u64) -> u64 {
    // Verify inputs
    require(amount_a_in > 0, "Input amount must be greater than zero");
    require(pool.token_a_reserve > 0, "Pool has no reserves");
    require(pool.token_b_reserve > 0, "Pool has no reserves");

    // Calculate fee
    let fee_amount = (amount_a_in * pool.fee_bps) / 10000;
    let amount_a_net = amount_a_in - fee_amount;

    // Apply constant product formula
    let denominator = pool.token_a_reserve + amount_a_net;
    let amount_b_out = (pool.token_b_reserve * amount_a_net) / denominator;

    return amount_b_out;
}

// Quote the amount of token A you'd receive for swapping amount_b
pub quote_swap_b_to_a(pool: AMMPool, amount_b_in: u64) -> u64 {
    // Verify inputs
    require(amount_b_in > 0, "Input amount must be greater than zero");
    require(pool.token_a_reserve > 0, "Pool has no reserves");
    require(pool.token_b_reserve > 0, "Pool has no reserves");

    // Calculate fee
    let fee_amount = (amount_b_in * pool.fee_bps) / 10000;
    let amount_b_net = amount_b_in - fee_amount;

    // Apply constant product formula
    let denominator = pool.token_b_reserve + amount_b_net;
    let amount_a_out = (pool.token_a_reserve * amount_b_net) / denominator;

    return amount_a_out;
}

// Quote LP shares you'd receive for adding liquidity
pub quote_add_liquidity(
    pool: AMMPool,
    amount_a: u64,
    amount_b: u64
) -> u64 {
    if (amount_a == 0 || amount_b == 0) {
        return 0;
    }

    if (pool.total_lp_shares == 0) {
        // Initial liquidity
        return sqrt_product(amount_a, amount_b);
    } else {
        // Proportional share
        let shares_from_a = (amount_a * pool.total_lp_shares) / pool.token_a_reserve;
        let shares_from_b = (amount_b * pool.total_lp_shares) / pool.token_b_reserve;

        if (shares_from_a < shares_from_b) {
            return shares_from_a;
        } else {
            return shares_from_b;
        }
    }
}

// ============================================================================
// POOL INFORMATION
// ============================================================================

// Get the reserves of the pool
pub get_reserves(pool: AMMPool) -> (u64, u64) {
    return (pool.token_a_reserve, pool.token_b_reserve);
}

// Get the total LP shares
pub get_total_lp_shares(pool: AMMPool) -> u64 {
    return pool.total_lp_shares;
}

// Get the fee in basis points
pub get_fee_bps(pool: AMMPool) -> u64 {
    return pool.fee_bps;
}

// Get the LP shares balance for an account
pub get_lp_balance(lp_account: LPAccount) -> u64 {
    return lp_account.lp_shares;
}

// Calculate current price (token_b per unit of token_a)
pub get_spot_price(pool: AMMPool) -> u64 {
    require(pool.token_a_reserve > 0, "Pool has no reserves");

    // Price = token_b_reserve / token_a_reserve
    // Returning scaled by 1e6 for precision
    return (pool.token_b_reserve * 1000000) / pool.token_a_reserve;
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

// Calculate approximate square root of product for initial LP shares
// Uses a simplified algorithm for WASM compatibility
fn sqrt_product(a: u64, b: u64) -> u64 {
    // Simple approximation: if a and b are roughly equal, sqrt(a*b) ≈ (a+b)/2
    // For initial liquidity, use minimum of a and b as approximation
    if (a < b) {
        return a;
    } else {
        return b;
    }
}

// Verify pool invariant (k value increases or stays same)
fn verify_invariant(pool: AMMPool) -> bool {
    let current_k = pool.token_a_reserve * pool.token_b_reserve;
    return current_k >= pool.last_k;
}
