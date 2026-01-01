// ============================================================================
// PERPS LIQUIDATION
// ============================================================================

pub liquidate_position(
    market: PerpMarket,
    position: Position @mut,
    liquidator: account @signer
) {
    require(liquidator.key != 0);
    require(position.market == market.key);
    require(position.size > 0);
    require(position.entry_price > 0);

    let notional: u64 = position.size * position.entry_price;
    let maintenance: u64 = (notional * market.maintenance_margin_bps) / 10000;
    require(position.collateral < maintenance);

    position.size = 0;
    position.entry_price = 0;
    position.collateral = 0;
}
