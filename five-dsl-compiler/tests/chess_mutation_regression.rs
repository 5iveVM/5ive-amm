use five_dsl_compiler::DslCompiler;
use five_protocol::opcodes;

fn collect_store_fields(bytecode: &[u8]) -> Vec<(u8, u32)> {
    let mut stores = Vec::new();
    let mut i = 0usize;
    while i < bytecode.len() {
        if bytecode[i] == opcodes::STORE_FIELD {
            if i + 5 < bytecode.len() {
                let account_index = bytecode[i + 1];
                let field_offset = u32::from_le_bytes([
                    bytecode[i + 2],
                    bytecode[i + 3],
                    bytecode[i + 4],
                    bytecode[i + 5],
                ]);
                stores.push((account_index, field_offset));
            }
            i += 6;
            continue;
        }

        // Forward progress for non-target opcodes.
        i += 1;
    }
    stores
}

#[test]
fn nested_mutation_helpers_keep_account_mapping() {
    let source = r#"
account Run {
    white_pawns: u64;
    black_pawns: u64;
    move_count: u64;
}

fn set_white(run: Run @mut, value: u64) {
    run.white_pawns = value;
}

fn advance(run: Run @mut) {
    set_white(run, 11);
    run.black_pawns = run.white_pawns + 1;
    run.move_count = run.black_pawns;
}

pub submit_like(player: account @signer, run: Run @mut) {
    require(player.ctx.key != run.ctx.key);
    advance(run);
}
"#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile nested helper mutation");
    let stores = collect_store_fields(&bytecode);
    assert!(
        !stores.is_empty(),
        "expected STORE_FIELD emissions for nested mutating helpers"
    );

    // Account stores must target account params, never script account index 0.
    assert!(
        stores.iter().all(|(acc, _)| *acc != 0),
        "unexpected script-account stores in nested helper path: {:?}",
        stores
    );
}

#[test]
fn tuple_account_assignment_uses_account_index_mapping() {
    let source = r#"
account Run {
    white_pawns: u64;
    black_pawns: u64;
}

pub tuple_write(run: Run @mut) {
    (run.white_pawns, run.black_pawns) = (5, 7);
}
"#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile tuple account assignment");
    let stores = collect_store_fields(&bytecode);
    assert!(
        stores.len() >= 2,
        "expected at least two STORE_FIELD ops for tuple field assignment, got {:?}",
        stores
    );

    // run is the first account param; mapped account index must not be script account 0.
    assert!(
        stores.iter().all(|(acc, _)| *acc != 0),
        "tuple field assignment wrote to script account index 0: {:?}",
        stores
    );
}

#[test]
fn signer_turn_guard_compiles_with_expected_constraint_opcodes() {
    let source = r#"
account Run {
    white_player: pubkey;
    black_player: pubkey;
    turn: u64;
}

pub guard(player: account @signer, run: Run @mut) {
    require(run.turn == 0 || run.turn == 1);
    if (run.turn == 0) {
        require(player.ctx.key == run.white_player);
    } else {
        require(player.ctx.key == run.black_player);
    }
}
"#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile signer/turn guards");

    assert!(
        bytecode.contains(&opcodes::CHECK_SIGNER),
        "expected CHECK_SIGNER opcode in signer-guarded instruction"
    );
    assert!(
        bytecode.contains(&opcodes::LOAD_FIELD_PUBKEY),
        "expected LOAD_FIELD_PUBKEY for pubkey field guard checks"
    );
}
