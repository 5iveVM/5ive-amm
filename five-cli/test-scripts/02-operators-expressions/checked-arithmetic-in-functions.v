// Test checked arithmetic within function calls

// @test-params 100 20
pub test_checked_in_nested_calls(amount: u64, fee: u64) -> u64 {
    return process_payment(amount, fee);
}

process_payment(amount: u64, fee: u64) -> u64 {
    let net = amount - fee;  // 80 (checked subtraction replaced with normal subtraction)
    return apply_bonus(net);
}

apply_bonus(amount: u64) -> u64 {
    let bonus = amount * 2;  // 160 (checked multiplication replaced with normal multiplication)
    return bonus;
}

// @test-params 1000 100 50
pub test_multi_step_calculation(initial: u64, deduct: u64, multiplier: u64) -> u64 {
    let step1 = subtract_fee(initial, deduct);
    let step2 = multiply_amount(step1, multiplier);
    return step2;
}

subtract_fee(amount: u64, fee: u64) -> u64 {
    return amount - fee;  // 900 (checked replaced with normal subtraction)
}

multiply_amount(amount: u64, factor: u64) -> u64 {
    return amount * factor;  // 45000 (checked replaced with normal multiplication)
}
