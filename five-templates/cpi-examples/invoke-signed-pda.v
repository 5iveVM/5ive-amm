interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    burn @discriminator(8) (
        token_account: Account,
        mint: Account,
        authority: Account,
        amount: u64
    );
}

pub burn_from_pda(
    pda_authority: account @signer,
    token_account: account @mut,
    mint: account @mut,
    token_program: account
) {
    SPLToken.burn(token_account, mint, pda_authority, 1000);
}
