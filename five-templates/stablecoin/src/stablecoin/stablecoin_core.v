// ============================================================================
// STABLECOIN CORE
// ============================================================================

pub init_engine(
    engine: StablecoinEngine @mut @init,
    authority: account @signer,
    collateral_mint: pubkey,
    debt_mint: pubkey,
    collateral_factor_bps: u64,
    liquidation_threshold_bps: u64,
    name: string
) -> pubkey {
    require(collateral_factor_bps <= liquidation_threshold_bps);
    require(liquidation_threshold_bps <= 10000);

    engine.authority = authority.key;
    engine.collateral_mint = collateral_mint;
    engine.debt_mint = debt_mint;
    engine.collateral_factor_bps = collateral_factor_bps;
    engine.liquidation_threshold_bps = liquidation_threshold_bps;
    engine.total_collateral = 0;
    engine.total_debt = 0;
    engine.is_paused = false;
    engine.name = name;
    return engine.key;
}

pub init_position(
    position: Position @mut @init,
    owner: account @signer,
    engine: pubkey
) -> pubkey {
    position.owner = owner.key;
    position.engine = engine;
    position.collateral = 0;
    position.debt = 0;
    position.last_update_slot = get_clock().slot;
    return position.key;
}

pub deposit_collateral(
    engine: StablecoinEngine @mut,
    position: Position @mut,
    owner: account @signer,
    amount: u64
) {
    require(!engine.is_paused);
    require(position.owner == owner.key);
    require(position.engine == engine.key);
    require(amount > 0);
    require(engine.total_collateral <= 18446744073709551615 - amount);
    require(position.collateral <= 18446744073709551615 - amount);

    engine.total_collateral = engine.total_collateral + amount;
    position.collateral = position.collateral + amount;
    position.last_update_slot = get_clock().slot;
}

pub withdraw_collateral(
    engine: StablecoinEngine @mut,
    position: Position @mut,
    owner: account @signer,
    amount: u64
) {
    require(!engine.is_paused);
    require(position.owner == owner.key);
    require(position.engine == engine.key);
    require(position.collateral >= amount);
    require(engine.total_collateral >= amount);

    let new_collateral: u64 = position.collateral - amount;
    let max_debt: u64 = (new_collateral * engine.collateral_factor_bps) / 10000;
    require(max_debt >= position.debt);

    engine.total_collateral = engine.total_collateral - amount;
    position.collateral = new_collateral;
    position.last_update_slot = get_clock().slot;
}

pub mint_stable(
    engine: StablecoinEngine @mut,
    position: Position @mut,
    owner: account @signer,
    amount: u64
) {
    require(!engine.is_paused);
    require(position.owner == owner.key);
    require(position.engine == engine.key);
    require(amount > 0);
    require(engine.total_debt <= 18446744073709551615 - amount);
    require(position.debt <= 18446744073709551615 - amount);

    let max_debt: u64 = (position.collateral * engine.collateral_factor_bps) / 10000;
    require(max_debt >= position.debt + amount);

    engine.total_debt = engine.total_debt + amount;
    position.debt = position.debt + amount;
    position.last_update_slot = get_clock().slot;
}

pub repay_stable(
    engine: StablecoinEngine @mut,
    position: Position @mut,
    payer: account @signer,
    amount: u64
) {
    require(payer.key != 0);
    require(position.engine == engine.key);
    require(position.debt >= amount);
    require(engine.total_debt >= amount);

    engine.total_debt = engine.total_debt - amount;
    position.debt = position.debt - amount;
    position.last_update_slot = get_clock().slot;
}
