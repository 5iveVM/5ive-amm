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
    status: u8;
    version: u8;
}

pub play_ttt(
    match_state: MatchState @mut,
    authority: account,
    delegate: account @signer,
    session: Session @session(delegate=delegate, authority=authority, target_program=target_program, scope_hash=scope_hash, bind_account=match_state, nonce_field=session_nonce, current_slot=current_slot),
    target_program: pubkey,
    scope_hash: u64,
    session_nonce: u64,
    current_slot: u64,
    cell_index: u64
) {
    // Game logic omitted for brevity; delegated path is validated by @session.
    require(cell_index < 9);
    match_state.session_nonce = match_state.session_nonce + 1;
}
