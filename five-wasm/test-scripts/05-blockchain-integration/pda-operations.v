// Find PDA and return both address and bump seed
pub find_vault_pda(user: pubkey, seed: u64) -> (pubkey, u8) {
    return derive_pda(user, "vault", seed);  // Returns (address, bump)
}

// Validate PDA with known bump (faster, lower CU cost)
pub validate_vault_pda(user: pubkey, seed: u64, bump: u8) -> pubkey {
    return derive_pda(user, "vault", seed, bump);  // Validates with bump
}

// Find token PDA with bump
pub find_token_pda(mint: pubkey, owner: pubkey) -> (pubkey, u8) {
    return derive_pda("token", mint, owner);
}

// Validate token PDA with bump
pub validate_token_pda(mint: pubkey, owner: pubkey, bump: u8) -> pubkey {
    return derive_pda("token", mint, owner, bump);
}

// Simple PDA operations
pub find_simple_pda(seed: u64) -> (pubkey, u8) {
    return derive_pda("simple", seed);
}

pub validate_simple_pda(seed: u64, bump: u8) -> pubkey {
    return derive_pda("simple", seed, bump);
}

// PARAMETERLESS TEST FUNCTIONS - CLI ENTRY POINTS
test_vault_pda() -> (pubkey, u8) {
    return derive_pda("11111111111111111111111111111112", "vault", 123);
}

test_token_pda() -> (pubkey, u8) {
    return derive_pda("token", "22222222222222222222222222222222", "33333333333333333333333333333333");
}

test_simple_pda() -> (pubkey, u8) {
    return derive_pda("simple", 456);
}
