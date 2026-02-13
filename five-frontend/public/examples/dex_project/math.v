// Math helper module for DEX calculations
// @module

pub square(x: u64) -> u64 {
    return x * x;
}

pub min(a: u64, b: u64) -> u64 {
    if (a < b) {
        return a;
    }
    return b;
}

pub sqrt_product(a: u64, b: u64) -> u64 {
    require(a > 0);
    require(b > 0);
    return min(a, b);
}
