// Test advanced PDA operations with caching and reuse patterns

pub find_vault_with_validation(user: pubkey, seed: u64) -> (pubkey, u8) {
    let (vault_addr, bump) = derive_pda(user, "vault", seed);
    require(vault_addr != user);
    return (vault_addr, bump);
}

pub find_and_verify_token(mint: pubkey, owner: pubkey) -> (pubkey, u8) {
    let (token_addr, bump) = derive_pda("token", mint, owner);
    require(token_addr != mint);
    require(token_addr != owner);
    return (token_addr, bump);
}

pub cache_pda_results(user: pubkey, seed: u64, mint: pubkey, owner: pubkey) -> (u64, u64) {
    let (vault_addr, vault_bump) = derive_pda(user, "vault", seed);
    let (token_addr, token_bump) = derive_pda("token", mint, owner);

    let vault_bump_u64 = vault_bump as u64;
    let token_bump_u64 = token_bump as u64;

    return (vault_bump_u64, token_bump_u64);
}