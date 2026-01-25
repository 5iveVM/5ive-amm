// SPL Token Mint Example
//
// This contract demonstrates CPI to the SPL Token program.
// It mints tokens from a token mint to a destination account.
//
// Interface: Calls SPL Token's mint_to instruction
// Serializer: Borsh (standard for Anchor/SPL)
// Data Args: amount (u64 literal)
// Account Args: mint, to, authority

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: pubkey,
        to: pubkey,
        authority: pubkey,
        amount: u64
    );
}

pub mint_tokens(
    mint: account @mut,
    to: account @mut,
    authority: account @signer,
    amount: u64
) {
    // Call SPL Token's mint_to instruction
    // - mint: the token mint to mint from
    // - to: destination token account
    // - authority: mint authority (must be signer)
    // - amount: number of tokens to mint (as u64 literal)
    SPLToken.mint_to(mint, to, authority, 1000);
}
