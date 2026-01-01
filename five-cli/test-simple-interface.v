interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (mint: pubkey, to: pubkey, authority: pubkey, amount: u64);
}

test_simple(mint: account @mut, dest: account @mut, amount: u64) {
    SPLToken.mint_to(mint, dest, mint, amount);
}