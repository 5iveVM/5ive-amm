pub test() -> u64 {
    let a = pop_u64();  // This should cause stack underflow
    return a;
}