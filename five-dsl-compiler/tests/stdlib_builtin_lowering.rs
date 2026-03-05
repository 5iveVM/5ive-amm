use five_dsl_compiler::DslCompiler;
use five_protocol::{
    opcodes::{CALL_NATIVE, GET_CLOCK},
    FIVE_MAGIC,
};

fn compile_source(source: &str) -> Vec<u8> {
    DslCompiler::compile_dsl(source).expect("compile lowering probe")
}

#[test]
fn now_seconds_lowers_to_get_clock_opcode() {
    let bytecode = compile_source("pub fn run() -> u64 { return get_clock().slot; }");
    assert!(bytecode.starts_with(&FIVE_MAGIC));
    assert!(
        bytecode.contains(&GET_CLOCK),
        "expected GET_CLOCK opcode in now_seconds wrapper bytecode"
    );
}

#[test]
fn memory_copy_lowers_to_native_call_path() {
    let bytecode =
        compile_source("pub fn run(dst: u64, src: u64, len: u64) { memcpy(dst, src, len); }");
    assert!(bytecode.starts_with(&FIVE_MAGIC));
    assert!(
        bytecode.contains(&CALL_NATIVE),
        "expected CALL_NATIVE in memory_copy wrapper bytecode"
    );
}

#[test]
fn return_data_set_lowers_to_native_call_path() {
    let bytecode = compile_source("pub fn run(data: u64) { set_return_data(data); }");
    assert!(bytecode.starts_with(&FIVE_MAGIC));
    assert!(
        bytecode.contains(&CALL_NATIVE),
        "expected CALL_NATIVE in return_data_set wrapper bytecode"
    );
}

#[test]
fn clock_sysvar_lowers_to_native_call_path() {
    let bytecode = compile_source("pub fn run() { get_clock_sysvar(); }");
    assert!(bytecode.starts_with(&FIVE_MAGIC));
    assert!(
        bytecode.contains(&CALL_NATIVE),
        "expected CALL_NATIVE in clock_sysvar wrapper bytecode"
    );
}
