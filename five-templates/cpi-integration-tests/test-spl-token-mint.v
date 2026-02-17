interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: Account,
        to: Account,
        authority: Account,
        amount: u64
    );
}

mut total_minted: u64;
mut last_mint_time: u64;

pub setup() {
    total_minted = 0;
    last_mint_time = 0;
}

pub mint_tokens(
    mint: account @mut,
    to: account @mut,
    authority: account @signer,
    token_program: account
) -> u64 {
    SPLToken.mint_to(mint, to, authority, 1000);
    total_minted = total_minted + 1000;
    return total_minted;
}

pub mint_variable_amount(
    mint: account @mut,
    to: account @mut,
    authority: account @signer,
    amount: u64
) -> u64 {
    return total_minted;
}
