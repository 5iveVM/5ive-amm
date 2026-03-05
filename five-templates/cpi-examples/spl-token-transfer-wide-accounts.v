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
    SPLToken::mint_to(mint, user1_token, user1, 500);
}

pub transfer_wide(
    pad_a: account,
    pad_b: account,
    pad_c: account,
    pad_d: account,
    pad_e: account,
    user1_token: account @mut,
    user2_token: account @mut,
    user1: account @signer
) {
    SPLToken::transfer(user1_token, user2_token, user1, 100);
}
