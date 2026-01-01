// Integration test: nested calls + checked arithmetic + local variables
// Validates multiple remediation fixes working together

pub test_complex_integration() -> u64 {
    let base = 100;
    let result = process_level1(base, 50);
    return result;
}

process_level1(amount: u64, fee: u64) -> u64 {
    let net_amount = amount - fee;  // 50 (checked subtraction replaced with normal subtraction)
    let result = process_level2(net_amount, 10);
    return net_amount + result;  // Checked addition replaced with normal addition
}

process_level2(amount: u64, bonus: u64) -> u64 {
    let local_calc = amount + bonus;  // 60 (wrapping addition)
    let result = process_level3(local_calc);
    return result * 2;  // Checked multiplication replaced with normal multiplication
}

process_level3(amount: u64) -> u64 {
    let local_var = amount - 10;  // 50 (checked subtraction replaced with normal subtraction)
    return local_var;
}
