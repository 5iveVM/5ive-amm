    // Define SPL Token interface with custom discriminators
    interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
        initialize_mint @discriminator(0) (mint: pubkey, decimals: u8, authority: pubkey, freeze_authority: pubkey);
        mint_to @discriminator(7) (mint: pubkey, to: pubkey, authority: pubkey, amount: u64);
    }
    
pub create_mint(payer: account @signer, mint: account @init, decimals: u8) -> pubkey {
        // Call interface function using dot notation
        SPLToken::initialize_mint(mint, decimals, payer, payer);
        return mint;
    }
    
mint_tokens(mint: account @mut, dest: account @mut, amount: u64) {
        // Call interface function using dot notation
        SPLToken::mint_to(mint, dest, mint, amount);
    }
