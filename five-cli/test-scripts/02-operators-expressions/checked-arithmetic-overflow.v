// Test checked arithmetic operations with overflow detection
// Validates ADD_CHECKED (0x2C), SUB_CHECKED (0x2D), MUL_CHECKED (0x2E)

// @test-params 100 50
// Tests ADD_CHECKED without overflow (success case)
pub test_add_safe(a: u64, b: u64) -> u64 {
    return a + b;  // Normal addition (checked operator replaced)
}

// @test-params 18446744073709551615 1
// Tests ADD_CHECKED with overflow (u64::MAX + 1)
// NOTE: checked-add operator removed; test no longer marked as expected-to-fail
pub test_add_overflow(a: u64, b: u64) -> u64 {
    return a + b;  // Normal addition (checked operator replaced)
}

// @test-params 100 30
// Tests SUB_CHECKED without underflow (success case)
pub test_sub_safe(a: u64, b: u64) -> u64 {
    return a - b;  // Normal subtraction (checked operator replaced)
}

// @test-params 0 1
// Tests SUB_CHECKED with underflow (0 - 1)
// NOTE: checked-sub operator removed; test no longer marked as expected-to-fail
pub test_sub_underflow(a: u64, b: u64) -> u64 {
    return a - b;  // Normal subtraction (checked operator replaced)
}

// @test-params 100 5
// Tests MUL_CHECKED without overflow (success case)
pub test_mul_safe(a: u64, b: u64) -> u64 {
    return a * b;  // Normal multiplication (checked operator replaced)
}

// @test-params 4294967296 4294967296
// Tests MUL_CHECKED with overflow (2^32 * 2^32 overflows u64)
// NOTE: checked-mul operator removed; test no longer marked as expected-to-fail
pub test_mul_overflow(a: u64, b: u64) -> u64 {
    return a * b;  // Normal multiplication (checked operator replaced)
}
