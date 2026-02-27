// Single-sided staking template (simplified)

account Pool {
    reward_rate_per_slot: u64; // rewards per slot per staked unit (scaled)
    last_update_slot: u64;
    acc_reward_per_share: u64; // accumulated rewards per share (scaled)
    scale: u64; // scaling factor
}

account StakeAccount {
    owner_key: pubkey;
    amount: u64; // staked amount
    reward_debt: u64; // amount * acc_reward_per_share at stake/update
}

// Initialize pool
pub init_pool(state: Pool @mut, reward_rate_per_slot: u64, scale: u64) {
    state.reward_rate_per_slot = reward_rate_per_slot;
    state.last_update_slot = get_clock();
    state.acc_reward_per_share = 0;
    state.scale = scale;
}

// Update global accumulator (no total supply tracking for simplicity)
pub accrue(state: Pool @mut, slots: u64) {
    // In a real pool, this would depend on total staked; here we just add scaled rate
    state.acc_reward_per_share = state.acc_reward_per_share + (state.reward_rate_per_slot * slots);
    state.last_update_slot = state.last_update_slot + slots;
}

// Initialize a staker
pub init_staker(state: StakeAccount @mut, owner: pubkey) {
    state.owner_key = owner;
    state.amount = 0;
    state.reward_debt = 0;
}

// Stake more tokens (accounting only)
pub stake(state: StakeAccount @mut, owner: pubkey, amount: u64, acc_reward_per_share: u64) {
    require(state.owner_key == owner);
    state.reward_debt = state.reward_debt + (amount * acc_reward_per_share);
    state.amount = state.amount + amount;
}

// Unstake some tokens (accounting only)
pub unstake(state: StakeAccount @mut, owner: pubkey, amount: u64, acc_reward_per_share: u64) {
    require(state.owner_key == owner);
    require(state.amount >= amount);
    state.amount = state.amount - amount;
    // Adjust debt proportionally (simplified)
    state.reward_debt = state.reward_debt - (amount * acc_reward_per_share);
}

// Claim rewards based on external view of acc_reward_per_share
pub claimable(state: StakeAccount, acc_reward_per_share: u64) -> u64 {
    let accrued = state.amount * acc_reward_per_share;
    if (accrued <= state.reward_debt) { return 0; }
    return accrued - state.reward_debt;
}

// After tokens sent, record the claim (accounting only)
pub record_claim(state: StakeAccount @mut, claimed: u64) {
    state.reward_debt = state.reward_debt + claimed;
}
