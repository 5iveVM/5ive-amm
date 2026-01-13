// ============================================================================
// STABLECOIN TYPES
// ============================================================================

account StablecoinEngine {
    authority: pubkey;
    collateral_mint: pubkey;
    debt_mint: pubkey;
    collateral_factor_bps: u64;
    liquidation_threshold_bps: u64;
    total_collateral: u64;
    total_debt: u64;
    is_paused: bool;
    name: string<32>;
}

account Position {
    owner: pubkey;
    engine: pubkey;
    collateral: u64;
    debt: u64;
    last_update_slot: u64;
}
