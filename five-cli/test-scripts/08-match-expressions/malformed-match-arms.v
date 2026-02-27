// @should-fail compile
pub test() -> u64 {
    let value = Some(42);
    match value {
        Some(x) => {
            return x;
        }
