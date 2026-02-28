#![cfg(feature = "validator-harness")]

mod harness;

use five::state::ScriptAccountHeader;
use five_dsl_compiler::{CompilationConfig, CompilationMode, DslCompiler};
use harness::fixtures::{canonical_execute_payload, TypedParam};
use harness::validator::{
    build_deploy_instruction, build_execute_instruction_with_extras, RuntimeAccount,
    ValidatorHarness,
};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Signature, Signer},
    system_program,
};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn compile_probe(source: &str) -> Vec<u8> {
    let probe_dir = {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("five-account-probe-{}", nanos));
        fs::create_dir_all(&dir).expect("create probe dir");
        dir
    };
    let main_path = probe_dir.join("main.v");
    fs::write(&main_path, source).expect("write probe source");
    let config = CompilationConfig::new(CompilationMode::Testing);
    DslCompiler::compile_with_auto_discovery(&main_path, &config)
        .expect("compile account probe")
}

fn execute_account_probe(
    h: &ValidatorHarness,
    label: &str,
    bytecode: &[u8],
    target_owner: Pubkey,
    target_lamports: u64,
    params_builder: impl FnOnce(Pubkey) -> Vec<TypedParam>,
) -> (Signature, u64, Signature, u64) {
    let vm_state = h.ensure_vm_state().expect("vm_state ready");
    h.ensure_fee_vault_shard(vm_state, 0)
        .expect("fee vault ready");

    let script = h
        .create_program_owned_account(
            ScriptAccountHeader::LEN + bytecode.len(),
            h.rent_exempt(ScriptAccountHeader::LEN + bytecode.len())
                .expect("rent exempt"),
            h.program_id,
        )
        .expect("create script account");

    let target = h
        .create_program_owned_account(16, target_lamports, target_owner)
        .expect("create target account");

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
            owner: system_program::id(),
            lamports: 0,
            data_len: 0,
            is_signer: true,
            is_writable: true,
            executable: false,
        },
    );
    accounts.insert(
        "target".to_string(),
        RuntimeAccount {
            pubkey: target.pubkey(),
            signer: Some(target),
            owner: target_owner,
            lamports: target_lamports,
            data_len: 16,
            is_signer: false,
            is_writable: false,
            executable: false,
        },
    );
    let params = params_builder(accounts["target"].pubkey);

    let deploy = h
        .send_ixs(
            &format!("{}_deploy", label),
            vec![build_deploy_instruction(
                h.program_id,
                &accounts,
                "script",
                "vm_state",
                "payer",
                bytecode,
                0,
                &[],
            )],
            vec![],
            Some(1_400_000),
        )
        .expect("deploy account probe");

    let execute = h
        .send_ixs(
            &format!("{}_execute", label),
            vec![build_execute_instruction_with_extras(
                h.program_id,
                &accounts,
                "script",
                "vm_state",
                &["target".to_string()],
                canonical_execute_payload(0, &params),
            )],
            vec![],
            Some(1_400_000),
        )
        .expect("execute account probe");

    (
        deploy.signature,
        deploy.units_consumed,
        execute.signature,
        execute.units_consumed,
    )
}

fn maybe_write_probe_artifact(payload: &str) {
    let Ok(path) = std::env::var("FIVE_CU_PROBE_OUTPUT_PATH") else {
        return;
    };

    let artifact_path = PathBuf::from(path);
    if let Some(parent) = artifact_path.parent() {
        fs::create_dir_all(parent).expect("create account probe artifact parent");
    }
    fs::write(&artifact_path, payload).expect("write account probe artifact");
}

#[test]
#[ignore = "requires running validator and pre-deployed program"]
fn validator_account_probe_onchain() {
    let h = match ValidatorHarness::from_env() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("SKIP validator_account_probe_onchain: {}", e);
            return;
        }
    };

    let target_owner = h.program_id;
    let target_lamports = h.rent_exempt(16).expect("rent exempt target");

    let load_source = r#"
pub run(target: account) -> u64 {
    return load_account_u64_word(target, 0);
}
"#;
    let lamports_source = r#"
pub run(target: account) -> u64 {
    let balance = target.ctx.lamports;
    require(balance == balance);
    return balance;
}
"#;
    let owner_source = r#"
pub run(target: account) -> u64 {
    let owner = target.ctx.owner;
    require(owner == owner);
    return 1;
}
"#;
    let key_source = r#"
pub run(target: account) -> u64 {
    let key = target.ctx.key;
    require(key == key);
    return 1;
}
"#;

    let load_bytecode = compile_probe(load_source);
    let lamports_bytecode = compile_probe(lamports_source);
    let owner_bytecode = compile_probe(owner_source);
    let key_bytecode = compile_probe(key_source);
    let (load_deploy_signature, load_deploy_cu, load_execute_signature, load_execute_cu) = execute_account_probe(
        &h,
        "account_load_u64_word",
        &load_bytecode,
        target_owner,
        target_lamports,
        |_| vec![TypedParam::Account(1)],
    );
    let (
        lamports_deploy_signature,
        lamports_deploy_cu,
        lamports_execute_signature,
        lamports_execute_cu,
    ) = execute_account_probe(
        &h,
        "account_ctx_lamports",
        &lamports_bytecode,
        target_owner,
        target_lamports,
        |_| vec![TypedParam::Account(1)],
    );
    let (owner_deploy_signature, owner_deploy_cu, owner_execute_signature, owner_execute_cu) = execute_account_probe(
        &h,
        "account_ctx_owner",
        &owner_bytecode,
        target_owner,
        target_lamports,
        |_| vec![TypedParam::Account(1)],
    );
    let (key_deploy_signature, key_deploy_cu, key_execute_signature, key_execute_cu) = execute_account_probe(
        &h,
        "account_ctx_key",
        &key_bytecode,
        target_owner,
        target_lamports,
        |_| vec![TypedParam::Account(1)],
    );

    maybe_write_probe_artifact(&format!(
        concat!(
            "{{",
            "\"kind\":\"account\",",
            "\"loadDeploySignature\":\"{}\",",
            "\"loadDeployCu\":{},",
            "\"loadExecuteSignature\":\"{}\",",
            "\"loadExecuteCu\":{},",
            "\"lamportsDeploySignature\":\"{}\",",
            "\"lamportsDeployCu\":{},",
            "\"lamportsExecuteSignature\":\"{}\",",
            "\"lamportsExecuteCu\":{},",
            "\"ownerDeploySignature\":\"{}\",",
            "\"ownerDeployCu\":{},",
            "\"ownerExecuteSignature\":\"{}\",",
            "\"ownerExecuteCu\":{},",
            "\"keyDeploySignature\":\"{}\",",
            "\"keyDeployCu\":{},",
            "\"keyExecuteSignature\":\"{}\",",
            "\"keyExecuteCu\":{}",
            "}}"
        ),
        load_deploy_signature,
        load_deploy_cu,
        load_execute_signature,
        load_execute_cu,
        lamports_deploy_signature,
        lamports_deploy_cu,
        lamports_execute_signature,
        lamports_execute_cu,
        owner_deploy_signature,
        owner_deploy_cu,
        owner_execute_signature,
        owner_execute_cu,
        key_deploy_signature,
        key_deploy_cu,
        key_execute_signature,
        key_execute_cu
    ));

    println!(
        "ACCOUNT_PROBE load_deploy_signature={} load_deploy_cu={} load_execute_signature={} load_execute_cu={} lamports_deploy_signature={} lamports_deploy_cu={} lamports_execute_signature={} lamports_execute_cu={} owner_deploy_signature={} owner_deploy_cu={} owner_execute_signature={} owner_execute_cu={} key_deploy_signature={} key_deploy_cu={} key_execute_signature={} key_execute_cu={}",
        load_deploy_signature,
        load_deploy_cu,
        load_execute_signature,
        load_execute_cu,
        lamports_deploy_signature,
        lamports_deploy_cu,
        lamports_execute_signature,
        lamports_execute_cu,
        owner_deploy_signature,
        owner_deploy_cu,
        owner_execute_signature,
        owner_execute_cu,
        key_deploy_signature,
        key_deploy_cu,
        key_execute_signature,
        key_execute_cu
    );
}
