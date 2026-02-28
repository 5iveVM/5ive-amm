#![cfg(feature = "validator-harness")]

mod harness;

use five::state::ScriptAccountHeader;
use five_dsl_compiler::DslCompiler;
use harness::fixtures::canonical_execute_payload;
use harness::validator::{
    build_deploy_instruction, build_execute_instruction_with_extras, RuntimeAccount,
    ValidatorHarness,
};
use solana_sdk::{signature::Signature, signer::Signer, system_program};
use std::{
    collections::BTreeMap,
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn compile_probe(source: &str) -> Vec<u8> {
    let _probe_dir = {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("five-stdlib-probe-{}", nanos));
        fs::create_dir_all(&dir).expect("create probe dir");
        dir
    };
    DslCompiler::compile_dsl(source).expect("compile stdlib-equivalent probe")
}

fn execute_probe(
    h: &ValidatorHarness,
    label: &str,
    bytecode: &[u8],
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
        .expect("deploy probe");

    let execute = h
        .send_ixs(
            &format!("{}_execute", label),
            vec![build_execute_instruction_with_extras(
                h.program_id,
                &accounts,
                "script",
                "vm_state",
                &["payer".to_string()],
                canonical_execute_payload(0, &[]),
            )],
            vec![],
            Some(1_400_000),
        )
        .expect("execute probe");

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
        fs::create_dir_all(parent).expect("create stdlib probe artifact parent");
    }
    fs::write(&artifact_path, payload).expect("write stdlib probe artifact");
}

#[test]
#[ignore = "requires running validator and pre-deployed program"]
fn validator_stdlib_time_and_sysvar_onchain() {
    let h = match ValidatorHarness::from_env() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("SKIP validator_stdlib_time_and_sysvar_onchain: {}", e);
            return;
        }
    };

    let now_source = r#"
pub now_seconds() -> u64 {
    return get_clock();
}

pub run() -> u64 {
    return now_seconds();
}
"#;
    let clock_source = r#"
pub clock_sysvar() {
    get_clock_sysvar();
}

pub run() -> u64 {
    clock_sysvar();
    return get_clock();
}
"#;

    let now_bytecode = compile_probe(now_source);
    let clock_bytecode = compile_probe(clock_source);

    let (now_deploy_signature, now_deploy_cu, now_execute_signature, now_execute_cu) =
        execute_probe(&h, "stdlib_now_seconds", &now_bytecode);
    let (clock_deploy_signature, clock_deploy_cu, clock_execute_signature, clock_execute_cu) =
        execute_probe(&h, "stdlib_clock_sysvar", &clock_bytecode);

    maybe_write_probe_artifact(&format!(
        concat!(
            "{{",
            "\"kind\":\"stdlib\",",
            "\"nowDeploySignature\":\"{}\",",
            "\"nowDeployCu\":{},",
            "\"nowExecuteSignature\":\"{}\",",
            "\"nowExecuteCu\":{},",
            "\"clockDeploySignature\":\"{}\",",
            "\"clockDeployCu\":{},",
            "\"clockExecuteSignature\":\"{}\",",
            "\"clockExecuteCu\":{}",
            "}}"
        ),
        now_deploy_signature,
        now_deploy_cu,
        now_execute_signature,
        now_execute_cu,
        clock_deploy_signature,
        clock_deploy_cu,
        clock_execute_signature,
        clock_execute_cu
    ));

    println!(
        "STDLIB_PROBE now_deploy_signature={} now_deploy_cu={} now_execute_signature={} now_execute_cu={} clock_deploy_signature={} clock_deploy_cu={} clock_execute_signature={} clock_execute_cu={}",
        now_deploy_signature,
        now_deploy_cu,
        now_execute_signature,
        now_execute_cu,
        clock_deploy_signature,
        clock_deploy_cu,
        clock_execute_signature,
        clock_execute_cu
    );
}
