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

pub mint_to_user2(
    mint: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    SPLToken.mint_to(mint, user2_token, user1, 500);
}

pub transfer_dynamic(
    user2_token: account @mut,
    user3_token: account @mut,
    user2: account @signer,
    amount: u64
) {
    SPLToken.transfer(user2_token, user3_token, user2, amount);
}
