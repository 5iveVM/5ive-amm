// Test RETURN opcode with local_base restoration (Task 1.4)
// Validates that local variables remain accessible after nested returns

pub test_return_nested() -> u64 {
    let outer_local = 100;
    let result = middle_function(10);
    return outer_local + result;  // outer_local should still be 100
}

middle_function(input: u64) -> u64 {
    let middle_local = input + 20;  // 30
    let result = inner_function(middle_local);
    // middle_local should still be 30 after inner returns
    return middle_local + result;
}

inner_function(input: u64) -> u64 {
    let inner_local = input + 30;  // 60
    return inner_local;  // RETURN should restore local_base correctly
}
