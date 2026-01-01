// @test-params 255 15
pub test(a: u64, b: u64) -> u64 {
    let and_result = a & b;
    let or_result = a | b;
    let xor_result = a ^ b;
    let not_result = ~b;
    return (and_result + or_result) - xor_result;
}
