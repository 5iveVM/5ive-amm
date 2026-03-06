// AMM Unit Tests
// Validates constant-product math, fee split, slippage, and liquidity accounting.
// Last @test-params value is the expected result for non-void functions.

// Initial LP mint: first deposit uses sum of amounts
// @test-params 500 700 1200
pub test_initial_lp_mint(amount_a: u64, amount_b: u64) -> u64 {
    return amount_a + amount_b;
}

// Proportional LP: liquidity = (amount_a * lp_supply) / reserve_a
// @test-params 200 1500 3000 100
pub test_proportional_lp_mint(amount_a: u64, lp_supply: u64, reserve_a: u64) -> u64 {
    return (amount_a * lp_supply) / reserve_a;
}

// Ratio guard: amount_a * reserve_b == amount_b * reserve_a
// @test-params 200 300 2 3 true
pub test_ratio_guard_passes(amount_a: u64, amount_b: u64, reserve_a: u64, reserve_b: u64) -> bool {
    return amount_a * reserve_b == amount_b * reserve_a;
}

// Ratio guard should fail when ratio is wrong
// @test-params 200 301 2 3 false
pub test_ratio_guard_fails(amount_a: u64, amount_b: u64, reserve_a: u64, reserve_b: u64) -> bool {
    return amount_a * reserve_b == amount_b * reserve_a;
}

// Swap with split fee: protocol_fee + lp_fee carved from input, then AMM formula applied
// protocol_fee_num=1, lp_fee_num=2, fee_denom=1000 -> total_fee=3/1000
// dx_after_fee = in - protocol - lp = in * (denom - fee_num) / denom
// @test-params 1000 1000 100 1 2 1000 90
pub test_swap_with_split_fee(reserve_a: u64, reserve_b: u64, amount_in: u64, protocol_fee_num: u64, lp_fee_num: u64, fee_denom: u64) -> u64 {
    let protocol_fee: u64 = (amount_in * protocol_fee_num) / fee_denom;
    let lp_fee: u64 = (amount_in * lp_fee_num) / fee_denom;
    let dx_after_fee: u64 = amount_in - protocol_fee - lp_fee;
    return (reserve_b * dx_after_fee) / (reserve_a + dx_after_fee);
}

// Slippage check: amount_out must be >= min_amount_out
// @test-params 90 80 true
pub test_slippage_guard_passes(amount_out: u64, min_amount_out: u64) -> bool {
    return amount_out >= min_amount_out;
}

// @test-params 70 80 false
pub test_slippage_guard_fails(amount_out: u64, min_amount_out: u64) -> bool {
    return amount_out >= min_amount_out;
}

// k must not decrease after swap (constant product invariant)
// @test-params 1000 1000 100 3 1000 true
pub test_k_non_decreasing_after_swap(reserve_a: u64, reserve_b: u64, amount_in: u64, fee_numerator: u64, fee_denominator: u64) -> bool {
    let dx_after_fee: u64 = (amount_in * (fee_denominator - fee_numerator)) / fee_denominator;
    let amount_out: u64 = (reserve_b * dx_after_fee) / (reserve_a + dx_after_fee);
    let k_before: u64 = reserve_a * reserve_b;
    let new_a: u64 = reserve_a + amount_in;
    let new_b: u64 = reserve_b - amount_out;
    return new_a * new_b >= k_before;
}

// Larger input -> larger output (monotonicity)
// @test-params 1000 1000 50 200 3 1000 true
pub test_swap_output_monotonic(reserve_a: u64, reserve_b: u64, small_in: u64, large_in: u64, fee_numerator: u64, fee_denominator: u64) -> bool {
    let small_after_fee: u64 = (small_in * (fee_denominator - fee_numerator)) / fee_denominator;
    let small_out: u64 = (reserve_b * small_after_fee) / (reserve_a + small_after_fee);
    let large_after_fee: u64 = (large_in * (fee_denominator - fee_numerator)) / fee_denominator;
    let large_out: u64 = (reserve_b * large_after_fee) / (reserve_a + large_after_fee);
    return large_out > small_out;
}

// Remove liquidity proportional share
// @test-params 100 1000 5000 500
pub test_remove_liquidity_amount_a(lp_amount: u64, lp_supply: u64, reserve_a: u64) -> u64 {
    return (lp_amount * reserve_a) / lp_supply;
}

// @test-params 100 1000 8000 800
pub test_remove_liquidity_amount_b(lp_amount: u64, lp_supply: u64, reserve_b: u64) -> u64 {
    return (lp_amount * reserve_b) / lp_supply;
}

// min_liquidity slippage check on add_liquidity
// @test-params 1200 1000 true
pub test_min_liquidity_guard(liquidity: u64, min_liquidity: u64) -> bool {
    return liquidity >= min_liquidity;
}

// Protocol fee accumulation: after swap, protocol_fees_a increases by protocol portion
// @test-params 1000 1 1000 1
pub test_protocol_fee_accrual(amount_in: u64, protocol_fee_num: u64, fee_denom: u64) -> u64 {
    return (amount_in * protocol_fee_num) / fee_denom;
}
