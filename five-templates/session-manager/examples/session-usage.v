// Example usage in a game script.

account GameState {
    authority: pubkey;
    session_nonce: u64;
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

pub play_move(
    game: GameState @mut,
    authority: account,
    delegate: account,
    session: Session @session(delegate=delegate, authority=authority, target_program=target_program, scope_hash=scope_hash, bind_account=game, nonce_field=session_nonce, current_slot=current_slot),
    target_program: pubkey,
    scope_hash: u64,
    session_nonce: u64,
    current_slot: u64
) {
    // Existing game logic follows after compiler-emitted session checks.
    game.session_nonce = game.session_nonce + 1;
}
