// Session Manager (v1)
// Sidecar session accounts created via normal execute flow.

account Session {
    authority: pubkey;
    delegate: pubkey;
    target_program: pubkey;
    expires_at_slot: u64;
    scope_hash: u64;
    nonce: u64;
    bind_account: pubkey;
    status: u8;
    version: u8;
}

pub create_session(
    session: Session @mut,
    authority: account @signer,
    delegate: account,
    target_program: pubkey,
    expires_at_slot: u64,
    scope_hash: u64,
    bind_account: pubkey,
    nonce: u64
) {
    session.authority = authority.ctx.key;
    session.delegate = delegate.ctx.key;
    session.target_program = target_program;
    session.expires_at_slot = expires_at_slot;
    session.scope_hash = scope_hash;
    session.bind_account = bind_account;
    session.nonce = nonce;
    session.status = 1;
    session.version = 1;
}

pub revoke_session(
    session: Session @mut,
    authority: account @signer
) {
    require(session.authority == authority.ctx.key);
    session.status = 0;
}
