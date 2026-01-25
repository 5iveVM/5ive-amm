// SPL Token Mint Integration Test Contract
//
// This contract tests CPI to the SPL Token program's mint_to instruction.
// It verifies:
// - Correct serialization of instruction data
// - Proper account ordering
// - Discriminator encoding (Borsh format)
// - State changes in external program

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: pubkey,
        to: pubkey,
        authority: pubkey,
        amount: u64
    );
}

// Track mint operations
mut total_minted: u64;
mut last_mint_time: u64;

pub init() {
    total_minted = 0;
    last_mint_time = 0;
}

pub mint_tokens(
    mint: account @mut,
    to: account @mut,
    authority: account @signer
) -> u64 {
    // Call SPL Token's mint_to instruction
    // Expected instruction format (Borsh):
    // [32 bytes: mint pubkey]
    // [32 bytes: to pubkey]
    // [32 bytes: authority pubkey]
    // [8 bytes: 1000 as u64 LE]
    // [1 byte: discriminator 7]
    SPLToken.mint_to(mint, to, authority, 1000);

    // Track state change
    total_minted = total_minted + 1000;

    return total_minted;
}

pub mint_variable_amount(
    mint: account @mut,
    to: account @mut,
    authority: account @signer,
    amount: u64
) -> u64 {
    // Note: This will fail in current MVP
    // because data arguments must be literals
    // Workaround: use different contract instances
    // with different literal amounts

    // Would call: SPLToken.mint_to(mint, to, authority, amount);
    // But 'amount' is not a literal, so this fails

    // This demonstrates the MVP limitation
    return total_minted;
}
