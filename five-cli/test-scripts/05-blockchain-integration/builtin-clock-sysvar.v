script main {
    use std::builtins;

    pub builtin_clock_sysvar_smoke() -> i64 {
        builtins::clock_sysvar();
        return builtins::now_seconds();
    }
}
