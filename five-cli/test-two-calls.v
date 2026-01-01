// Test case: two function calls without local variables
add(a: u64, b: u64) -> u64 {
    return a + b;
}

multiply(a: u64, b: u64) -> u64 {
    return a * b;
}

pub test() -> u64 {
    add(5, 3);
    return multiply(4, 2);
}