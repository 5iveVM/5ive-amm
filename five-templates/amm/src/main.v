import amm_types;
import amm_math;
import amm_liquidity;
import amm_swap;
import pool_manager;

// Entry Points

pub fn initialize_pool(
    pool: AMMPool @mut @init,
    token_a: pubkey,
    token_b: pubkey,
    fee_bps: u64
) {
    pool_manager::initialize_pool(pool, token_a, token_b, fee_bps);
}

pub fn add_liquidity(
    pool: AMMPool @mut,
    lp_account: LPTokenAccount @mut,
    provider: account @signer,
    amount_a: u64,
    amount_b: u64,
    min_liquidity: u64
) -> u64 {
    return amm_liquidity::add_liquidity(pool, lp_account, provider, amount_a, amount_b, min_liquidity);
}

pub fn remove_liquidity(
    pool: AMMPool @mut,
    lp_account: LPTokenAccount @mut,
    provider: account @signer,
    liquidity: u64,
    min_a: u64,
    min_b: u64
) {
    amm_liquidity::remove_liquidity(pool, lp_account, provider, liquidity, min_a, min_b);
}

pub fn swap_a_to_b(
    pool: AMMPool @mut,
    amount_in: u64,
    min_out: u64
) -> u64 {
    return amm_swap::swap_a_to_b(pool, amount_in, min_out);
}

pub fn swap_b_to_a(
    pool: AMMPool @mut,
    amount_in: u64,
    min_out: u64
) -> u64 {
    return amm_swap::swap_b_to_a(pool, amount_in, min_out);
}
