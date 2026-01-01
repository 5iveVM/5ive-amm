// Test that local variables are isolated per call frame
// Validates Task 1.2 fix (per-frame local storage isolation)

pub test_local_isolation() -> u64 {
    let result1 = caller_function(5);
    let result2 = caller_function(10);
    return result1 + result2;  // Should be 15 + 30 = 45
}

caller_function(input: u64) -> u64 {
    let local_var = input;        // Stored in caller's frame
    let result = callee_function(local_var);
    return local_var + result;    // local_var should be unchanged
}

callee_function(input: u64) -> u64 {
    let local_var = input * 2;    // Different local_var in callee's frame
    return local_var;             // Should not corrupt caller's local_var
}
