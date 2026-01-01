// ============================================================================
// INSURANCE CORE
// ============================================================================

pub init_pool(
    pool: InsurancePool @mut @init,
    authority: account @signer,
    stake_mint: pubkey,
    premium_rate_bps: u64,
    name: string
) -> pubkey {
    require(premium_rate_bps <= 10000);
    pool.authority = authority.key;
    pool.stake_mint = stake_mint;
    pool.total_stake = 0;
    pool.premium_rate_bps = premium_rate_bps;
    pool.is_paused = false;
    pool.name = name;
    return pool.key;
}

pub purchase_policy(
    pool: InsurancePool,
    policy: Policy @mut @init,
    holder: account @signer,
    coverage_amount: u64,
    start_slot: u64,
    end_slot: u64
) -> pubkey {
    require(!pool.is_paused);
    require(coverage_amount > 0);
    require(start_slot < end_slot);

    policy.pool = pool.key;
    policy.holder = holder.key;
    policy.coverage_amount = coverage_amount;
    policy.premium_paid = (coverage_amount * pool.premium_rate_bps) / 10000;
    policy.start_slot = start_slot;
    policy.end_slot = end_slot;
    policy.active = true;
    return policy.key;
}
