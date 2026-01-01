add(a: u64, b: u64) -> u64 {
    return a + b;
}

multiply(a: u64, b: u64) -> u64 {
    return a * b;
}

pub test() -> u64 {
    let sum = add(5, 3);
    let product = multiply(4, 2);
    return sum + product;
}
