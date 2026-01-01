// ============================================================================
// AMM CORE
// ============================================================================

pub init_pool(
    pool: AMMPool @mut @init,
    authority: account @signer,
    token_a_mint: pubkey,
    token_b_mint: pubkey,
    lp_token_mint: pubkey,
    fee_bps: u64,
    name: string
) -> pubkey {
    require(token_a_mint != token_b_mint);
    require(fee_bps <= 1000);

    pool.token_a_reserve = 0;
    pool.token_b_reserve = 0;
    pool.total_lp_shares = 0;
    pool.fee_bps = fee_bps;
    pool.token_a_mint = token_a_mint;
    pool.token_b_mint = token_b_mint;
    pool.lp_token_mint = lp_token_mint;
    pool.pool_authority = authority.key;
    pool.initialized = true;
    pool.is_paused = false;
    pool.name = name;
    return pool.key;
}

pub init_lp_account(
    lp: LPAccount @mut @init,
    owner: account @signer,
    pool: pubkey
) -> pubkey {
    lp.owner = owner.key;
    lp.pool = pool;
    lp.lp_shares = 0;
    return lp.key;
}

pub pause_pool(pool: AMMPool @mut, authority: account @signer) {
    require(pool.pool_authority == authority.key);
    pool.is_paused = true;
}

pub unpause_pool(pool: AMMPool @mut, authority: account @signer) {
    require(pool.pool_authority == authority.key);
    pool.is_paused = false;
}
