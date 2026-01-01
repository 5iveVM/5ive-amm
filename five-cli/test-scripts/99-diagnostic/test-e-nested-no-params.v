// Test E: Nested call with 0 params at inner level (isolate nesting vs params)
pub outer() -> u64 {
    return helper();
}

pub helper() -> u64 {
    return 42;
}
