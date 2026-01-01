// ============================================================================
// INSURANCE TYPES
// ============================================================================

account InsurancePool {
    authority: pubkey;
    stake_mint: pubkey;
    total_stake: u64;
    premium_rate_bps: u64;
    is_paused: bool;
    name: string;
}

account Policy {
    pool: pubkey;
    holder: pubkey;
    coverage_amount: u64;
    premium_paid: u64;
    start_slot: u64;
    end_slot: u64;
    active: bool;
}

account Claim {
    policy: pubkey;
    claimant: pubkey;
    amount: u64;
    approved: bool;
}
