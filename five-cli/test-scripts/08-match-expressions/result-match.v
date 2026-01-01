// Test Result type matching with Ok and Err patterns

pub test() -> u64 {
    let value = Ok(15);
    match value {
        Ok(x) => {
            return x + 10;
        }
        Err(msg) => {
            return 0;
        }
    }
}