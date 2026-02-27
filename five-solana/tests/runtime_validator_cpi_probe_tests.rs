#![cfg(feature = "validator-harness")]

mod harness;

use five::state::ScriptAccountHeader;
use five_dsl_compiler::DslCompiler;
use harness::fixtures::canonical_execute_payload;
use harness::validator::{
    build_deploy_instruction, build_execute_instruction_with_extras, RuntimeAccount,
    ValidatorHarness,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Signature, Signer},
    system_program,
};
use std::collections::BTreeMap;

const MEMO_PROGRAM_ID: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

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
interface MemoProgram @program("{memo_program_id}") @serializer(raw) {{
    write @discriminator_bytes([]) (memo: [u8; 64]);
}}

pub cpi_memo(memo_program: account) -> u64 {{
    MemoProgram.write([{memo_literal}]);
    return 1;
}}
"#,
        memo_program_id = MEMO_PROGRAM_ID,
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
interface MemoProgram @program("{memo_program_id}") @serializer(raw) {{
    write @discriminator_bytes([]) (authority: account, memo: [u8; 64]);
}}

pub cpi_memo_with_signer(memo_program: account, authority: account) -> u64 {{
    MemoProgram.write(authority, [{memo_literal}]);
    return 1;
}}
"#,
        memo_program_id = MEMO_PROGRAM_ID,
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
interface MemoProgram @program("{memo_program_id}") @serializer(raw) {{
    write @discriminator_bytes([]) (authority: account @authority, memo: [u8; 64]);
}}

pub cpi_memo_auto(vm_state: account @pda(seeds=["vm_state"]), memo_program: account) -> u64 {{
    MemoProgram.write(vm_state, [{memo_literal}]);
    return 1;
}}
"#,
        memo_program_id = MEMO_PROGRAM_ID,
        memo_literal = memo_literal
    )
}

fn deploy_with_chunk_fallback(
    h: &ValidatorHarness,
    accounts: &BTreeMap<String, RuntimeAccount>,
    bytecode: &[u8],
) -> (Signature, u64) {
    let direct = h.send_ixs(
        "cpi_probe_deploy",
        vec![build_deploy_instruction(
            h.program_id,
            accounts,
            "script",
            "vm_state",
            "payer",
            bytecode,
            0,
            &[],
        )],
        vec![],
        Some(1_400_000),
    );
    if let Ok(ok) = direct {
        return (ok.signature, ok.units_consumed);
    }
    let err = direct
        .err()
        .unwrap_or_else(|| "direct deploy failed".to_string());
    if !err.contains("too large") {
        panic!("deploy tx: {}", err);
    }

    let fee_shard_index = 0u8;
    let (fee_vault, _fee_bump) = Pubkey::find_program_address(
        &[b"\xFFfive_vm_fee_vault_v1", &[fee_shard_index]],
        &h.program_id,
    );
    let mut total_cu = 0u64;
    let mut last_sig: Option<Signature> = None;

    const INIT_CHUNK: usize = 512;
    const APPEND_CHUNK: usize = 850;

    let first = bytecode.len().min(INIT_CHUNK);
    let mut init_data = Vec::with_capacity(1 + 4 + first);
    init_data.push(4u8);
    init_data.extend_from_slice(&(bytecode.len() as u32).to_le_bytes());
    init_data.extend_from_slice(&bytecode[..first]);
    let init_ix = Instruction {
        program_id: h.program_id,
        accounts: vec![
            AccountMeta::new(accounts["script"].pubkey, false),
            AccountMeta::new_readonly(accounts["payer"].pubkey, true),
            AccountMeta::new(accounts["vm_state"].pubkey, false),
            AccountMeta::new(fee_vault, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: init_data,
    };
    let init = h
        .send_ixs(
            "cpi_probe_init_large",
            vec![init_ix],
            vec![],
            Some(1_400_000),
        )
        .expect("init large deploy");
    total_cu = total_cu.saturating_add(init.units_consumed);
    last_sig = Some(init.signature);

    let mut offset = first;
    while offset < bytecode.len() {
        let end = (offset + APPEND_CHUNK).min(bytecode.len());
        let mut append_data = Vec::with_capacity(1 + (end - offset));
        append_data.push(5u8);
        append_data.extend_from_slice(&bytecode[offset..end]);
        let append_ix = Instruction {
            program_id: h.program_id,
            accounts: vec![
                AccountMeta::new(accounts["script"].pubkey, false),
                AccountMeta::new_readonly(accounts["payer"].pubkey, true),
                AccountMeta::new(accounts["vm_state"].pubkey, false),
                AccountMeta::new(fee_vault, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: append_data,
        };
        let append = h
            .send_ixs("cpi_probe_append", vec![append_ix], vec![], Some(1_400_000))
            .expect("append deploy chunk");
        total_cu = total_cu.saturating_add(append.units_consumed);
        last_sig = Some(append.signature);
        offset = end;
    }

    let finalize_ix = Instruction {
        program_id: h.program_id,
        accounts: vec![
            AccountMeta::new(accounts["script"].pubkey, false),
            AccountMeta::new_readonly(accounts["payer"].pubkey, true),
        ],
        data: vec![7u8],
    };
    let finalize = h
        .send_ixs(
            "cpi_probe_finalize",
            vec![finalize_ix],
            vec![],
            Some(1_400_000),
        )
        .expect("finalize deploy");
    total_cu = total_cu.saturating_add(finalize.units_consumed);
    last_sig = Some(finalize.signature);

    (last_sig.expect("chunked deploy signature"), total_cu)
}

fn run_cpi_probe(
    test_name: &str,
    source: &str,
    extras: &[String],
) -> (Signature, u64, Signature, u64) {
    let h = match ValidatorHarness::from_env() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("SKIP {}: {}", test_name, e);
            return (Signature::default(), 0, Signature::default(), 0);
        }
    };

    let vm_state = h.ensure_vm_state().expect("vm_state ready");
    h.ensure_fee_vault_shard(vm_state, 0)
        .expect("fee vault ready");

    let bytecode = DslCompiler::compile_dsl(source).expect("compile memo cpi probe dsl");

    let script = h
        .create_program_owned_account(
            ScriptAccountHeader::LEN + bytecode.len(),
            h.rent_exempt(ScriptAccountHeader::LEN + bytecode.len())
                .expect("rent exempt"),
            h.program_id,
        )
        .expect("create script account");

    let mut accounts = BTreeMap::<String, RuntimeAccount>::new();
    accounts.insert(
        "script".to_string(),
        RuntimeAccount {
            pubkey: script.pubkey(),
            signer: Some(script),
            owner: h.program_id,
            lamports: 0,
            data_len: ScriptAccountHeader::LEN + bytecode.len(),
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );
    accounts.insert(
        "vm_state".to_string(),
        RuntimeAccount {
            pubkey: vm_state,
            signer: None,
            owner: h.program_id,
            lamports: 0,
            data_len: 0,
            is_signer: false,
            is_writable: true,
            executable: false,
        },
    );
    accounts.insert(
        "payer".to_string(),
        RuntimeAccount {
            pubkey: h.payer.pubkey(),
            signer: None,
            owner: solana_sdk::system_program::id(),
            lamports: 0,
            data_len: 0,
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );
    accounts.insert(
        "memo_program".to_string(),
        RuntimeAccount {
            pubkey: MEMO_PROGRAM_ID.parse().expect("memo program id"),
            signer: None,
            owner: solana_sdk::bpf_loader::id(),
            lamports: 0,
            data_len: 0,
            is_signer: false,
            is_writable: false,
            executable: true,
        },
    );

    let (deploy_signature, deploy_cu) = deploy_with_chunk_fallback(&h, &accounts, &bytecode);
    let execute_ix = build_execute_instruction_with_extras(
        h.program_id,
        &accounts,
        "script",
        "vm_state",
        extras,
        canonical_execute_payload(0, &[]),
    );
    let execute = h
        .send_ixs(test_name, vec![execute_ix], vec![], None)
        .expect("execute cpi probe");

    (
        deploy_signature,
        deploy_cu,
        execute.signature,
        execute.units_consumed,
    )
}

#[test]
#[ignore = "requires running validator and pre-deployed program"]
fn validator_cpi_fixed_bytes_probe_onchain() {
    let source = build_memo_cpi_source();
    let (deploy_signature, deploy_cu, execute_signature, execute_cu) = run_cpi_probe(
        "cpi_probe_execute",
        &source,
        &["memo_program".to_string(), "payer".to_string()],
    );
    if deploy_signature == Signature::default() {
        return;
    }

    println!(
        "CPI_FIXED_BYTES_PROBE deploy_signature={} deploy_cu={} execute_signature={} execute_cu={}",
        deploy_signature, deploy_cu, execute_signature, execute_cu
    );
}

#[test]
#[ignore = "requires running validator and pre-deployed program"]
fn validator_cpi_fixed_bytes_with_signer_probe_onchain() {
    let source = build_memo_signer_cpi_source();
    let (deploy_signature, deploy_cu, execute_signature, execute_cu) = run_cpi_probe(
        "cpi_probe_execute_signer",
        &source,
        &["memo_program".to_string(), "payer".to_string()],
    );
    if deploy_signature == Signature::default() {
        return;
    }

    println!(
        "CPI_FIXED_BYTES_SIGNER_PROBE deploy_signature={} deploy_cu={} execute_signature={} execute_cu={}",
        deploy_signature,
        deploy_cu,
        execute_signature,
        execute_cu
    );
}

#[test]
#[ignore = "requires running validator and pre-deployed program"]
fn validator_cpi_auto_authority_pda_probe_onchain() {
    let source = build_memo_auto_pda_cpi_source();
    let (deploy_signature, deploy_cu, execute_signature, execute_cu) = run_cpi_probe(
        "cpi_probe_execute_auto_pda",
        &source,
        &[
            "vm_state".to_string(),
            "memo_program".to_string(),
            "payer".to_string(),
        ],
    );
    if deploy_signature == Signature::default() {
        return;
    }

    println!(
        "CPI_AUTO_PDA_AUTHORITY_PROBE deploy_signature={} deploy_cu={} execute_signature={} execute_cu={}",
        deploy_signature,
        deploy_cu,
        execute_signature,
        execute_cu
    );
}
