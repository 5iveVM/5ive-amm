// Test script to verify interface registry functionality
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    initialize_mint @discriminator(0) (mint: pubkey, decimals: u8, authority: pubkey, freeze_authority: pubkey);
    mint_to @discriminator(7) (mint: pubkey, to: pubkey, authority: pubkey, amount: u64);
}

test_interface_registry(mint: account @mut, dest: account @mut, amount: u64) {
    SPLToken.mint_to(mint, dest, mint, amount);
}