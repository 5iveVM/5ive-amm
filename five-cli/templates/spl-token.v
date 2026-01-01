// SPL Token interface template (CPI-style)

// Define SPL Token interface with explicit discriminators and program ID
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    initialize_mint @discriminator(0) (mint: pubkey, decimals: u8, authority: pubkey, freeze_authority: pubkey);
    transfer        @discriminator(3) (source: pubkey, dest: pubkey, authority: pubkey, amount: u64);
    mint_to         @discriminator(7) (mint: pubkey, to: pubkey, authority: pubkey, amount: u64);
}

// If you prefer creating the mint via CPI (not required when created in JS)
pub create_mint(payer: account @signer, mint: account, decimals: u8) -> pubkey {
    SPLToken.initialize_mint(mint, decimals, payer, payer);
    return mint;
}

// Mint tokens to a destination account via interface (requires mint authority signer)
pub mint_tokens(mint: account @mut, dest: account @mut, authority: account @signer, amount: u64) {
    SPLToken.mint_to(mint, dest, authority, amount);
}

// Transfer tokens via CPI (uses source account owner as authority signer)
pub transfer_tokens(source: account @mut, dest: account @mut, authority: account @signer, amount: u64) {
    SPLToken.transfer(source, dest, authority, amount);
}
