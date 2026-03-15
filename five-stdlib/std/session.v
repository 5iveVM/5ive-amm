// std::session
// Canonical delegated session sidecar account (v1).

pub account Session {
    authority: pubkey;
    delegate: pubkey;
    target_program: pubkey;
    expires_at_slot: u64;
    scope_hash: u64;
    nonce: u64;
    bind_account: pubkey;
    manager_script_account: pubkey;
    manager_code_hash: pubkey;
    manager_version: u8;
    status: u8;
    version: u8;
}
