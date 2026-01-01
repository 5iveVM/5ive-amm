// Test basic arithmetic operators: add, subtract, multiply, divide
pub test() -> u64 {
    let a = 10;
    let b = 3;
    let add_result = a + b;
    let sub_result = a - b;
    let mul_result = a * b;
    let div_result = a / b;
    return add_result + sub_result + mul_result + div_result;
}
