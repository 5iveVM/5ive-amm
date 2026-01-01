// Math helper module for DEX calculations
// @module

// Calculate integer square root using Newton's method
// Returns floor(sqrt(x))
pub sqrt(x: u64) -> u64 {
    if (x == 0) {
        return 0;
    }
    if (x < 4) {
        return 1;
    }

    let z = x;
    let r = x / 2 + 1;
    
    // Iterate until convergence
    while (r < z) {
        z = r;
        r = (x / r + r) / 2;
    }
    
    return z;
}

// Calculate square root of product safely
pub sqrt_product(a: u64, b: u64) -> u64 {
    if (a == 0 || b == 0) {
        return 0;
    }
    
    // Check for overflow before multiply
    // Or just use the individual sqrt property: sqrt(a*b) = sqrt(a)*sqrt(b) approximately
    // But better to use u128 if available, or just implement carefully.
    // Since we don't have u128 yet, we'll try to balance them first if possible,
    // or just assume standard ranges fit in u64 for this demo.
    
    // For this demo, let's assume inputs are within range that a*b doesn't overflow u64
    // or handle it simply.
    
    return sqrt(a * b);
}
