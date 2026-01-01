// Test simple match expression parsing

pub test_match() -> u64 {
    let value = Some(42);
    match value {
        Some(x) => {
            return x;
        }
        None => {
            return 0;
        }
    }
}