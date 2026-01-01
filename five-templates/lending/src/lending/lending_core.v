// ============================================================================
// LENDING CORE
// ============================================================================

pub init_lending_market(
    market: LendingMarket @mut @init,
    authority: account @signer,
    quote_mint: pubkey,
    name: string
) -> pubkey {
    market.authority = authority.key;
    market.quote_mint = quote_mint;
    market.is_paused = false;
    market.name = name;
    market.created_slot = get_clock();
    return market.key;
}

pub init_reserve(
    reserve: Reserve @mut @init,
    authority: account @signer,
    market: LendingMarket,
    liquidity_mint: pubkey,
    collateral_mint: pubkey,
    borrow_rate_bps: u64,
    collateral_factor_bps: u64,
    liquidation_threshold_bps: u64,
    name: string
) -> pubkey {
    require(market.authority == authority.key);
    require(borrow_rate_bps <= 10000);
    require(liquidation_threshold_bps <= 10000);
    require(collateral_factor_bps <= liquidation_threshold_bps);

    reserve.market = market.key;
    reserve.liquidity_mint = liquidity_mint;
    reserve.collateral_mint = collateral_mint;
    reserve.total_deposits = 0;
    reserve.total_borrows = 0;
    reserve.available_liquidity = 0;
    reserve.borrow_rate_bps = borrow_rate_bps;
    reserve.collateral_factor_bps = collateral_factor_bps;
    reserve.liquidation_threshold_bps = liquidation_threshold_bps;
    reserve.last_update_slot = get_clock();
    reserve.is_paused = false;
    reserve.name = name;
    return reserve.key;
}
