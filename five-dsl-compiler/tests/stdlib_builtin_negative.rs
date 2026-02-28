use five_dsl_compiler::DslCompiler;

#[test]
fn invalid_memcpy_arity_fails_compilation() {
    let source = "pub fn run(dst: u64, src: u64, len: u64) { memcpy(dst, src); }";
    assert!(DslCompiler::compile_dsl(source).is_err());
}

#[test]
fn invalid_set_return_data_arity_fails_compilation() {
    let source = "pub fn run(data: u64) { set_return_data(); }";
    assert!(DslCompiler::compile_dsl(source).is_err());
}

#[test]
fn invalid_memcmp_arity_fails_compilation() {
    let source = "pub fn run(a: u64, b: u64, len: u64) { memcmp(a, b); }";
    assert!(DslCompiler::compile_dsl(source).is_err());
}

#[test]
fn invalid_close_account_arity_fails_compilation() {
    let source = "pub fn run(source: account, destination: account) { close_account(source); }";
    assert!(DslCompiler::compile_dsl(source).is_err());
}
