interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    transfer @discriminator(3) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
}

pub cpi_only(
    user_token_a: account @mut,
    pool_token_a_vault: account @mut,
    user_authority: account @signer,
    amount_a: u64
) {
    SPLToken.transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
}
