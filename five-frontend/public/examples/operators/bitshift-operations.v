// @test-params 16 4
pub test(value: u64, shift_amount: u64) -> u64 {
    let left_shift = value << shift_amount;
    let right_shift = value >> 2;
    return left_shift + right_shift;
}
