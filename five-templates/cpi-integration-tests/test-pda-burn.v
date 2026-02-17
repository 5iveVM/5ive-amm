interface SystemProgram @program("11111111111111111111111111111111") {
    transfer @discriminator(2) (from: pubkey, to: pubkey, amount: u64);
}

mut total_burned: u64;
mut pda_derived: pubkey;

pub setup() {
    total_burned = 0;
}

pub burn_from_pda(
    token_account: account @mut @signer,
    mint: account @mut,
    pda_authority: account
) -> u64 {
    SystemProgram.transfer(token_account, mint, 1);
    total_burned = total_burned + 1;
    return total_burned;
}

pub get_total_burned() -> u64 {
    return total_burned;
}
