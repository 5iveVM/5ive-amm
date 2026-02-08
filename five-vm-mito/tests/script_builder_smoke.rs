mod support;

use five_protocol::opcodes::*;
use five_protocol::{FIVE_HEADER_OPTIMIZED_SIZE, FIVE_MAGIC};
use five_vm_mito::{FIVE_VM_PROGRAM_ID, MitoVM, Value, stack::StackStorage};
use support::script_builder::{FunctionVisibility, ScriptBuilder, ScriptBuilderError};

#[test]
fn builder_patches_call_addresses() -> Result<(), ScriptBuilderError> {
    let mut builder = ScriptBuilder::new();
    builder
        .public_function("main", |f| {
            f.push_u64(5).push_u64(7).call("add", 2).return_value();
        })?
        .private_function("add", |f| {
            f.load_param(1).load_param(2).emit(ADD).ret();
        })?;

    let script = builder.build()?;
    let mut storage = StackStorage::new();
    let result = MitoVM::execute_direct(&script, &[], &[], &FIVE_VM_PROGRAM_ID, &mut storage).unwrap();
    assert_eq!(result, Some(Value::U64(12)));
    Ok(())
}

#[test]
fn detects_unknown_call_target() {
    let mut builder = ScriptBuilder::new();
    builder
        .public_function("main", |f| {
            f.call("missing", 0).return_value();
        })
        .unwrap();

    let err = builder.build().unwrap_err();
    assert!(matches!(err, ScriptBuilderError::UnknownFunction(name) if name == "missing"));
}

#[test]
fn rejects_duplicate_names() {
    let mut builder = ScriptBuilder::new();
    builder
        .public_function("main", |f| {
            f.return_value();
        })
        .unwrap();

    let err = builder
        .private_function("main", |f| {
            f.return_value();
        })
        .unwrap_err();

    assert!(matches!(err, ScriptBuilderError::DuplicateFunction(name) if name == "main"));
}

#[test]
fn enforces_public_function_requirement() {
    let mut builder = ScriptBuilder::new();
    builder
        .private_function("helper", |f| {
            f.return_value();
        })
        .unwrap();

    let err = builder.build().unwrap_err();
    assert!(matches!(err, ScriptBuilderError::NoPublicFunctions));
}

#[test]
fn exposes_visibility_enum() {
    // Compile-time check ensures enum is public; no runtime logic needed.
    let _visibility = FunctionVisibility::Public;
}

#[test]
fn exercise_build_script_and_features() -> Result<(), ScriptBuilderError> {
    // Test the static build_script helper and feature setting methods.
    let script = ScriptBuilder::build_script(|builder| {
        // Set features to a non-zero value and add a function.
        builder
            .set_features(42)
            .public_function("main", |f| {
                f.return_value();
            })
            .expect("public_function should succeed");
    });

    // Verify script is built and features are included in header (V3 header).
    assert_eq!(&script[0..4], &FIVE_MAGIC[..], "script magic mismatch");
    // Features is a u32 LE at bytes 4-7; first byte should be 42
    assert_eq!(script[4], 42, "features byte 0 should be set to 42");
    // Public function count is at index 8 in V3 header
    assert_eq!(script[8], 1, "public function count should be 1");
    Ok(())
}

#[test]
fn exercise_with_features_method() -> Result<(), ScriptBuilderError> {
    // Test the with_features builder method.
    let mut builder = ScriptBuilder::new().with_features(123);
    builder.public_function("entry", |f| {
        f.return_value();
    })?;

    let script = builder.build()?;
    assert_eq!(script[4], 123, "features should be set via with_features");
    Ok(())
}

#[test]
fn exercise_function_builder_methods() -> Result<(), ScriptBuilderError> {
    // Test various FunctionBuilder methods that were previously unused.
    let script = ScriptBuilder::build_script(|builder| {
        builder
            .public_function("main", |f| {
                // Use various methods to build bytecode.
                f.emit_bytes(&[0xAB, 0xCD]) // emit_bytes
                    .push_u8(42) // push_u8
                    .push_bool(true) // push_bool
                    .push_i64(-12345) // push_i64
                    .load_param(1); // load_param
                                    // Use code_mut to directly modify code (for testing)
                f.code_mut().push(LOAD_PARAM);
                f.code_mut().push(2);
                f.label("loop_start") // label
                    .jump("loop_start") // jump
                    .jump_if("loop_start"); // jump_if
                                            // Use ret instead of return_value for testing
                f.ret(); // ret
                         // Use call_raw for raw call
                f.call_raw(1, 0x100);
                f.halt(); // halt
            })
            .expect("public_function should succeed");
    });

    // Basic check that script was built without error and has expected structure.
    assert!(
        script.len() > FIVE_HEADER_OPTIMIZED_SIZE,
        "script should have bytecode"
    );
    assert_eq!(&script[0..4], &FIVE_MAGIC[..], "script magic mismatch");
    Ok(())
}
