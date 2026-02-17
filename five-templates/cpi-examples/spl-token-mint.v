interface SystemProgram @program("11111111111111111111111111111111") {
    transfer @discriminator(2) (from: pubkey, to: pubkey, amount: u64);
}

pub mint_tokens(
    mint: account @mut @signer,
    to: account @mut,
    authority: account @signer
) {
    SystemProgram.transfer(mint, to, 1);
}
