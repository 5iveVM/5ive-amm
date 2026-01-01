// Test Option type matching with different patterns

pub test() -> u64 {
    let input = Some(15);
    match input {
        Some(value) => {
            return value * 2;
        }
        None => {
            return 0;
        }
    }
}