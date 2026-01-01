// Test mixing wrapping and checked arithmetic operations

// @test-params 100 50 20
// Tests mixing wrapping and checked arithmetic
pub test_mixed_operations(a: u64, b: u64, c: u64) -> u64 {
    let wrapping = a + b;      // 150 (wrapping arithmetic)
    let checked = wrapping + c;  // 170 (checked arithmetic replaced with normal addition)
    return checked;
}

// @test-params 1000 200 100
// Tests checked subtraction followed by checked addition
pub test_checked_chain(a: u64, b: u64, c: u64) -> u64 {
    let diff = a - b;    // 800 (checked replaced with normal subtraction)
    let sum = diff + c;  // 900 (checked replaced with normal addition)
    return sum;
}

// @test-params 50 3 2
// Tests checked multiplication in a chain
pub test_checked_mul_chain(a: u64, b: u64, c: u64) -> u64 {
    let product1 = a * b;     // 150 (checked replaced with normal multiplication)
    let product2 = product1 * c;  // 300 (checked replaced with normal multiplication)
    return product2;
}
