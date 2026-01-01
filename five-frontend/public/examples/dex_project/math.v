// Math helper module for DEX calculations
// @module

pub sqrt(x: u64) -> u64 {
    if (x == 0) { return 0; }
    if (x < 4) { return 1; }
    let z = x;
    let r = x / 2 + 1;
    while (r < z) {
        z = r;
        r = (x / r + r) / 2;
    }
    return z;
}

pub sqrt_product(a: u64, b: u64) -> u64 {
    if (a == 0 || b == 0) { return 0; }
    return sqrt(a * b);
}
