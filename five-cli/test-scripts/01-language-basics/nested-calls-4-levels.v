// Test 4-level nested function calls
// Validates call depth and stack management

pub test_nested_4_levels() -> u64 {
    return level1(10);
}

level1(val: u64) -> u64 {
    let local1 = val + 100;  // 110
    return level2(local1);
}

level2(val: u64) -> u64 {
    let local2 = val + 200;  // 310
    return level3(local2);
}

level3(val: u64) -> u64 {
    let local3 = val + 300;  // 610
    return level4(local3);
}

level4(val: u64) -> u64 {
    let local4 = val + 400;  // 1010
    return local4;
}
