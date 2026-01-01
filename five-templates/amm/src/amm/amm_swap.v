// ============================================================================
// AMM SWAP
// ============================================================================

pub swap_a_to_b(
    pool: AMMPool @mut,
    amount_in: u64,
    min_out: u64
) -> u64 {
    require(!pool.is_paused);
    require(amount_in > 0);
    require(pool.token_a_reserve > 0);
    require(pool.token_b_reserve > 0);

    let amount_in_after_fee: u64 = amount_in - (amount_in * pool.fee_bps) / 10000;
    let new_reserve_a: u64 = pool.token_a_reserve + amount_in_after_fee;
    let k: u64 = pool.token_a_reserve * pool.token_b_reserve;
    let new_reserve_b: u64 = k / new_reserve_a;
    let amount_out: u64 = pool.token_b_reserve - new_reserve_b;

    require(amount_out >= min_out);

    pool.token_a_reserve = new_reserve_a;
    pool.token_b_reserve = new_reserve_b;
    return amount_out;
}
