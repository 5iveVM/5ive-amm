#![cfg(feature = "validator-harness")]

mod harness;

use five::state::ScriptAccountHeader;
use five_dsl_compiler::DslCompiler;
use harness::fixtures::canonical_execute_payload;
use harness::validator::{
    build_deploy_instruction, build_execute_instruction_with_extras, RuntimeAccount, ValidatorHarness,
};
use ed25519_dalek::{Keypair as DalekKeypair, Signer as DalekSigner};
use solana_sdk::{
    ed25519_instruction::new_ed25519_instruction,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Signature, Signer},
    system_program,
    sysvar,
};
use std::collections::BTreeMap;

fn bytes_literal(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| b.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn build_probe_source(message: &[u8], signature: &[u8], expect_valid: bool) -> String {
    let message_literal = bytes_literal(message);
    let signature_literal = bytes_literal(signature);
    let branch = if expect_valid {
        r#"
    if ok {
        return 1;
    }
    require(false);
    return 0;
"#
    } else {
        r#"
    if ok {
        require(false);
    }
    return 1;
"#
    };

    format!(
        r#"
pub crypto_probe(owner: account @signer, instruction_sysvar: account) -> u64 {{
    let message: [u8; {message_len}] = [{message_literal}];
    let signature: [u8; 64] = [{signature_literal}];

    let ok = verify_ed25519_instruction(
        instruction_sysvar,
        owner.ctx.key,
        message,
        signature
    );
{branch}
}}
"#,
        message_len = message.len(),
        message_literal = message_literal,
        signature_literal = signature_literal,
        branch = branch
    )
}

fn deploy_with_chunk_fallback(
    h: &ValidatorHarness,
    accounts: &BTreeMap<String, RuntimeAccount>,
    bytecode: &[u8],
) -> (Signature, u64) {
    let direct = h.send_ixs(
        "crypto_probe_deploy",
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
    let err = direct.err().unwrap_or_else(|| "direct deploy failed".to_string());
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
        .send_ixs("crypto_probe_init_large", vec![init_ix], vec![], Some(1_400_000))
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
            .send_ixs("crypto_probe_append", vec![append_ix], vec![], Some(1_400_000))
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
            "crypto_probe_finalize",
            vec![finalize_ix],
            vec![],
            Some(1_400_000),
        )
        .expect("finalize deploy");
    total_cu = total_cu.saturating_add(finalize.units_consumed);
    last_sig = Some(finalize.signature);

    (
        last_sig.expect("chunked deploy signature"),
        total_cu,
    )
}

fn execute_crypto_probe(
    h: &ValidatorHarness,
    vm_state: Pubkey,
    source: &str,
    proof_ixs: Vec<Instruction>,
    label: &str,
) -> (Signature, u64, Signature, u64) {
    let bytecode = DslCompiler::compile_dsl(source).expect("compile probe dsl");

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
        "instruction_sysvar".to_string(),
        RuntimeAccount {
            pubkey: sysvar::instructions::id(),
            signer: None,
            owner: sysvar::id(),
            lamports: 0,
            data_len: 0,
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );

    let (deploy_signature, deploy_cu) = deploy_with_chunk_fallback(h, &accounts, &bytecode);

    let execute_ix = build_execute_instruction_with_extras(
        h.program_id,
        &accounts,
        "script",
        "vm_state",
        &["payer".to_string(), "instruction_sysvar".to_string()],
        canonical_execute_payload(0, &[]),
    );

    let mut ixs = proof_ixs;
    ixs.push(execute_ix);
    let execute = h.send_ixs(label, ixs, vec![], None).expect("execute tx");

    (
        deploy_signature,
        deploy_cu,
        execute.signature,
        execute.units_consumed,
    )
}

#[test]
#[ignore = "requires running validator and pre-deployed program"]
fn validator_crypto_probe_onchain() {
    let h = match ValidatorHarness::from_env() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("SKIP validator_crypto_probe_onchain: {}", e);
            return;
        }
    };

    let (vm_state, vm_bump) = Pubkey::find_program_address(&[b"vm_state"], &h.program_id);
    let vm_state_ready = h
        .rpc
        .get_account(&vm_state)
        .ok()
        .map(|acc| acc.owner == h.program_id && acc.data.len() > 80 && acc.data[80] != 0)
        .unwrap_or(false);
    if !vm_state_ready {
        let init_vm_ix = Instruction {
            program_id: h.program_id,
            accounts: vec![
                AccountMeta::new(vm_state, false),
                AccountMeta::new_readonly(h.payer.pubkey(), true),
                AccountMeta::new(h.payer.pubkey(), true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![0u8, vm_bump],
        };
        let init_res = h.send_ixs("initialize_vm_state_direct", vec![init_vm_ix], vec![], None);
        if let Err(err) = init_res {
            if !err.contains("Custom(1023)") && !err.contains("already initialized") {
                panic!("initialize vm_state: {}", err);
            }
        }
    }

    let fee_shard_index = 0u8;
    let (fee_vault, fee_bump) = Pubkey::find_program_address(
        &[b"\xFFfive_vm_fee_vault_v1", &[fee_shard_index]],
        &h.program_id,
    );
    if h.rpc.get_account(&fee_vault).is_err() {
        let init_fee_vault_ix = Instruction {
            program_id: h.program_id,
            accounts: vec![
                AccountMeta::new(vm_state, false),
                AccountMeta::new(h.payer.pubkey(), true),
                AccountMeta::new(fee_vault, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: vec![11u8, fee_shard_index, fee_bump],
        };
        h.send_ixs(
            "initialize_fee_vault_direct",
            vec![init_fee_vault_ix],
            vec![],
            None,
        )
        .expect("initialize fee vault");
    }

    let message = b"five-crypto-probe-ed25519-msg-v1";
    let dalek = DalekKeypair::from_bytes(&h.payer.to_bytes()).expect("payer keypair bytes");
    let signature = dalek.sign(message).to_bytes();
    let positive_source = build_probe_source(message, &signature, true);
    let ed25519_ix = new_ed25519_instruction(&dalek, message);
    let (deploy_signature, deploy_cu, execute_signature, execute_cu) = execute_crypto_probe(
        &h,
        vm_state,
        &positive_source,
        vec![ed25519_ix],
        "crypto_probe_execute_valid",
    );

    let mut bad_signature = signature;
    bad_signature[0] ^= 0x80;
    let negative_source = build_probe_source(message, &bad_signature, false);
    let (_, _, bad_execute_signature, bad_execute_cu) = execute_crypto_probe(
        &h,
        vm_state,
        &negative_source,
        vec![new_ed25519_instruction(&dalek, message)],
        "crypto_probe_execute_invalid",
    );

    println!(
        "CRYPTO_PROBE valid_deploy_signature={} valid_deploy_cu={} valid_execute_signature={} valid_execute_cu={} invalid_execute_signature={} invalid_execute_cu={}",
        deploy_signature,
        deploy_cu,
        execute_signature,
        execute_cu,
        bad_execute_signature,
        bad_execute_cu
    );
}
