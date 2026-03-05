// ============================================================================
// STAKING CORE
// ============================================================================

pub init_staking_pool(
    pool: StakingPool @mut @init,
    authority: account @signer,
    staking_mint: pubkey,
    reward_mint: pubkey,
    reward_rate_bps: u64,
    name: string
) -> pubkey {
    require(reward_rate_bps <= 10000);
    pool.authority = authority.key;
    pool.staking_mint = staking_mint;
    pool.reward_mint = reward_mint;
    pool.total_staked = 0;
    pool.reward_rate_bps = reward_rate_bps;
    pool.last_update_slot = get_clock().slot;
    pool.is_paused = false;
    pool.name = name;
    return pool.key;
}

pub update_reward_rate(
    pool: StakingPool @mut,
    authority: account @signer,
    reward_rate_bps: u64
) {
    require(pool.authority == authority.key);
    require(reward_rate_bps <= 10000);
    pool.reward_rate_bps = reward_rate_bps;
    pool.last_update_slot = get_clock().slot;
}

pub pause_pool(pool: StakingPool @mut, authority: account @signer) {
    require(pool.authority == authority.key);
    pool.is_paused = true;
}

pub unpause_pool(pool: StakingPool @mut, authority: account @signer) {
    require(pool.authority == authority.key);
    pool.is_paused = false;
}
