use five_dsl_compiler::bytecode_generator::types::{ABIField, ABIFunction, FIVEABI};
use five_dsl_compiler::FiveFile;
use five_vm_mito::error::VMError;

fn make_fn(i: u8) -> ABIFunction {
    ABIFunction {
        name: format!("f{}", i),
        index: i,
        parameters: Vec::new(),
        return_type: None,
        is_public: true,
        bytecode_offset: 0,
    }
}

fn make_field(i: u64) -> ABIField {
    ABIField {
        name: format!("f{}", i),
        field_type: "u64".to_string(),
        is_mutable: false,
        memory_offset: i,
    }
}

#[test]
fn serialize_abi_errors_on_too_many_functions() {
    let mut functions = Vec::new();
    for i in 0..=64 {
        functions.push(make_fn(i as u8));
    }
    let abi = FIVEABI {
        program_name: "p".to_string(),
        functions,
        fields: Vec::new(),
        version: "1.0".to_string(),
    };
    let file = FiveFile::new(abi, Vec::new());
    assert!(matches!(file.to_bytes(), Err(VMError::InvalidScript)));
}

#[test]
fn serialize_abi_errors_on_too_many_fields() {
    let mut fields = Vec::new();
    for i in 0..=64 {
        fields.push(make_field(i));
    }
    let abi = FIVEABI {
        program_name: "p".to_string(),
        functions: Vec::new(),
        fields,
        version: "1.0".to_string(),
    };
    let file = FiveFile::new(abi, Vec::new());
    assert!(matches!(file.to_bytes(), Err(VMError::InvalidScript)));
}
