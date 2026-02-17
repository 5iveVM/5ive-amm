interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    burn @discriminator(8) (
        token_account: Account,
        mint: Account,
        authority: Account,
        amount: u64
    );
}

mut total_burned: u64;
mut pda_derived: pubkey;

pub setup() {
    total_burned = 0;
}

pub burn_from_pda(
    pda_authority: account @signer,
    token_account: account @mut,
    mint: account @mut,
    token_program: account
) -> u64 {
    SPLToken.burn(token_account, mint, pda_authority, 1000);
    total_burned = total_burned + 1000;
    return total_burned;
}

pub get_total_burned() -> u64 {
    return total_burned;
}
