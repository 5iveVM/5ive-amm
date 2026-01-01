// ============================================================================
// LENDING DEPOSIT
// ============================================================================

pub init_obligation(
    obligation: Obligation @mut @init,
    owner: account @signer,
    reserve: pubkey
) -> pubkey {
    obligation.owner = owner.key;
    obligation.reserve = reserve;
    obligation.deposited_collateral = 0;
    obligation.borrowed_amount = 0;
    obligation.last_update_slot = get_clock();
    return obligation.key;
}

pub deposit_collateral(
    reserve: Reserve @mut,
    obligation: Obligation @mut,
    owner: account @signer,
    amount: u64
) {
    require(!reserve.is_paused);
    require(obligation.owner == owner.key);
    require(obligation.reserve == reserve.key);
    require(amount > 0);

    reserve.total_deposits = reserve.total_deposits + amount;
    reserve.available_liquidity = reserve.available_liquidity + amount;
    obligation.deposited_collateral = obligation.deposited_collateral + amount;
    obligation.last_update_slot = get_clock();
    reserve.last_update_slot = get_clock();
}
