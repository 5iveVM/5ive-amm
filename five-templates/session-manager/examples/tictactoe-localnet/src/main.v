// Tic-tac-toe sessionized move flow using sidecar Session PDA.

account MatchState {
    player1: pubkey;
    player2: pubkey;
    current_turn: u64;
    session_nonce: u64;
    ttt_p1_bits: u64;
    ttt_p2_bits: u64;
}

account Session {
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

pub play_ttt(
    match_state: MatchState @mut,
    authority: account,
    delegate: account @signer,
    session: Session @session(delegate, authority, target_program, scope_hash, match_state, session_nonce, current_slot, manager_script_account, manager_code_hash, manager_version),
    target_program: pubkey,
    scope_hash: u64,
    session_nonce: u64,
    current_slot: u64,
    manager_script_account: pubkey,
    manager_code_hash: pubkey,
    manager_version: u8,
    cell_index: u64
) {
    require(session.status == 1);
    require(cell_index < 9);
    match_state.session_nonce = match_state.session_nonce + 1;
}
