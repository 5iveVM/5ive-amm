// ============================================================================
// PERPS CORE
// ============================================================================

pub init_market(
    market: PerpMarket @mut @init,
    authority: account @signer,
    base_mint: pubkey,
    quote_mint: pubkey,
    maintenance_margin_bps: u64,
    liquidation_fee_bps: u64,
    name: string
) -> pubkey {
    require(maintenance_margin_bps <= 10000);
    require(liquidation_fee_bps <= 10000);

    market.authority = authority.key;
    market.base_mint = base_mint;
    market.quote_mint = quote_mint;
    market.maintenance_margin_bps = maintenance_margin_bps;
    market.liquidation_fee_bps = liquidation_fee_bps;
    market.is_paused = false;
    market.name = name;
    return market.key;
}

pub init_position(
    position: Position @mut @init,
    owner: account @signer,
    market: pubkey
) -> pubkey {
    position.owner = owner.key;
    position.market = market;
    position.size = 0;
    position.entry_price = 0;
    position.collateral = 0;
    return position.key;
}

pub open_position(
    market: PerpMarket,
    position: Position @mut,
    owner: account @signer,
    size: u64,
    entry_price: u64,
    collateral: u64
) {
    require(!market.is_paused);
    require(position.owner == owner.key);
    require(position.market == market.key);
    require(size > 0);
    require(entry_price > 0);
    require(collateral > 0);

    position.size = size;
    position.entry_price = entry_price;
    position.collateral = collateral;
}

pub close_position(
    position: Position @mut,
    owner: account @signer
) {
    require(position.owner == owner.key);
    position.size = 0;
    position.entry_price = 0;
    position.collateral = 0;
}

pub update_collateral(
    position: Position @mut,
    owner: account @signer,
    collateral: u64
) {
    require(position.owner == owner.key);
    position.collateral = collateral;
}
