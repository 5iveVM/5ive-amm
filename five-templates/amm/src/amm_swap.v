// Swap A for B
pub fn swap_a_to_b(
    pool: AMMPool @mut,
    amount_a_in: u64,
    min_b_out: u64
) -> u64 {
    require(!pool.is_paused);
    require(amount_a_in > 0);
    
    // Fee
    let fee = (amount_a_in * pool.fee_bps) / 10000;
    let amount_a_less_fee = amount_a_in - fee;
    
    // Calculate Output
    // (x + dx)(y - dy) = k
    // y - dy = k / (x + dx)
    // dy = y - k / (x + dx)
    
    let denominator = pool.token_a_reserve + amount_a_less_fee;
    let amount_b_out = (pool.token_b_reserve * amount_a_less_fee) / denominator;
    
    require(amount_b_out >= min_b_out);
    
    // Update Reserves
    pool.token_a_reserve = pool.token_a_reserve + amount_a_in;
    pool.token_b_reserve = pool.token_b_reserve - amount_b_out;
    
    // Recalculate k
    pool.last_k = pool.token_a_reserve * pool.token_b_reserve;
    
    return amount_b_out;
}

// Swap B for A
pub fn swap_b_to_a(
    pool: AMMPool @mut,
    amount_b_in: u64,
    min_a_out: u64
) -> u64 {
    require(!pool.is_paused);
    require(amount_b_in > 0);
    
    // Fee
    let fee = (amount_b_in * pool.fee_bps) / 10000;
    let amount_b_less_fee = amount_b_in - fee;
    
    // Calculate Output
    let denominator = pool.token_b_reserve + amount_b_less_fee;
    let amount_a_out = (pool.token_a_reserve * amount_b_less_fee) / denominator;
    
    require(amount_a_out >= min_a_out);
    
    // Update Reserves
    pool.token_b_reserve = pool.token_b_reserve + amount_b_in;
    pool.token_a_reserve = pool.token_a_reserve - amount_a_out;
    
    // Recalculate k
    pool.last_k = pool.token_a_reserve * pool.token_b_reserve;
    
    return amount_a_out;
}
