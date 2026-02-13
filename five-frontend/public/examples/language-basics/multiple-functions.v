add(a: u64, b: u64) -> u64 {
    return a + b;
}

multiply(a: u64, b: u64) -> u64 {
    return a * b;
}

// @test-params 5 3 4
pub test(a: u64, b: u64, c: u64) -> u64 {
    let sum = add(a, b);
    let product = multiply(sum, c);
    return sum + product;
}
