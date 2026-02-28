script main {
    use std::builtins;

    pub builtin_now_seconds() -> u64 {
        return builtins::now_seconds();
    }
}
