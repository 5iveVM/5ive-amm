// Find PDA and return both address and bump seed
pub find_vault_pda(user: pubkey, seed: u64) -> (pubkey, u8) {
    return derive_pda(user, "vault", seed);
}

pub validate_vault_pda(user: pubkey, seed: u64, bump: u8) -> pubkey {
    return derive_pda(user, "vault", seed, bump);
}
