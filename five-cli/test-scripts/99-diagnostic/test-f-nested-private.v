// Test with private nested functions (like the failing language-basics tests)
pub outer() -> u64 {
    return helper();
}

// This is PRIVATE (no pub keyword)
helper() -> u64 {
    return 42;
}
