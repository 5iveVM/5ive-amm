// Test tuple destructuring with return values

pub test() -> (u64, u64) {
    let a = 10;
    let b = 20;
    let (result1, result2) = (a + b, a * b);
    return (result1, result2);
}

pub process_tuple_result() -> u64 {
    let (x, y) = (5, 3);
    let sum = x + y;
    return sum * 2;
}