// Square root approximation
pub fn sqrt(value: u64) -> u64 {
    if (value == 0) { return 0; }
    if (value < 4) { return 1; }
    
    let mut z = value;
    let mut x = value / 2 + 1;
    
    while (x < z) {
        z = x;
        x = (value / x + x) / 2;
    }
    
    return z;
}

// Calculate Constant Product Invariant (K)
pub fn calculate_k(reserve_a: u64, reserve_b: u64) -> u64 {
    return reserve_a * reserve_b;
}

// Calculate Quote (Amount B out for Amount A in) - No Fees
pub fn get_quote(amount_a_in: u64, reserve_a: u64, reserve_b: u64) -> u64 {
    if (amount_a_in == 0 || reserve_a == 0 || reserve_b == 0) {
        return 0;
    }
    
    // amount_out = (amount_in * reserve_out) / (reserve_in + amount_in)
    let numerator = amount_a_in * reserve_b;
    let denominator = reserve_a + amount_a_in;
    
    return numerator / denominator;
}
