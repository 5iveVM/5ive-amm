mod support;

use five_protocol::opcodes::*;
use five_vm_mito::{AccountInfo, FIVE_VM_PROGRAM_ID, MitoVM, Value, stack::StackStorage};
use support::script_builder::ScriptBuilder;

fn build_simple_call_script() -> Vec<u8> {
    let mut builder = ScriptBuilder::new();
    builder
        .public_function("main", |f| {
            f.push_u64(5).push_u64(3).call("add", 2).return_value();
        })
        .unwrap();
    builder
        .private_function("add", |f| {
            f.load_param(1).load_param(2).emit(ADD).return_value();
        })
        .unwrap();

    builder.build().expect("valid script")
}

#[test]
fn test_simple_function_call() {
    let bytecode = build_simple_call_script();
    let accounts: &[AccountInfo] = &[];
    let mut storage = StackStorage::new(&bytecode);
    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &FIVE_VM_PROGRAM_ID, &mut storage).unwrap();
    assert_eq!(
        result,
        Some(Value::U64(8)),
        "add_numbers(5, 3) should return 8"
    );
}

#[test]
fn test_basic_vm_execution() {
    let bytecode = build_simple_call_script();
    let accounts: &[AccountInfo] = &[];
    let mut storage = StackStorage::new(&bytecode);
    let result = MitoVM::execute_direct(&bytecode, &[], accounts, &FIVE_VM_PROGRAM_ID, &mut storage);
    assert!(
        result.is_ok(),
        "script built via ScriptBuilder should execute"
    );
}
