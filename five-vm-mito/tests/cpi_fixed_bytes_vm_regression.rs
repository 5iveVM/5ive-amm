use five_dsl_compiler::DslCompiler;
use five_vm_mito::{
    stack::StackStorage, utils::find_program_address_offchain, AccountInfo, MitoVM,
    FIVE_VM_PROGRAM_ID,
};
use pinocchio::pubkey::Pubkey;

const MEMO_PROGRAM_ID: [u8; 32] = [
    5, 74, 83, 90, 153, 41, 33, 6, 77, 36, 232, 113, 96, 218, 56, 124, 124, 53, 181, 221, 188, 146,
    187, 129, 228, 31, 168, 64, 65, 5, 68, 141,
];

fn build_memo_cpi_source() -> String {
    let memo_bytes = [
        102u8, 105, 118, 101, 45, 99, 112, 105, 45, 112, 114, 111, 98, 101, 45, 102, 105, 120, 101,
        100, 45, 98, 121, 116, 101, 115, 45, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 45, 65, 66,
        67, 68, 69, 70, 45, 71, 72, 73, 74, 75, 76, 45, 77, 78, 79, 80, 81, 82, 45, 83, 84, 85, 86,
        87,
    ];
    let memo_literal = memo_bytes
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"
interface MemoProgram @program("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr") @serializer(raw) {{
    write @discriminator_bytes([]) (memo: [u8; 64]);
}}

pub cpi_memo(memo_program: account) -> u64 {{
    MemoProgram::write([{memo_literal}]);
    return 1;
}}
"#,
        memo_literal = memo_literal
    )
}

fn build_memo_signer_cpi_source() -> String {
    let memo_bytes = [
        102u8, 105, 118, 101, 45, 99, 112, 105, 45, 115, 105, 103, 110, 101, 114, 45, 112, 114,
        111, 98, 101, 45, 102, 105, 120, 101, 100, 45, 98, 121, 116, 101, 115, 45, 48, 49, 50, 51,
        52, 53, 54, 55, 56, 57, 45, 65, 66, 67, 68, 69, 70, 45, 71, 72, 73, 74, 75, 76, 45, 77, 78,
        79, 80, 81,
    ];
    let memo_literal = memo_bytes
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"
interface MemoProgram @program("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr") @serializer(raw) {{
    write @discriminator_bytes([]) (authority: account, memo: [u8; 64]);
}}

pub cpi_memo_with_signer(memo_program: account, authority: account) -> u64 {{
    MemoProgram::write(authority, [{memo_literal}]);
    return 1;
}}
"#,
        memo_literal = memo_literal
    )
}

fn build_memo_auto_pda_cpi_source() -> String {
    let memo_bytes = [
        102u8, 105, 118, 101, 45, 99, 112, 105, 45, 97, 117, 116, 111, 45, 112, 100, 97, 45, 112,
        114, 111, 98, 101, 45, 102, 105, 120, 101, 100, 45, 98, 121, 116, 101, 115, 45, 48, 49, 50,
        51, 52, 53, 54, 55, 56, 57, 45, 65, 66, 67, 68, 69, 70, 45, 71, 72, 73, 74, 75, 76, 45, 77,
        78, 79,
    ];
    let memo_literal = memo_bytes
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"
interface MemoProgram @program("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr") @serializer(raw) {{
    write @discriminator_bytes([]) (authority: account @authority, memo: [u8; 64]);
}}

pub cpi_memo_auto(vm_state: account @pda(seeds=["vm_state"]), memo_program: account) -> u64 {{
    MemoProgram::write(vm_state, [{memo_literal}]);
    return 1;
}}
"#,
        memo_literal = memo_literal
    )
}

fn account_info(
    key: &'static Pubkey,
    owner: &'static Pubkey,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
) -> AccountInfo {
    let lamports = Box::leak(Box::new(1_000_000u64));
    let data = Box::leak(vec![0u8; 0].into_boxed_slice());
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
fn cpi_fixed_bytes_vm_regression_matches_execute_account_layout() {
    let source = build_memo_cpi_source();
    let bytecode = DslCompiler::compile_dsl(&source).expect("compile cpi memo");

    let vm_state = Box::leak(Box::new(Pubkey::from([2u8; 32])));
    let memo_program = Box::leak(Box::new(Pubkey::from(MEMO_PROGRAM_ID)));
    let payer = Box::leak(Box::new(Pubkey::from([7u8; 32])));
    let fee_vault = Box::leak(Box::new(Pubkey::from([8u8; 32])));
    let system = Box::leak(Box::new(Pubkey::from([0u8; 32])));

    let accounts = vec![
        account_info(vm_state, &FIVE_VM_PROGRAM_ID, false, true, false),
        account_info(memo_program, system, false, false, true),
        account_info(payer, system, true, true, false),
        account_info(payer, system, true, true, false),
        account_info(fee_vault, &FIVE_VM_PROGRAM_ID, false, true, false),
        account_info(system, system, false, false, true),
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

    println!("VM_RESULT={:?}", result);
    assert_eq!(result, Ok(Some(five_vm_mito::Value::U64(1))));
}

#[test]
fn cpi_fixed_bytes_with_signer_vm_regression_matches_execute_account_layout() {
    let source = build_memo_signer_cpi_source();
    let bytecode = DslCompiler::compile_dsl(&source).expect("compile cpi memo signer");

    let vm_state = Box::leak(Box::new(Pubkey::from([2u8; 32])));
    let memo_program = Box::leak(Box::new(Pubkey::from(MEMO_PROGRAM_ID)));
    let payer = Box::leak(Box::new(Pubkey::from([7u8; 32])));
    let fee_vault = Box::leak(Box::new(Pubkey::from([8u8; 32])));
    let system = Box::leak(Box::new(Pubkey::from([0u8; 32])));

    let accounts = vec![
        account_info(vm_state, &FIVE_VM_PROGRAM_ID, false, true, false),
        account_info(memo_program, system, false, false, true),
        account_info(payer, system, true, true, false),
        account_info(payer, system, true, true, false),
        account_info(payer, system, true, true, false),
        account_info(fee_vault, &FIVE_VM_PROGRAM_ID, false, true, false),
        account_info(system, system, false, false, true),
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

    println!("VM_SIGNER_RESULT={:?}", result);
    assert_eq!(result, Ok(Some(five_vm_mito::Value::U64(1))));
}

#[test]
fn cpi_fixed_bytes_auto_pda_vm_regression_matches_execute_account_layout() {
    let source = build_memo_auto_pda_cpi_source();
    let bytecode = DslCompiler::compile_dsl(&source).expect("compile cpi memo auto pda");

    let root_script_key = Pubkey::from([9u8; 32]);
    let (vm_state_key, _bump) =
        find_program_address_offchain(&[root_script_key.as_ref(), b"vm_state"], &FIVE_VM_PROGRAM_ID)
            .expect("derive vm_state pda");
    let vm_state = Box::leak(Box::new(vm_state_key));
    let memo_program = Box::leak(Box::new(Pubkey::from(MEMO_PROGRAM_ID)));
    let payer = Box::leak(Box::new(Pubkey::from([7u8; 32])));
    let fee_vault = Box::leak(Box::new(Pubkey::from([8u8; 32])));
    let system = Box::leak(Box::new(Pubkey::from([0u8; 32])));

    let accounts = vec![
        account_info(vm_state, &FIVE_VM_PROGRAM_ID, false, true, false),
        account_info(vm_state, &FIVE_VM_PROGRAM_ID, false, true, false),
        account_info(memo_program, system, false, false, true),
        account_info(payer, system, true, true, false),
        account_info(payer, system, true, true, false),
        account_info(fee_vault, &FIVE_VM_PROGRAM_ID, false, true, false),
        account_info(system, system, false, false, true),
    ];

    let input = canonical_execute_payload(0);
    let mut storage = StackStorage::new();
    let result = MitoVM::execute_direct_with_root_script(
        &bytecode,
        &input,
        &accounts,
        &FIVE_VM_PROGRAM_ID,
        root_script_key,
        &mut storage,
    );
    assert_eq!(result, Ok(Some(five_vm_mito::Value::U64(1))));
}
