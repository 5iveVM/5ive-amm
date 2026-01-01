// Test working Option<T> and Result<T,E> types without match expressions

pub test_option_some() -> Option<u64> {
    return Some(42);
}

pub test_option_none() -> Option<u64> {
    return None;
}

pub test_result_ok() -> Result<u64, string> {
    return Ok(100);
}

pub test_result_err() -> Result<u64, string> {
    return Err("Something went wrong");
}

pub test_nested_option() -> Option<Option<u64>> {
    return Some(Some(123));
}

pub test_nested_result() -> Result<Result<u64, string>, string> {
    return Ok(Ok(456));
}