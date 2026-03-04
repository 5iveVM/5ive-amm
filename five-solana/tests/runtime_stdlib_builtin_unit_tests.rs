use five::instructions::{deploy, execute};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_dsl_compiler::DslCompiler;
use five_protocol::execute_payload::{canonical_execute_payload, TypedParam};
use pinocchio::{account_info::AccountInfo, pubkey::Pubkey};

fn create_account<'a>(
    key: &'a Pubkey,
    is_signer: bool,
    is_writable: bool,
    lamports: &'a mut u64,
    data: &'a mut [u8],
    owner: &'a Pubkey,
) -> AccountInfo {
    AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, false, 0)
}

fn runtime_program_id() -> Pubkey {
    five::hardcoded_program_id()
}

fn vm_state_pda(program_id: &Pubkey) -> Pubkey {
    five_vm_mito::utils::find_program_address_offchain(&[b"vm_state"], program_id)
        .expect("canonical vm state pda")
        .0
}

fn fee_vault_pda(program_id: &Pubkey) -> Pubkey {
    five_vm_mito::utils::find_program_address_offchain(
        &[b"\xFFfive_vm_fee_vault_v1", &[0u8]],
        program_id,
    )
    .expect("canonical fee vault pda")
    .0
}

#[test]
fn now_seconds_builtin_equivalent_deploys_and_executes_in_runtime_unit() {
    run_stdlib_builtin_case(
        r#"
pub now_seconds() -> u64 {
    return get_clock();
}

pub run() -> u64 {
    return now_seconds();
}
"#,
    );
}

#[test]
fn clock_sysvar_builtin_equivalent_deploys_and_executes_in_runtime_unit() {
    run_stdlib_builtin_case(
        r#"
pub clock_sysvar() {
    get_clock_sysvar();
}

pub now_seconds() -> u64 {
    return get_clock();
}

pub run() -> u64 {
    clock_sysvar();
    return now_seconds();
}
"#,
    );
}

fn run_stdlib_builtin_case(source: &str) {
    let program_id = runtime_program_id();
    let vm_key = vm_state_pda(&program_id);
    let fee_vault_key = fee_vault_pda(&program_id);
    let owner_key = Pubkey::from([7u8; 32]);
    let script_key = Pubkey::from([8u8; 32]);
    let system_key = Pubkey::default();

    let bytecode = DslCompiler::compile_dsl(source).expect("compile stdlib-equivalent builtin");

    let mut script_lamports = 2_000_000;
    let mut vm_lamports = 2_000_000;
    let mut fee_vault_lamports = 0u64;
    let mut owner_lamports = 2_000_000;
    let mut system_lamports = 1u64;

    let mut script_data = vec![0u8; ScriptAccountHeader::LEN + bytecode.len()];
    let mut vm_data = vec![0u8; FIVEVMState::LEN];
    let mut fee_vault_data = [];
    let mut owner_data = [];
    let mut system_data = [];

    {
        let state = FIVEVMState::from_account_data_mut(&mut vm_data).expect("vm state layout");
        state.initialize(owner_key, 255);
        state.deploy_fee_lamports = 0;
        state.execute_fee_lamports = 0;
    }

    let script = create_account(
        &script_key,
        false,
        true,
        &mut script_lamports,
        script_data.as_mut_slice(),
        &program_id,
    );
    let vm = create_account(
        &vm_key,
        false,
        true,
        &mut vm_lamports,
        vm_data.as_mut_slice(),
        &program_id,
    );
    let owner = create_account(
        &owner_key,
        true,
        true,
        &mut owner_lamports,
        &mut owner_data,
        &system_key,
    );
    let fee_vault = create_account(
        &fee_vault_key,
        false,
        true,
        &mut fee_vault_lamports,
        &mut fee_vault_data,
        &program_id,
    );
    let system_program = create_account(
        &system_key,
        false,
        false,
        &mut system_lamports,
        &mut system_data,
        &system_key,
    );

    let deploy_accounts = [
        script.clone(),
        vm.clone(),
        owner.clone(),
        fee_vault.clone(),
        system_program.clone(),
    ];
    deploy(&program_id, &deploy_accounts, &bytecode, &[], 0, 0).expect("deploy builtin probe");

    let payload = canonical_execute_payload(0, &[] as &[TypedParam]);
    let execute_accounts = [script, vm, owner, fee_vault, system_program];
    execute(&program_id, &execute_accounts, &payload).expect("execute builtin probe");
}
