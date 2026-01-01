pub add(a: u64, b: u64) -> u64 {
    return a + b;
}

pub multiply(a: u64, b: u64) -> u64 {
    return a * b;
}

pub test() -> u64 {
    let sum = add(5, 3);
    let product = multiply(4, 2);
    return sum + product;
}

pub get_constant() -> u64 {
    return 42;
}

pub complex_calculation(x: u64, y: u64, z: u64) -> u64 {
    let sum = add(x, y);
    let product = multiply(sum, z);
    return product + get_constant();
}
