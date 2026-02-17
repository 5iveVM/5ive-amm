interface SystemProgram @program("11111111111111111111111111111111") {
    transfer @discriminator(2) (from: pubkey, to: pubkey, amount: u64);
}

pub burn_from_pda(
    token_account: account @mut @signer,
    mint: account @mut,
    pda_authority: account
) {
    SystemProgram.transfer(token_account, mint, 1);
}
