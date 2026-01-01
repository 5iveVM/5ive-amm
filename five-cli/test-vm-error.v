pub test() -> u64 {
    // This should cause a stack underflow error
    pop();
    return 42;
}