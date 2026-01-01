// Test case: function call with local variable
add(a: u64, b: u64) -> u64 {
    return a + b;
}

pub test() -> u64 {
    let sum = add(5, 3);
    return sum;
}