// Minimal test case: single function call
add(a: u64, b: u64) -> u64 {
    return a + b;
}

pub test() -> u64 {
    return add(5, 3);
}