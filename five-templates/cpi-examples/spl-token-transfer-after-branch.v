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

pub mint_source(
    mint: account @mut,
    user1_token: account @mut,
    user1: account @signer
) {
    SPLToken.mint_to(mint, user1_token, user1, 500);
}

pub transfer_after_branch(
    user1_token: account @mut,
    user2_token: account @mut,
    user1: account @signer,
    amount_a: u64,
    amount_b: u64,
    min_liquidity: u64
) {
    let mut liquidity: u64 = 0;

    if (amount_a == amount_b) {
        liquidity = amount_a + amount_b;
    } else {
        liquidity = amount_a;
    }

    require(liquidity >= min_liquidity);
    SPLToken.transfer(user1_token, user2_token, user1, amount_a);
}
