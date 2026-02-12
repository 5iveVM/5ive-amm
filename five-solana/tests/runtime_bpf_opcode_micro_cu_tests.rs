mod harness;

use std::{
    collections::BTreeMap,
    path::PathBuf,
};

use five::instructions::{DEPLOY_INSTRUCTION, EXECUTE_INSTRUCTION};
use five::state::{FIVEVMState, ScriptAccountHeader};
use five_dsl_compiler::DslCompiler;
use five_protocol::opcodes::{
    self, ADD, HALT, POP, PUSH_1, PUSH_2, REQUIRE_PARAM_GT_ZERO, SET_LOCAL_0,
    SUB, MUL, DIV, AND, OR, NOT, BITWISE_AND, SHIFT_LEFT,
    DUP, SWAP, GET_LOCAL_0, DEC_JUMP_NZ, CMP_EQ_JUMP,
    REQUIRE, MUL_CHECKED, MUL_DIV, PUSH_U8, PUSH_U16, PUSH_U64,
    LOAD, STORE,
};
use harness::fixtures::{canonical_execute_payload, TypedParam};
use harness::perf::{assert_no_regression, print_bench_line, CuMetrics};
use solana_program_test::{ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    compute_budget::ComputeBudgetInstruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
    transaction::Transaction,
};

#[derive(Debug)]
struct RuntimeAccount {
    pubkey: Pubkey,
    signer: Option<Keypair>,
    owner: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
}

struct TxOutcome {
    success: bool,
    units_consumed: u64,
    error: Option<String>,
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_stack_add_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((4 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_1);
        body.push(PUSH_2);
        body.push(ADD);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_stack_add_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("stack_arithmetic", "ADD", "hot_loop", &metrics);
    assert_no_regression("opcode_stack_add_loop", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_fused_require_param_gt_zero_bpf_cu() {
    let program_id = load_program_id();
    let script = harness::script_with_header(1, 1, &[REQUIRE_PARAM_GT_ZERO, 1, HALT]);
    let payload = canonical_execute_payload(0, &[TypedParam::U64(42)]);
    let metrics = run_single_script_case(
        program_id,
        "opcode_fused_require_param_gt_zero",
        script,
        payload,
    )
    .await;

    print_bench_line("fused", "REQUIRE_PARAM_GT_ZERO", "single", &metrics);
    assert_no_regression("opcode_fused_require_param_gt_zero", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_nibble_set_local_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((2 * 96) + 2);
    for _ in 0..96 {
        body.push(PUSH_1);
        body.push(SET_LOCAL_0);
    }
    body.push(PUSH_1);
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_nibble_set_local_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("locals", "SET_LOCAL_0", "hot_loop", &metrics);
    assert_no_regression("opcode_nibble_set_local_loop", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_call_external_cold_and_hot_bpf_cu() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);

    let program_id = read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run cargo-build-sbf --manifest-path five-solana/Cargo.toml")
        .pubkey();

    let token_bytecode_path = repo_root.join("five-templates/token/src/token.bin");
    let token_bytecode = harness::compile::load_or_compile_bytecode(&token_bytecode_path)
        .expect("token bytecode should be readable for micro external benchmark");

    let mut accounts = base_accounts(program_id, 40_000_000);

    let token_script_pubkey = Pubkey::new_unique();
    accounts.insert(
        "token_script".to_string(),
        RuntimeAccount {
            pubkey: token_script_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + token_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + token_bytecode.len()],
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let token_import_address = bs58::encode(token_script_pubkey.to_bytes()).into_string();
    let caller_source = format!(
        r#"
        use "{token_import_address}"::{{transfer}};

        pub fn call_transfer(
            source_account: account @mut,
            destination_account: account @mut,
            owner: account @mut,
            ext0: account
        ) {{
            transfer(source_account, destination_account, owner, 1);
        }}
    "#
    );
    let caller_bytecode = DslCompiler::compile_dsl(&caller_source).expect("caller script compile");
    accounts.insert(
        "caller_script".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + caller_bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + caller_bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let owner_pubkey = accounts["owner"].pubkey;
    let mint_pubkey = Pubkey::new_unique();
    let source_token_pubkey = Pubkey::new_unique();
    let destination_token_pubkey = Pubkey::new_unique();

    let mut source_data = vec![0u8; 192];
    source_data[0..32].copy_from_slice(owner_pubkey.as_ref());
    source_data[32..64].copy_from_slice(mint_pubkey.as_ref());
    source_data[64..72].copy_from_slice(&500u64.to_le_bytes());
    source_data[72] = 0;
    accounts.insert(
        "source_token".to_string(),
        RuntimeAccount {
            pubkey: source_token_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(source_data.len()),
            data: source_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mut destination_data = vec![0u8; 192];
    destination_data[0..32].copy_from_slice(destination_token_pubkey.as_ref());
    destination_data[32..64].copy_from_slice(mint_pubkey.as_ref());
    destination_data[64..72].copy_from_slice(&100u64.to_le_bytes());
    destination_data[72] = 0;
    accounts.insert(
        "destination_token".to_string(),
        RuntimeAccount {
            pubkey: destination_token_pubkey,
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(destination_data.len()),
            data: destination_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mut ctx = start_context(program_id, &accounts).await;

    let deploy_token_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "token_script",
        "vm_state",
        "owner",
        &token_bytecode,
    );
    let deploy_token = simulate_and_process(
        &mut ctx,
        vec![deploy_token_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_token.success, "token deploy failed: {:?}", deploy_token.error);

    let deploy_caller_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        "owner",
        &caller_bytecode,
    );
    let deploy_caller = simulate_and_process(
        &mut ctx,
        vec![deploy_caller_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy_caller.success, "caller deploy failed: {:?}", deploy_caller.error);

    let payload = canonical_execute_payload(
        0,
        &[
            TypedParam::Account(1),
            TypedParam::Account(2),
            TypedParam::Account(3),
            TypedParam::Account(4),
        ],
    );
    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "caller_script",
        "vm_state",
        &["source_token", "destination_token", "owner", "token_script"],
        payload,
    );

    let execute_cold = simulate_and_process(
        &mut ctx,
        vec![execute_ix.clone()],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(execute_cold.success, "cold execute failed: {:?}", execute_cold.error);

    let execute_hot = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(execute_hot.success, "hot execute failed: {:?}", execute_hot.error);

    let deploy_units = deploy_token.units_consumed.saturating_add(deploy_caller.units_consumed);

    let cold = CuMetrics {
        deploy: deploy_units,
        execute: execute_cold.units_consumed,
        total: deploy_units.saturating_add(execute_cold.units_consumed),
    };
    let hot = CuMetrics {
        deploy: deploy_units,
        execute: execute_hot.units_consumed,
        total: deploy_units.saturating_add(execute_hot.units_consumed),
    };

    print_bench_line("function", "CALL_EXTERNAL", "cold", &cold);
    print_bench_line("function", "CALL_EXTERNAL", "hot", &hot);
    assert_no_regression("opcode_call_external_cold", &cold);
    assert_no_regression("opcode_call_external_hot", &hot);
}

// ===== ARITHMETIC FAMILY BENCHMARKS =====

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_arithmetic_sub_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((4 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_2);
        body.push(PUSH_1);
        body.push(SUB);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_arithmetic_sub_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("arithmetic", "SUB", "hot_loop", &metrics);
    assert_no_regression("opcode_arithmetic_sub_loop", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_arithmetic_mul_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((4 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_2);
        body.push(PUSH_1);
        body.push(MUL);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_arithmetic_mul_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("arithmetic", "MUL", "hot_loop", &metrics);
    assert_no_regression("opcode_arithmetic_mul_loop", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_arithmetic_div_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((4 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_U8);
        body.push(2);
        body.push(PUSH_U8);
        body.push(100);
        body.push(DIV);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_arithmetic_div_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("arithmetic", "DIV", "hot_loop", &metrics);
    assert_no_regression("opcode_arithmetic_div_loop", &metrics);
}

// Note: MUL_CHECKED opcode requires validated state and is tested via DSL compilation.

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_arithmetic_mul_div_fused_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((5 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_U8);
        body.push(2);
        body.push(PUSH_U8);
        body.push(100);
        body.push(PUSH_U8);
        body.push(50);
        body.push(MUL_DIV);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_arithmetic_mul_div_fused_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("arithmetic", "MUL_DIV", "hot_loop", &metrics);
    assert_no_regression("opcode_arithmetic_mul_div_fused_loop", &metrics);
}

// ===== LOGICAL FAMILY BENCHMARKS =====

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_logical_and_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((4 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_1);
        body.push(PUSH_1);
        body.push(AND);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_logical_and_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("logical", "AND", "hot_loop", &metrics);
    assert_no_regression("opcode_logical_and_loop", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_logical_or_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((4 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_1);
        body.push(PUSH_2);
        body.push(OR);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_logical_or_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("logical", "OR", "hot_loop", &metrics);
    assert_no_regression("opcode_logical_or_loop", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_logical_not_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((3 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_1);
        body.push(NOT);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_logical_not_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("logical", "NOT", "hot_loop", &metrics);
    assert_no_regression("opcode_logical_not_loop", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_logical_bitwise_and_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((4 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_U8);
        body.push(0xFF);
        body.push(PUSH_U8);
        body.push(0x0F);
        body.push(BITWISE_AND);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_logical_bitwise_and_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("logical", "BITWISE_AND", "hot_loop", &metrics);
    assert_no_regression("opcode_logical_bitwise_and_loop", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_logical_shift_left_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((4 * 96) + 1);
    for _ in 0..96 {
        body.push(PUSH_U8);
        body.push(1);
        body.push(PUSH_U8);
        body.push(100);
        body.push(SHIFT_LEFT);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_logical_shift_left_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("logical", "SHIFT_LEFT", "hot_loop", &metrics);
    assert_no_regression("opcode_logical_shift_left_loop", &metrics);
}

// ===== CONTROL FLOW BENCHMARKS =====
// Note: Complex branching opcodes (DEC_JUMP_NZ, CMP_EQ_JUMP) require careful bytecode construction
// and are better tested via DSL compilation. These are tested in scenario tests.

// Note: REQUIRE hot_loop needs proper error handling setup and is tested via DSL compilation.

// ===== MEMORY FAMILY BENCHMARKS =====
// Note: Memory operations (LOAD, STORE, LOAD_FIELD, STORE_FIELD) are complex and require
// proper bytecode/context setup. These are better tested via DSL compilation and are
// measured in scenario tests.

// ===== STACK OPERATIONS BENCHMARKS =====

// Note: DUP and SWAP opcodes require more complex bytecode setup with proper stack management.
// These are better tested via DSL compilation patterns and are measured through scenario tests.

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_locals_get_local_0_hot_loop_bpf_cu() {
    let program_id = load_program_id();
    let mut body = Vec::with_capacity((3 * 96) + 2);
    body.push(PUSH_1);
    body.push(SET_LOCAL_0);
    for _ in 0..96 {
        body.push(GET_LOCAL_0);
        body.push(POP);
    }
    body.push(HALT);

    let script = harness::script_with_header(1, 1, &body);
    let metrics = run_single_script_case(
        program_id,
        "opcode_locals_get_local_0_loop",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("locals", "GET_LOCAL_0", "hot_loop", &metrics);
    assert_no_regression("opcode_locals_get_local_0_loop", &metrics);
}

// ===== COLD SINGLE VARIANT BENCHMARKS =====

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_arithmetic_add_cold_single_bpf_cu() {
    let program_id = load_program_id();
    let script = harness::script_with_header(1, 1, &[PUSH_1, PUSH_2, ADD, POP, HALT]);
    let metrics = run_single_script_case(
        program_id,
        "opcode_arithmetic_add_single",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("arithmetic", "ADD", "cold_single", &metrics);
    assert_no_regression("opcode_arithmetic_add_single", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_arithmetic_div_cold_single_bpf_cu() {
    let program_id = load_program_id();
    let script = harness::script_with_header(1, 1, &[PUSH_U8, 2, PUSH_U8, 100, DIV, POP, HALT]);
    let metrics = run_single_script_case(
        program_id,
        "opcode_arithmetic_div_single",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("arithmetic", "DIV", "cold_single", &metrics);
    assert_no_regression("opcode_arithmetic_div_single", &metrics);
}

#[tokio::test(flavor = "multi_thread")]
async fn opcode_micro_control_require_cold_single_bpf_cu() {
    let program_id = load_program_id();
    let script = harness::script_with_header(1, 1, &[PUSH_1, REQUIRE, HALT]);
    let metrics = run_single_script_case(
        program_id,
        "opcode_control_require_single",
        script,
        canonical_execute_payload(0, &[]),
    )
    .await;

    print_bench_line("control", "REQUIRE", "cold_single", &metrics);
    assert_no_regression("opcode_control_require_single", &metrics);
}

#[test]
fn opcode_family_manifest_covers_all_valid_opcodes() {
    for opcode in 0u8..=u8::MAX {
        if opcodes::is_valid_opcode(opcode) {
            assert!(
                family_for_opcode(opcode).is_some(),
                "missing family mapping for opcode {} ({})",
                opcode,
                opcodes::opcode_name(opcode)
            );
        }
    }
}

fn family_for_opcode(opcode: u8) -> Option<&'static str> {
    match opcode {
        0x00..=0x0F => Some("control"),
        0x10..=0x1F => Some("stack"),
        0x20..=0x2F => Some("arithmetic"),
        0x30..=0x3F => Some("logical"),
        0x40..=0x4F => Some("memory"),
        0x50..=0x5F => Some("account"),
        0x60..=0x6F => Some("array_string"),
        0x70..=0x7F => Some("constraint"),
        0x80..=0x8F => Some("system"),
        0x90..=0x9F => Some("function"),
        0xA0..=0xAF => Some("locals"),
        0xB0..=0xB8 => Some("stack"),
        0xC0..=0xCF => Some("fused"),
        0xD0..=0xDF => Some("nibble"),
        0xE0..=0xEF => Some("fused"),
        0xF0..=0xFF => Some("advanced"),
        _ => None,
    }
}

async fn run_single_script_case(
    program_id: Pubkey,
    test_name: &str,
    bytecode: Vec<u8>,
    payload: Vec<u8>,
) -> CuMetrics {
    let mut accounts = base_accounts(program_id, 20_000_000);
    accounts.insert(
        "script".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(ScriptAccountHeader::LEN + bytecode.len()),
            data: vec![0u8; ScriptAccountHeader::LEN + bytecode.len()],
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    let mut ctx = start_context(program_id, &accounts).await;

    let deploy_ix = build_deploy_instruction(
        program_id,
        &accounts,
        "script",
        "vm_state",
        "owner",
        &bytecode,
    );
    let deploy = simulate_and_process(
        &mut ctx,
        vec![deploy_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(deploy.success, "{} deploy failed: {:?}", test_name, deploy.error);

    let execute_ix = build_execute_instruction(
        program_id,
        &accounts,
        "script",
        "vm_state",
        &["owner"],
        payload,
    );
    let execute = simulate_and_process(
        &mut ctx,
        vec![execute_ix],
        collect_signers(&accounts, &["owner"]),
        Some(1_400_000),
    )
    .await;
    assert!(execute.success, "{} execute failed: {:?}", test_name, execute.error);

    CuMetrics {
        deploy: deploy.units_consumed,
        execute: execute.units_consumed,
        total: deploy.units_consumed.saturating_add(execute.units_consumed),
    }
}

fn load_program_id() -> Pubkey {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let bpf_dir = repo_root.join("target/deploy");
    std::env::set_var("BPF_OUT_DIR", &bpf_dir);
    read_keypair_file(bpf_dir.join("five-keypair.json"))
        .expect("missing target/deploy/five-keypair.json; run cargo-build-sbf --manifest-path five-solana/Cargo.toml")
        .pubkey()
}

fn base_accounts(program_id: Pubkey, owner_lamports: u64) -> BTreeMap<String, RuntimeAccount> {
    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();
    let owner_signer = Keypair::new();
    let owner_pubkey = owner_signer.pubkey();
    accounts.insert(
        "owner".to_string(),
        RuntimeAccount {
            pubkey: owner_pubkey,
            signer: Some(owner_signer),
            owner: system_program::id(),
            lamports: owner_lamports,
            data: vec![],
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );

    let mut vm_state_data = vec![0u8; FIVEVMState::LEN];
    {
        let vm_state = FIVEVMState::from_account_data_mut(&mut vm_state_data)
            .expect("invalid vm state account layout");
        vm_state.initialize(owner_pubkey.to_bytes());
        vm_state.deploy_fee_lamports = 0;
        vm_state.execute_fee_lamports = 0;
    }
    accounts.insert(
        "vm_state".to_string(),
        RuntimeAccount {
            pubkey: Pubkey::new_unique(),
            signer: None,
            owner: program_id,
            lamports: Rent::default().minimum_balance(FIVEVMState::LEN),
            data: vm_state_data,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );

    accounts
}

async fn start_context(program_id: Pubkey, accounts: &BTreeMap<String, RuntimeAccount>) -> ProgramTestContext {
    let mut program_test = ProgramTest::new("five", program_id, None);
    program_test.prefer_bpf(true);

    for account in accounts.values() {
        if account.pubkey == program_id || account.pubkey == system_program::id() {
            continue;
        }
        program_test.add_account(
            account.pubkey,
            Account {
                lamports: account.lamports,
                data: account.data.clone(),
                owner: account.owner,
                executable: account.executable,
                rent_epoch: 0,
            },
        );
    }

    program_test.start_with_context().await
}

fn build_deploy_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    vm_state_name: &str,
    owner_name: &str,
    bytecode: &[u8],
) -> Instruction {
    let mut data = Vec::with_capacity(10 + bytecode.len());
    data.push(DEPLOY_INSTRUCTION);
    data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    data.push(0);
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(bytecode);

    Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(accounts[script_name].pubkey, false),
            AccountMeta::new(accounts[vm_state_name].pubkey, false),
            AccountMeta::new_readonly(accounts[owner_name].pubkey, true),
        ],
        data,
    }
}

fn build_execute_instruction(
    program_id: Pubkey,
    accounts: &BTreeMap<String, RuntimeAccount>,
    script_name: &str,
    vm_state_name: &str,
    extras: &[&str],
    payload: Vec<u8>,
) -> Instruction {
    let mut data = Vec::with_capacity(1 + payload.len());
    data.push(EXECUTE_INSTRUCTION);
    data.extend_from_slice(&payload);

    let mut metas = vec![
        AccountMeta::new(accounts[script_name].pubkey, false),
        AccountMeta::new(accounts[vm_state_name].pubkey, false),
    ];
    for name in extras {
        let account = &accounts[*name];
        let is_external_script = *name != script_name && name.ends_with("_script");
        metas.push(AccountMeta {
            pubkey: account.pubkey,
            is_signer: account.is_signer,
            is_writable: if is_external_script { false } else { account.is_writable },
        });
    }

    Instruction {
        program_id,
        accounts: metas,
        data,
    }
}

fn collect_signers<'a>(accounts: &'a BTreeMap<String, RuntimeAccount>, names: &[&str]) -> Vec<&'a Keypair> {
    let mut out = Vec::new();
    for name in names {
        if let Some(kp) = accounts[*name].signer.as_ref() {
            out.push(kp);
        }
    }
    out
}

async fn simulate_and_process(
    ctx: &mut ProgramTestContext,
    instructions: Vec<Instruction>,
    extra_signers: Vec<&Keypair>,
    cu_limit: Option<u32>,
) -> TxOutcome {
    let mut all_instructions = Vec::with_capacity(instructions.len() + 1);
    if let Some(limit) = cu_limit {
        all_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(limit));
    }
    all_instructions.extend(instructions);

    let mut signers: Vec<&Keypair> = Vec::with_capacity(1 + extra_signers.len());
    signers.push(&ctx.payer);
    signers.extend(extra_signers);

    let tx = Transaction::new_signed_with_payer(
        &all_instructions,
        Some(&ctx.payer.pubkey()),
        &signers,
        ctx.last_blockhash,
    );

    let simulation = ctx.banks_client.simulate_transaction(tx.clone()).await;
    let (simulated_units, sim_logs) = match simulation {
        Ok(sim_result) => {
            let units = sim_result
                .simulation_details
                .as_ref()
                .map(|d| d.units_consumed)
                .unwrap_or(0);
            let logs = sim_result
                .simulation_details
                .as_ref()
                .map(|d| d.logs.clone())
                .unwrap_or_default();
            (units, logs)
        }
        Err(err) => {
            return TxOutcome {
                success: false,
                units_consumed: 0,
                error: Some(format!("simulate failed: {}", err)),
            }
        }
    };

    match ctx.banks_client.process_transaction(tx).await {
        Ok(()) => TxOutcome {
            success: true,
            units_consumed: simulated_units,
            error: None,
        },
        Err(err) => {
            for log in &sim_logs {
                println!("SIM_LOG {}", log);
            }
            TxOutcome {
                success: false,
                units_consumed: simulated_units,
                error: Some(err.to_string()),
            }
        }
    }
}
