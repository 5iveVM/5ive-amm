// ============================================================================
// LENDING BORROW
// ============================================================================

pub borrow(
    reserve: Reserve @mut,
    obligation: Obligation @mut,
    borrower: account @signer,
    amount: u64
) {
    require(!reserve.is_paused);
    require(obligation.owner == borrower.ctx.key);
    require(obligation.reserve == reserve.ctx.key);
    require(amount > 0);
    require(reserve.available_liquidity >= amount);

    let max_borrow: u64 = (obligation.deposited_collateral * reserve.collateral_factor_bps) / 10000;
    require(max_borrow >= obligation.borrowed_amount + amount);

    reserve.available_liquidity = reserve.available_liquidity - amount;
    reserve.total_borrows = reserve.total_borrows + amount;
    obligation.borrowed_amount = obligation.borrowed_amount + amount;
    obligation.last_update_slot = get_clock();
    reserve.last_update_slot = get_clock();
}

pub repay(
    reserve: Reserve @mut,
    obligation: Obligation @mut,
    payer: account @signer,
    amount: u64
) {
    require(payer.ctx.key != 0);
    require(obligation.reserve == reserve.ctx.key);
    require(obligation.borrowed_amount >= amount);
    require(reserve.total_borrows >= amount);

    obligation.borrowed_amount = obligation.borrowed_amount - amount;
    reserve.total_borrows = reserve.total_borrows - amount;
    reserve.available_liquidity = reserve.available_liquidity + amount;
    obligation.last_update_slot = get_clock();
    reserve.last_update_slot = get_clock();
}
