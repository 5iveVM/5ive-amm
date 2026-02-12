import amm_types;
import amm_liquidity;
import amm_swap;
import pool_manager;

use "11111111111111111111111111111111"::{transfer};

pub fn initialize_pool(
    pool: amm_types::AMMPool @mut @init(payer=payer),
    payer: account @signer,
    token_a: pubkey,
    token_b: pubkey,
    fee_bps: u64
) {
    pool_manager::initialize_pool(pool, payer, token_a, token_b, fee_bps);
}

pub fn add_liquidity(
    pool: amm_types::AMMPool @mut,
    lp_account: amm_types::LPTokenAccount @mut,
    provider: account @signer,
    provider_token_a: account @mut,
    provider_token_b: account @mut,
    pool_token_a: account @mut,
    pool_token_b: account @mut,
    amount_a: u64,
    amount_b: u64,
    min_liquidity: u64,
    token_bytecode: account
) -> u64 {
    transfer(provider_token_a, pool_token_a, provider, amount_a);
    transfer(provider_token_b, pool_token_b, provider, amount_b);

    return amm_liquidity::add_liquidity(
        pool,
        lp_account,
        provider,
        amount_a,
        amount_b,
        min_liquidity
    );
}

pub fn remove_liquidity(
    pool: amm_types::AMMPool @mut,
    lp_account: amm_types::LPTokenAccount @mut,
    provider: account @signer,
    pool_authority: account @signer,
    pool_token_a: account @mut,
    pool_token_b: account @mut,
    provider_token_a: account @mut,
    provider_token_b: account @mut,
    liquidity: u64,
    min_a: u64,
    min_b: u64,
    token_bytecode: account
) {
    let amount_a: u64 = (liquidity * pool.token_a_reserve) / pool.total_liquidity;
    let amount_b: u64 = (liquidity * pool.token_b_reserve) / pool.total_liquidity;

    amm_liquidity::remove_liquidity(pool, lp_account, provider, liquidity, min_a, min_b);

    transfer(pool_token_a, provider_token_a, pool_authority, amount_a);
    transfer(pool_token_b, provider_token_b, pool_authority, amount_b);
}

pub fn swap_a_to_b(
    pool: amm_types::AMMPool @mut,
    trader: account @signer,
    pool_authority: account @signer,
    trader_token_a: account @mut,
    trader_token_b: account @mut,
    pool_token_a: account @mut,
    pool_token_b: account @mut,
    amount_in: u64,
    min_out: u64,
    token_bytecode: account
) -> u64 {
    transfer(trader_token_a, pool_token_a, trader, amount_in);

    let amount_out: u64 = amm_swap::swap_a_to_b(pool, amount_in, min_out);

    transfer(pool_token_b, trader_token_b, pool_authority, amount_out);

    return amount_out;
}

pub fn swap_b_to_a(
    pool: amm_types::AMMPool @mut,
    trader: account @signer,
    pool_authority: account @signer,
    trader_token_b: account @mut,
    trader_token_a: account @mut,
    pool_token_b: account @mut,
    pool_token_a: account @mut,
    amount_in: u64,
    min_out: u64,
    token_bytecode: account
) -> u64 {
    transfer(trader_token_b, pool_token_b, trader, amount_in);

    let amount_out: u64 = amm_swap::swap_b_to_a(pool, amount_in, min_out);

    transfer(pool_token_a, trader_token_a, pool_authority, amount_out);

    return amount_out;
}
