interface SystemProgram @program("11111111111111111111111111111111") {
    transfer @discriminator(2) (from: pubkey, to: pubkey, amount: u64);
}

mut total_minted: u64;
mut last_mint_time: u64;

pub setup() {
    total_minted = 0;
    last_mint_time = 0;
}

pub mint_tokens(
    mint: account @mut @signer,
    to: account @mut,
    authority: account @signer
) -> u64 {
    SystemProgram.transfer(mint, to, 1);
    total_minted = total_minted + 1;
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
