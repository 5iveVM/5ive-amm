// Complex script optimized for nibble immediate opcodes
// Uses locals 0-3 extensively with multiple function calls and parameters

pub calculate_compound_interest(principal: u64, rate: u64, periods: u64) -> u64 {
    let base = principal;           // local 0 -> SET_LOCAL_0 (1 byte)
    let growth_rate = rate;         // local 1 -> SET_LOCAL_1 (1 byte) 
    let time_periods = periods;     // local 2 -> SET_LOCAL_2 (1 byte)
    let result = base;              // local 3 -> SET_LOCAL_3 (1 byte)
    
    // Heavy use of locals 0-3 in calculations (all nibble immediate)
    let temp1 = base + growth_rate;         // GET_LOCAL_0 + GET_LOCAL_1 (2 bytes)
    let temp2 = growth_rate * time_periods; // GET_LOCAL_1 * GET_LOCAL_2 (2 bytes)
    let temp3 = base * temp2;               // GET_LOCAL_0 * temp2 (GET_LOCAL_0 = 1 byte)
    let final_calc = temp3 + result;        // temp3 + GET_LOCAL_3 (1 byte)
    
    return final_calc;
}

pub calculate_portfolio_value(stock_price: u64, bond_price: u64, crypto_price: u64) -> u64 {
    let price1 = stock_price;       // local 0 -> SET_LOCAL_0 (1 byte)
    let price2 = bond_price;        // local 1 -> SET_LOCAL_1 (1 byte)
    let price3 = crypto_price;      // local 2 -> SET_LOCAL_2 (1 byte) 
    let total = 0;                  // local 3 -> SET_LOCAL_3 (1 byte)
    
    // Multiple calculations using nibble immediates
    let weighted1 = price1 * 50;    // GET_LOCAL_0 * 50 (1 byte)
    let weighted2 = price2 * 30;    // GET_LOCAL_1 * 30 (1 byte)
    let weighted3 = price3 * 20;    // GET_LOCAL_2 * 20 (1 byte)
    let sum = weighted1 + weighted2 + weighted3;
    
    return sum + total;             // sum + GET_LOCAL_3 (1 byte)
}

pub main_calculation(input1: u64, input2: u64, input3: u64) -> u64 {
    // Function calls with parameters - all using nibble immediate locals
    let result1 = calculate_compound_interest(input1, input2, input3);
    let result2 = calculate_portfolio_value(input1, input2, input3);
    
    let final_a = result1;          // local 0 -> SET_LOCAL_0 (1 byte)
    let final_b = result2;          // local 1 -> SET_LOCAL_1 (1 byte)
    let multiplier = 2;             // local 2 -> SET_LOCAL_2 (1 byte)
    let bonus = 100;                // local 3 -> SET_LOCAL_3 (1 byte)
    
    // Heavy local variable usage
    let scaled_a = final_a * multiplier;    // GET_LOCAL_0 * GET_LOCAL_2 (2 bytes)
    let scaled_b = final_b * multiplier;    // GET_LOCAL_1 * GET_LOCAL_2 (2 bytes)  
    let combined = scaled_a + scaled_b + bonus; // GET_LOCAL_0 + GET_LOCAL_1 + GET_LOCAL_3 (3 bytes)
    
    return combined;
}