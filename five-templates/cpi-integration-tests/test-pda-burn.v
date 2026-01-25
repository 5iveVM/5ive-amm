// SPL Token Burn with PDA Authority Integration Test Contract
//
// This contract tests INVOKE_SIGNED CPI with Program Derived Address authority.
// It verifies:
// - INVOKE_SIGNED opcode functionality
// - PDA authority validation by Solana runtime
// - Correct seed derivation
// - SPL Token burn instruction format
// - State changes with delegated authority

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    burn @discriminator(8) (
        token_account: pubkey,
        mint: pubkey,
        authority: pubkey,
        amount: u64
    );
}

// Track burn operations
mut total_burned: u64;
mut pda_derived: pubkey;

pub init() {
    total_burned = 0;
}

pub burn_from_pda(
    token_account: account @mut,
    mint: account @mut,
    pda_authority: account
) -> u64 {
    // Call SPL Token's burn instruction with PDA authority
    // The VM uses INVOKE_SIGNED internally
    // Solana validates the PDA against derived seeds
    //
    // Expected instruction format (Borsh):
    // [32 bytes: token_account pubkey]
    // [32 bytes: mint pubkey]
    // [32 bytes: pda_authority pubkey]
    // [8 bytes: 1000 as u64 LE]
    // [1 byte: discriminator 8]
    SPLToken.burn(token_account, mint, pda_authority, 1000);

    // Track state change
    total_burned = total_burned + 1000;

    return total_burned;
}

pub get_total_burned() -> u64 {
    return total_burned;
}
