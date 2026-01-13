// ============================================================================
// AMM TYPES
// ============================================================================

account AMMPool {
    token_a_reserve: u64;
    token_b_reserve: u64;
    total_lp_shares: u64;
    fee_bps: u64;
    token_a_mint: pubkey;
    token_b_mint: pubkey;
    lp_token_mint: pubkey;
    pool_authority: pubkey;
    initialized: bool;
    is_paused: bool;
    name: string<32>;
}

account LPAccount {
    owner: pubkey;
    pool: pubkey;
    lp_shares: u64;
}
