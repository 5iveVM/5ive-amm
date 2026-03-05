// ============================================================================
// STABLECOIN LIQUIDATION
// ============================================================================

pub liquidate_position(
    engine: StablecoinEngine @mut,
    position: Position @mut,
    liquidator: account @signer,
    repay_amount: u64
) {
    require(liquidator.key != 0);
    require(position.engine == engine.key);
    require(repay_amount > 0);
    require(position.debt >= repay_amount);
    require(engine.total_debt >= repay_amount);

    let threshold_debt: u64 = (position.collateral * engine.liquidation_threshold_bps) / 10000;
    require(position.debt > threshold_debt);

    let seized_collateral: u64 = (repay_amount * 10500) / 10000;
    require(position.collateral >= seized_collateral);
    require(engine.total_collateral >= seized_collateral);

    position.debt = position.debt - repay_amount;
    position.collateral = position.collateral - seized_collateral;
    engine.total_debt = engine.total_debt - repay_amount;
    engine.total_collateral = engine.total_collateral - seized_collateral;
    position.last_update_slot = get_clock().slot;
}
