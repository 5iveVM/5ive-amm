// Simplified test without Result types

pub test() -> u64 {
    let a = 10;
    let b = 2;
    if (b == 0) {
        return 0;
    } else {
        return a / b;
    }
}