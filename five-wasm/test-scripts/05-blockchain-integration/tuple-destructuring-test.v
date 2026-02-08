// Basic operations test

pub test() -> u64 {
    let a = 5;
    let b = 3;
    let result = a + b;
    if (result > 100) {
        return result * 2;
    } else {
        return result + 10;
    }
}
