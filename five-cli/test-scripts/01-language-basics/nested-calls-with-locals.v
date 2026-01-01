// Test local variables at each level of nested calls
// Validates that locals remain accessible after nested calls return

pub test_nested_with_locals() -> u64 {
    let base = 1;
    return level1(base);  // Expected: (((1+1)*2+2)*2+3)*2+4 = 28
}

level1(val: u64) -> u64 {
    let local1 = val + 1;  // 2
    let result = level2(local1);
    return result * 2 + local1;  // local1 should still be 2
}

level2(val: u64) -> u64 {
    let local2 = val + 2;  // 4
    let result = level3(local2);
    return result * 2 + local2;  // local2 should still be 4
}

level3(val: u64) -> u64 {
    let local3 = val + 3;  // 7
    let result = level4(local3);
    return result * 2 + local3;  // local3 should still be 7
}

level4(val: u64) -> u64 {
    let local4 = val + 4;  // 11
    return local4;
}
