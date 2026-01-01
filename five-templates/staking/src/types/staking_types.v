// ============================================================================
// STAKING TYPES
// ============================================================================

account StakingPool {
    authority: pubkey;
    staking_mint: pubkey;
    reward_mint: pubkey;
    total_staked: u64;
    reward_rate_bps: u64;
    last_update_slot: u64;
    is_paused: bool;
    name: string;
}

account Staker {
    owner: pubkey;
    pool: pubkey;
    staked_amount: u64;
    reward_debt: u64;
    last_update_slot: u64;
}

pub fn is_pool_active(pool: StakingPool) -> bool {
    return pool.authority != 0 && !pool.is_paused;
}
