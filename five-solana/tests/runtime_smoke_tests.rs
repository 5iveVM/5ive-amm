mod harness;

use five::state::ScriptAccountHeader;
use five_protocol::opcodes::HALT;
use harness::fixtures::canonical_execute_payload;
use harness::{AccountSeed, RuntimeHarness, script_with_header, unique_pubkey};

#[test]
fn deploy_and_execute_halt_script_without_localnet() {
    let program_id = unique_pubkey(1);
    let mut rt = RuntimeHarness::start(program_id);

    let admin = AccountSeed {
        key: unique_pubkey(2),
        owner: program_id,
        lamports: 10_000,
        data: vec![],
        is_signer: true,
        is_writable: true,
        executable: false,
    };
    rt.add_account("admin", admin.clone());

    let vm_state = RuntimeHarness::create_vm_state_seed(program_id);
    rt.add_account("vm_state", vm_state);
    rt.init_vm_state("vm_state", "admin");

    let bytecode = script_with_header(1, 1, &[HALT]);
    let script = RuntimeHarness::create_script_account_seed(program_id, bytecode.len());
    rt.add_account("script", script);

    let deploy = rt.deploy_script("script", "vm_state", "admin", &bytecode, 0, None);
    assert!(deploy.success, "deploy failed: {:?}", deploy.error);

    let payload = canonical_execute_payload(0, &[]);
    let execute = rt.execute_script("script", "vm_state", &["admin"], &payload);
    assert!(execute.success, "execute failed: {:?}", execute.error);

    let script_snapshot = rt.fetch_account("script");
    let header = ScriptAccountHeader::from_account_data(&script_snapshot.data).expect("deployed header");
    assert_eq!(header.bytecode_len(), bytecode.len());
}
