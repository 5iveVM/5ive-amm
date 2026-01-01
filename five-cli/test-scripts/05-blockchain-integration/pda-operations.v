// Test multi-parameter PDA derivation
pub find_vault_pda(user: pubkey, seed: u64) -> (pubkey, u8) {
    return derive_pda(user, "vault", seed);
}

// Test two-parameter PDA derivation
pub find_token_pda(mint: pubkey, owner: pubkey) -> (pubkey, u8) {
    return derive_pda("token", mint, owner);
}
