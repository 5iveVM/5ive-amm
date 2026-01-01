// ============================================================================
// LENDING LIQUIDATION
// ============================================================================

pub liquidate(
    reserve: Reserve @mut,
    obligation: Obligation @mut,
    liquidator: account @signer,
    repay_amount: u64
) {
    require(liquidator.key != 0);
    require(obligation.reserve == reserve.key);
    require(obligation.borrowed_amount >= repay_amount);

    let threshold_borrow: u64 = (obligation.deposited_collateral * reserve.liquidation_threshold_bps) / 10000;
    require(obligation.borrowed_amount > threshold_borrow);

    let seized_collateral: u64 = (repay_amount * 10500) / 10000;
    require(obligation.deposited_collateral >= seized_collateral);

    obligation.borrowed_amount = obligation.borrowed_amount - repay_amount;
    obligation.deposited_collateral = obligation.deposited_collateral - seized_collateral;
    reserve.total_borrows = reserve.total_borrows - repay_amount;
    reserve.total_deposits = reserve.total_deposits - seized_collateral;
    reserve.available_liquidity = reserve.available_liquidity + repay_amount - seized_collateral;
    obligation.last_update_slot = get_clock();
    reserve.last_update_slot = get_clock();
}
