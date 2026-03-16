use five_dsl_compiler::DslCompiler;
use five_protocol::{
    opcodes::{BR_EQ_U8_S8, JUMP_IF_NOT_S8, JUMP_S8, LOAD_FIELD_PUBKEY_S, LOAD_FIELD_S, STORE_FIELD_S},
    FEATURE_COMPACT_IMMEDIATES, FIVE_MAGIC,
};

fn compile_source(source: &str) -> Vec<u8> {
    DslCompiler::compile_dsl(source).expect("compile compaction probe")
}

fn header_features(bytecode: &[u8]) -> u32 {
    assert!(bytecode.starts_with(&FIVE_MAGIC));
    u32::from_le_bytes([bytecode[4], bytecode[5], bytecode[6], bytecode[7]])
}

#[test]
fn field_access_uses_compact_field_opcodes_when_offset_fits() {
    let bytecode = compile_source(
        r#"
        script compact_fields {
            account Vault {
                owner: Pubkey,
                amount: u64,
            }

            pub check(vault: Vault @has(owner), owner: Account) {
            }
        }
        "#,
    );

    assert!(
        bytecode.contains(&LOAD_FIELD_S)
            || bytecode.contains(&STORE_FIELD_S)
            || bytecode.contains(&LOAD_FIELD_PUBKEY_S),
        "expected compact field opcode in bytecode",
    );
    assert_ne!(
        header_features(&bytecode) & FEATURE_COMPACT_IMMEDIATES,
        0,
        "expected compact immediate feature bit when compact opcodes are emitted",
    );
}

#[test]
fn counted_loop_uses_short_backward_jumps_when_in_range() {
    let bytecode = compile_source(
        r#"
        script compact_branches {
            pub run() {
                let i: u64 = 0;
                while (i < 3) {
                    i = i + 1;
                }
            }
        }
        "#,
    );

    assert!(
        bytecode.contains(&JUMP_S8) || bytecode.contains(&JUMP_IF_NOT_S8) || bytecode.contains(&BR_EQ_U8_S8),
        "expected short branch opcode in loop bytecode",
    );
}
