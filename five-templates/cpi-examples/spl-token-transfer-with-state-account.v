interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: Account,
        to: Account,
        authority: Account,
        amount: u64
    );
    transfer @discriminator(3) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
}

account Pool {
    reserve_a: u64;
}

pub mint_source(
    mint: account @mut,
    user1_token: account @mut,
    user1: account @signer
) {
    SPLToken::mint_to(mint, user1_token, user1, 500);
}

pub transfer_with_state(
    pool: Pool @mut,
    user1_token: account @mut,
    user2_token: account @mut,
    user1: account @signer,
    amount: u64
) {
    require(amount > 0);
    SPLToken::transfer(user1_token, user2_token, user1, amount);
}
