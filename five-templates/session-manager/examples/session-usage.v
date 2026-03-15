// Example usage in a game script.

account GameState {
    authority: pubkey;
    session_nonce: u64;
}

pub play_move(
    game: GameState @mut,
    authority: account @session
) {
    // Existing game logic follows after compiler-emitted session checks.
    game.session_nonce = game.session_nonce + 1;
}
