use five_dsl_compiler::DslCompiler;
use five_vm_mito::{stack::StackStorage, AccountInfo, MitoVM, VMError, FIVE_VM_PROGRAM_ID};
use pinocchio::pubkey::Pubkey;

fn account_info(
    key: &'static Pubkey,
    owner: &'static Pubkey,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
    data_len: usize,
) -> AccountInfo {
    let lamports = Box::leak(Box::new(1_000_000u64));
    let data = Box::leak(vec![0u8; data_len].into_boxed_slice());
    AccountInfo::new(
        key,
        is_signer,
        is_writable,
        lamports,
        data,
        owner,
        executable,
        0,
    )
}

fn canonical_execute_payload(function_index: u32) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&function_index.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out
}

#[test]
fn nested_chess_style_mutation_executes_without_invalid_account_data() {
    let source = r#"
account Run {
    player: pubkey;
    white_pawns: u64;
    black_pawns: u64;
    white_king: u64;
    move_count: u64;
    tick: u64;
    status: u64;
}

fn set_piece(run: Run @mut, square: u64, present: u64) {
    if (square == 0) {
        run.white_pawns = present;
    } else {
        run.black_pawns = present;
    }
    run.tick = run.tick + 1;
}

fn apply_move_on_board(run: Run @mut) {
    set_piece(run, 0, 1);
    set_piece(run, 1, 1);
    run.white_king = run.white_pawns + run.black_pawns;
}

pub submit_move(run: Run @mut, player: account @signer) {
    require(player.ctx.key != run.ctx.key);
    apply_move_on_board(run);
    run.move_count = run.move_count + run.white_king;
    run.status = 1;
}
"#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile chess mutation fixture");

    let vm_state = Box::leak(Box::new(Pubkey::from([2u8; 32])));
    let player = Box::leak(Box::new(Pubkey::from([7u8; 32])));
    let run = Box::leak(Box::new(Pubkey::from([9u8; 32])));
    let system = Box::leak(Box::new(Pubkey::from([0u8; 32])));

    let accounts = vec![
        account_info(vm_state, &FIVE_VM_PROGRAM_ID, false, true, false, 128),
        account_info(run, &FIVE_VM_PROGRAM_ID, false, true, false, 512),
        account_info(player, system, true, false, false, 0),
    ];

    let input = canonical_execute_payload(0);
    let mut storage = StackStorage::new();
    let result = MitoVM::execute_direct(
        &bytecode,
        &input,
        &accounts,
        &FIVE_VM_PROGRAM_ID,
        &mut storage,
    );

    match result {
        Ok(_) => {}
        Err(err) => {
            assert_ne!(err, VMError::InvalidAccountData, "unexpected 9006 regression");
            assert_ne!(err, VMError::ConstraintViolation, "unexpected 9003 regression");
            panic!("unexpected execution error: {err:?}");
        }
    }
}

#[test]
fn chess_style_constraint_failure_is_explicit() {
    let source = r#"
account Run {
    player: pubkey;
    move_count: u64;
}

pub guarded(player: account @signer, run: Run @mut) {
    require(player.ctx.key == run.player);
    run.move_count = run.move_count + 1;
}
"#;

    let bytecode = DslCompiler::compile_dsl(source).expect("compile constraint fixture");

    let vm_state = Box::leak(Box::new(Pubkey::from([12u8; 32])));
    let player = Box::leak(Box::new(Pubkey::from([13u8; 32])));
    let run = Box::leak(Box::new(Pubkey::from([14u8; 32])));
    let system = Box::leak(Box::new(Pubkey::from([0u8; 32])));

    let accounts = vec![
        account_info(vm_state, &FIVE_VM_PROGRAM_ID, false, true, false, 128),
        account_info(player, system, true, false, false, 0),
        account_info(run, &FIVE_VM_PROGRAM_ID, false, true, false, 256),
    ];

    let input = canonical_execute_payload(0);
    let mut storage = StackStorage::new();
    let result = MitoVM::execute_direct(
        &bytecode,
        &input,
        &accounts,
        &FIVE_VM_PROGRAM_ID,
        &mut storage,
    );

    assert_eq!(result, Err(VMError::ConstraintViolation));
}
