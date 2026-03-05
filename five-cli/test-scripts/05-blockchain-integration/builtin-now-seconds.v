script main {
    use std::builtins;

    pub builtin_now_seconds() -> i64 {
        return builtins::now_seconds();
    }
}
