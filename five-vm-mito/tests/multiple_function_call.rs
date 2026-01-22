mod support;

use five_protocol::opcodes::*;
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value, stack::StackStorage};
use support::script_builder::ScriptBuilder;

#[test]
fn test_multiple_function_calls() {
    let mut builder = ScriptBuilder::new();
    builder
        .public_function("main", |f| {
            f.push_u64(5)
                .push_u64(3)
                .call("add", 2)
                .push_u64(4)
                .push_u64(2)
                .call("multiply", 2)
                .emit(ADD)
                .return_value();
        })
        .unwrap();
    builder
        .private_function("add", |f| {
            f.load_param(1).load_param(2).emit(ADD).return_value();
        })
        .unwrap();
    builder
        .private_function("multiply", |f| {
            f.load_param(1).load_param(2).emit(MUL).return_value();
        })
        .unwrap();

    let script = builder.build().expect("valid script");
    let mut storage = StackStorage::new(&script);
    let result = MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID, &mut storage).unwrap();
    assert_eq!(result, Some(Value::U64(16)), "(5+3)+(4*2) should equal 16");
}
