// Complex script that forces VLE usage by using high-index locals
// Uses locals 4+ extensively to avoid nibble immediate optimization

pub calculate_compound_interest(principal: u64, rate: u64, periods: u64) -> u64 {
    let unused0 = 0;                // local 0 (nibble slot unused)
    let unused1 = 0;                // local 1 (nibble slot unused)
    let unused2 = 0;                // local 2 (nibble slot unused) 
    let unused3 = 0;                // local 3 (nibble slot unused)
    
    let base = principal;           // local 4 -> SET_LOCAL 4 (2 bytes)
    let growth_rate = rate;         // local 5 -> SET_LOCAL 5 (2 bytes)
    let time_periods = periods;     // local 6 -> SET_LOCAL 6 (2 bytes)
    let result = base;              // local 7 -> SET_LOCAL 7 (2 bytes)
    
    // Heavy use of locals 4-7 in calculations (all VLE, 2 bytes each)
    let temp1 = base + growth_rate;         // GET_LOCAL 4 + GET_LOCAL 5 (4 bytes)
    let temp2 = growth_rate * time_periods; // GET_LOCAL 5 * GET_LOCAL 6 (4 bytes)
    let temp3 = base * temp2;               // GET_LOCAL 4 * temp2 (GET_LOCAL 4 = 2 bytes)
    let final_calc = temp3 + result;        // temp3 + GET_LOCAL 7 (2 bytes)
    
    return final_calc;
}

pub calculate_portfolio_value(stock_price: u64, bond_price: u64, crypto_price: u64) -> u64 {
    let unused0 = 0;                // local 0 (nibble slot unused)
    let unused1 = 0;                // local 1 (nibble slot unused)
    let unused2 = 0;                // local 2 (nibble slot unused)
    let unused3 = 0;                // local 3 (nibble slot unused)
    
    let price1 = stock_price;       // local 4 -> SET_LOCAL 4 (2 bytes)
    let price2 = bond_price;        // local 5 -> SET_LOCAL 5 (2 bytes)
    let price3 = crypto_price;      // local 6 -> SET_LOCAL 6 (2 bytes)
    let total = 0;                  // local 7 -> SET_LOCAL 7 (2 bytes)
    
    // Multiple calculations using VLE locals
    let weighted1 = price1 * 50;    // GET_LOCAL 4 * 50 (2 bytes)
    let weighted2 = price2 * 30;    // GET_LOCAL 5 * 30 (2 bytes)
    let weighted3 = price3 * 20;    // GET_LOCAL 6 * 20 (2 bytes)
    let sum = weighted1 + weighted2 + weighted3;
    
    return sum + total;             // sum + GET_LOCAL 7 (2 bytes)
}

pub main_calculation(input1: u64, input2: u64, input3: u64) -> u64 {
    // Function calls with parameters - all using VLE locals
    let result1 = calculate_compound_interest(input1, input2, input3);
    let result2 = calculate_portfolio_value(input1, input2, input3);
    
    let unused0 = 0;                // local 0 (nibble slot unused)
    let unused1 = 0;                // local 1 (nibble slot unused)
    let unused2 = 0;                // local 2 (nibble slot unused)
    let unused3 = 0;                // local 3 (nibble slot unused)
    
    let final_a = result1;          // local 4 -> SET_LOCAL 4 (2 bytes)
    let final_b = result2;          // local 5 -> SET_LOCAL 5 (2 bytes)
    let multiplier = 2;             // local 6 -> SET_LOCAL 6 (2 bytes)
    let bonus = 100;                // local 7 -> SET_LOCAL 7 (2 bytes)
    
    // Heavy local variable usage with VLE
    let scaled_a = final_a * multiplier;    // GET_LOCAL 4 * GET_LOCAL 6 (4 bytes)
    let scaled_b = final_b * multiplier;    // GET_LOCAL 5 * GET_LOCAL 6 (4 bytes)
    let combined = scaled_a + scaled_b + bonus; // GET_LOCAL 4 + GET_LOCAL 5 + GET_LOCAL 7 (6 bytes)
    
    return combined;
}