// Simple V3 pattern test
add_numbers(a: u64, b: u64) -> u64 {
    return a + b;
}

multiply_numbers(x: u64, y: u64) -> u64 {
    return x * y;
}

pub test() -> u64 {
    // Test function calls with V3 optimizations
    let sum = add_numbers(5, 3);           // Simple function call
    let product = multiply_numbers(4, 2);  // Another function call
    
    // Test arithmetic patterns that could use V3 fusion
    let result = sum + product + 1;        // Could use PUSH_ONE optimization
    return result;
}