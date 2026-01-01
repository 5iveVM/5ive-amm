// Five DSL - AMM Swap between Five Native Token and SPL Token
// Modular Refactored Version

// ============================================================================
// EXTERNAL INTERFACES
// ============================================================================

interface SPLToken {
    // Initialize a new mint
    initialize_mint @discriminator(0) (
        mint: pubkey,
        decimals: u8,
        authority: pubkey,
        freeze_authority: pubkey
    )

    // Mint tokens to a destination account
    mint_to @discriminator(7) (
        mint: pubkey,
        to: pubkey,
        authority: pubkey,
        amount: u64
    )

    // Transfer tokens between accounts
    transfer @discriminator(3) (
        source: pubkey,
        destination: pubkey,
        authority: pubkey,
        amount: u64
    )

    // Burn tokens from an account
    burn @discriminator(8) (
        mint: pubkey,
        target_account: pubkey,
        authority: pubkey,
        amount: u64
    )
}

// ============================================================================
// ACCOUNT DEFINITIONS
// ============================================================================

account AMMPool {
    token_a_reserve: u64;
    token_b_reserve: u64;
    total_lp_shares: u64;
    fee_bps: u64;
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

// Import modular math helper
import math;

// ============================================================================
// POOL INITIALIZATION
// ============================================================================

pub init_pool(
    pool: AMMPool @mut,
    authority: account @signer,
    token_a_mint: pubkey,
    token_b_mint: pubkey,
    lp_token_mint: pubkey,
    fee_bps: u64
) -> pubkey {
    require(fee_bps <= 1000);

    pool.token_a_reserve = 0;
    pool.token_b_reserve = 0;
    pool.total_lp_shares = 0;
    pool.fee_bps = fee_bps;
    pool.token_a_mint = token_a_mint;
    pool.token_b_mint = token_b_mint;
    pool.pool_authority = authority.key;
    pool.lp_token_mint = lp_token_mint;
    pool.last_k = 0;

    return pool.key;
}

pub init_lp_account(
    lp_account: LPAccount @mut,
    owner: account @signer,
    pool: pubkey
) -> pubkey {
    lp_account.owner = owner.key;
    lp_account.pool = pool;
    lp_account.lp_shares = 0;

    return lp_account.key;
}

// ============================================================================
// LIQUIDITY OPERATIONS
// ============================================================================

pub add_liquidity(
    pool: AMMPool @mut,
    lp_account: LPAccount @mut,
    owner: account @signer,
    amount_a: u64,
    amount_b: u64,
    min_lp_shares: u64
) -> u64 {
    require(amount_a > 0);
    require(amount_b > 0);
    require(lp_account.owner == owner.key);
    require(lp_account.pool == pool.key);

    let mut lp_shares: u64 = 0;

    if (pool.total_lp_shares == 0) {
        // Use proper integer sqrt from math module
        lp_shares = sqrt_product(amount_a, amount_b);
        require(lp_shares >= min_lp_shares);
    } else {
        let shares_from_a = (amount_a * pool.total_lp_shares) / pool.token_a_reserve;
        let shares_from_b = (amount_b * pool.total_lp_shares) / pool.token_b_reserve;

        if (shares_from_a < shares_from_b) {
            lp_shares = shares_from_a;
        } else {
            lp_shares = shares_from_b;
        }

        require(lp_shares >= min_lp_shares);
    }

    pool.token_a_reserve = pool.token_a_reserve + amount_a;
    pool.token_b_reserve = pool.token_b_reserve + amount_b;
    pool.total_lp_shares = pool.total_lp_shares + lp_shares;
    pool.last_k = pool.token_a_reserve * pool.token_b_reserve;

    lp_account.lp_shares = lp_account.lp_shares + lp_shares;

    return lp_shares;
}

pub remove_liquidity(
    pool: AMMPool @mut,
    lp_account: LPAccount @mut,
    owner: account @signer,
    lp_shares: u64,
    min_amount_a: u64,
    min_amount_b: u64
) -> (u64, u64) {
    require(lp_shares > 0);
    require(lp_account.owner == owner.key);
    require(lp_account.lp_shares >= lp_shares);

    let amount_a = (lp_shares * pool.token_a_reserve) / pool.total_lp_shares;
    let amount_b = (lp_shares * pool.token_b_reserve) / pool.total_lp_shares;

    require(amount_a >= min_amount_a);
    require(amount_b >= min_amount_b);

    pool.token_a_reserve = pool.token_a_reserve - amount_a;
    pool.token_b_reserve = pool.token_b_reserve - amount_b;
    pool.total_lp_shares = pool.total_lp_shares - lp_shares;
    pool.last_k = pool.token_a_reserve * pool.token_b_reserve;

    lp_account.lp_shares = lp_account.lp_shares - lp_shares;

    return (amount_a, amount_b);
}

// ============================================================================
// SWAP OPERATIONS
// ============================================================================

pub swap_a_to_b(
    pool: AMMPool @mut,
    amount_a_in: u64,
    min_b_out: u64
) -> u64 {
    require(amount_a_in > 0);
    require(pool.token_a_reserve > 0);
    require(pool.token_b_reserve > 0);

    let fee_amount = (amount_a_in * pool.fee_bps) / 10000;
    let amount_a_net = amount_a_in - fee_amount;

    let denominator = pool.token_a_reserve + amount_a_net;
    let amount_b_out = (pool.token_b_reserve * amount_a_net) / denominator;

    require(amount_b_out >= min_b_out);

    pool.token_a_reserve = pool.token_a_reserve + amount_a_in;
    pool.token_b_reserve = pool.token_b_reserve - amount_b_out;

    let new_k = pool.token_a_reserve * pool.token_b_reserve;
    require(new_k >= pool.last_k);

    pool.last_k = new_k;

    return amount_b_out;
}

pub swap_b_to_a(
    pool: AMMPool @mut,
    amount_b_in: u64,
    min_a_out: u64
) -> u64 {
    require(amount_b_in > 0);
    require(pool.token_a_reserve > 0);
    require(pool.token_b_reserve > 0);

    let fee_amount = (amount_b_in * pool.fee_bps) / 10000;
    let amount_b_net = amount_b_in - fee_amount;

    let denominator = pool.token_b_reserve + amount_b_net;
    let amount_a_out = (pool.token_a_reserve * amount_b_net) / denominator;

    require(amount_a_out >= min_a_out);

    pool.token_b_reserve = pool.token_b_reserve + amount_b_in;
    pool.token_a_reserve = pool.token_a_reserve - amount_a_out;

    let new_k = pool.token_a_reserve * pool.token_b_reserve;
    require(new_k >= pool.last_k);

    pool.last_k = new_k;

    return amount_a_out;
}

// ============================================================================
// SPL TOKEN INTEGRATION
// ============================================================================

pub create_spl_mint(
    payer: account @signer,
    mint: account @init,
    decimals: u8
) -> pubkey {
    SPLToken.initialize_mint(
        mint.key,
        decimals,
        payer.key,
        payer.key
    );

    return mint.key;
}

pub mint_spl_tokens(
    mint: account @mut,
    destination: account @mut,
    authority: account @signer,
    amount: u64
) {
    SPLToken.mint_to(
        mint.key,
        destination.key,
        authority.key,
        amount
    );
}

pub transfer_spl_tokens(
    source: account @mut,
    destination: account @mut,
    authority: account @signer,
    amount: u64
) {
    SPLToken.transfer(
        source.key,
        destination.key,
        authority.key,
        amount
    );
}

pub burn_spl_tokens(
    mint: account @mut,
    token_account: account @mut,
    authority: account @signer,
    amount: u64
) {
    SPLToken.burn(
        mint.key,
        token_account.key,
        authority.key,
        amount
    );
}
