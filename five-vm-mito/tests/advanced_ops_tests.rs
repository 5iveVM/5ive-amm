//! Advanced Operations Tests
//!
//! Tests for advanced opcodes that are not yet implemented or have special behavior.

use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value, VMError};

fn execute_script(bytecode: &[u8]) -> Result<Option<Value>, VMError> {
    MitoVM::execute_direct(bytecode, &[], &[], &FIVE_VM_PROGRAM_ID)
}

#[test]
fn test_bulk_load_field_n_unimplemented() {
    let bytecode = vec![
        0x35, 0x49, 0x56, 0x45, 0, 0, 0, 0, 0, 0,
        0xF7, // BULK_LOAD_FIELD_N
        0x00, // HALT
    ];
    let result = execute_script(&bytecode);
    match result {
        Err(VMError::InvalidInstruction) => {},
        _ => panic!("Expected InvalidInstruction for BULK_LOAD_FIELD_N"),
    }
}
