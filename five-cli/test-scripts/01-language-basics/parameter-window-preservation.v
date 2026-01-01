// Test that parameter windows are not corrupted by nested calls (Task 1.3)

pub test_param_preservation() -> u64 {
    return outer_function(10, 20, 30);
}

outer_function(a: u64, b: u64, c: u64) -> u64 {
    // Parameters a=10, b=20, c=30 should remain accessible
    let result = inner_function(a, b);
    // After inner returns, parameters should still be intact
    return result + c;  // Should be 30 + 30 = 60
}

inner_function(x: u64, y: u64) -> u64 {
    // Different parameters x and y in different frame
    return x + y;  // 10 + 20 = 30
}
