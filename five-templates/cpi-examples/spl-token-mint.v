interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: Account,
        to: Account,
        authority: Account,
        amount: u64
    );
}

pub mint_tokens(
    mint: account @mut,
    to: account @mut,
    authority: account @signer,
    token_program: account
) {
    SPLToken.mint_to(mint, to, authority, 1000);
}
