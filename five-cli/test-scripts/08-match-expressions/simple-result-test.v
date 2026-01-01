// Test simple Result match expressions

pub test_result() -> u64 {
    let value = Ok(100);
    match value {
        Ok(x) => {
            return x;
        }
        Err(msg) => {
            return 0;
        }
    }
}