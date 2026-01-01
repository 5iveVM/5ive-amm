// Minimal test script for function dispatch
mut value: u64;

init {
    value = 0;
}

pub set_value(new_value: u64) -> u64 {
    value = new_value;
    return value;
}

pub get_value() -> u64 {
    return value;
}
