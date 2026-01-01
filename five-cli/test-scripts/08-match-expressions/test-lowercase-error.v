// @should-fail compile
// Test simple function - 'test' is a reserved keyword

pub test() -> u64 {
    return 42;
}