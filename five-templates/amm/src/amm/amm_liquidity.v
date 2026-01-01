// ============================================================================
// AMM LIQUIDITY
// ============================================================================

pub add_liquidity(
    pool: AMMPool @mut,
    lp: LPAccount @mut,
    owner: account @signer,
    amount_a: u64,
    amount_b: u64
) -> u64 {
    require(!pool.is_paused);
    require(lp.owner == owner.key);
    require(lp.pool == pool.key);
    require(amount_a > 0);
    require(amount_b > 0);

    let shares: u64 = 0;
    if (pool.total_lp_shares == 0) {
        shares = amount_a + amount_b;
    } else {
        shares = (amount_a * pool.total_lp_shares) / pool.token_a_reserve;
    }

    pool.token_a_reserve = pool.token_a_reserve + amount_a;
    pool.token_b_reserve = pool.token_b_reserve + amount_b;
    pool.total_lp_shares = pool.total_lp_shares + shares;
    lp.lp_shares = lp.lp_shares + shares;
    return shares;
}

pub remove_liquidity(
    pool: AMMPool @mut,
    lp: LPAccount @mut,
    owner: account @signer,
    shares: u64
) -> u64 {
    require(!pool.is_paused);
    require(lp.owner == owner.key);
    require(lp.pool == pool.key);
    require(shares > 0);
    require(lp.lp_shares >= shares);

    let amount_a: u64 = (pool.token_a_reserve * shares) / pool.total_lp_shares;
    let amount_b: u64 = (pool.token_b_reserve * shares) / pool.total_lp_shares;

    pool.token_a_reserve = pool.token_a_reserve - amount_a;
    pool.token_b_reserve = pool.token_b_reserve - amount_b;
    pool.total_lp_shares = pool.total_lp_shares - shares;
    lp.lp_shares = lp.lp_shares - shares;

    return amount_a + amount_b;
}
