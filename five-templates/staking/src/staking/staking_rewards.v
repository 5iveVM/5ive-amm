// ============================================================================
// STAKING REWARDS
// ============================================================================

pub init_staker(
    staker: Staker @mut @init,
    owner: account @signer,
    pool: pubkey
) -> pubkey {
    staker.owner = owner.key;
    staker.pool = pool;
    staker.staked_amount = 0;
    staker.reward_debt = 0;
    staker.last_update_slot = get_clock();
    return staker.key;
}

pub stake(
    pool: StakingPool @mut,
    staker: Staker @mut,
    owner: account @signer,
    amount: u64
) {
    require(!pool.is_paused);
    require(staker.owner == owner.key);
    require(staker.pool == pool.key);
    require(amount > 0);
    require(pool.total_staked <= 18446744073709551615 - amount);
    require(staker.staked_amount <= 18446744073709551615 - amount);

    pool.total_staked = pool.total_staked + amount;
    staker.staked_amount = staker.staked_amount + amount;
    staker.last_update_slot = get_clock();
}

pub unstake(
    pool: StakingPool @mut,
    staker: Staker @mut,
    owner: account @signer,
    amount: u64
) {
    require(!pool.is_paused);
    require(staker.owner == owner.key);
    require(staker.pool == pool.key);
    require(amount > 0);
    require(staker.staked_amount >= amount);
    require(pool.total_staked >= amount);

    pool.total_staked = pool.total_staked - amount;
    staker.staked_amount = staker.staked_amount - amount;
    staker.last_update_slot = get_clock();
}

pub claim_rewards(
    pool: StakingPool @mut,
    staker: Staker @mut,
    owner: account @signer
) -> u64 {
    require(staker.owner == owner.key);
    require(staker.pool == pool.key);

    let reward: u64 = (staker.staked_amount * pool.reward_rate_bps) / 10000;
    require(reward >= staker.reward_debt);
    let claimable: u64 = reward - staker.reward_debt;
    staker.reward_debt = reward;
    staker.last_update_slot = get_clock();
    return claimable;
}
