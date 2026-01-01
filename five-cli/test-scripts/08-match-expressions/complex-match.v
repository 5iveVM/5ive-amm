// Test complex match expressions with multiple patterns

pub test() -> u64 {
    let value = Some(50);
    match value {
        Some(x) => {
            if (x > 100) {
                return x * 2;
            } else if (x > 10) {
                return x + 50;
            } else {
                return 0;
            }
        }
        None => {
            return 0;
        }
    }
}