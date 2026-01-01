// ============================================================================
// LENDING TYPES
// ============================================================================

account LendingMarket {
    authority: pubkey;
    quote_mint: pubkey;
    is_paused: bool;
    name: string;
    created_slot: u64;
}

account Reserve {
    market: pubkey;
    liquidity_mint: pubkey;
    collateral_mint: pubkey;
    total_deposits: u64;
    total_borrows: u64;
    available_liquidity: u64;
    borrow_rate_bps: u64;
    collateral_factor_bps: u64;
    liquidation_threshold_bps: u64;
    last_update_slot: u64;
    is_paused: bool;
    name: string;
}

account Obligation {
    owner: pubkey;
    reserve: pubkey;
    deposited_collateral: u64;
    borrowed_amount: u64;
    last_update_slot: u64;
}
