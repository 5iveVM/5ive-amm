// Token Example (adapted from five-templates/token.v)

account Mint {
    authority: pubkey;
    supply: u64;
    decimals: u8;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
}

pub init_mint(mint: Mint @mut, authority: account @signer, decimals: u8) {
    mint.authority = authority.key;
    mint.supply = 0;
    mint.decimals = decimals;
}

pub init_token_account(token_account: TokenAccount @mut, owner: account @signer, mint: account) {
    token_account.owner = owner.key;
    token_account.mint = mint.key;
    token_account.balance = 0;
}

pub mint_to(mint: Mint @mut, token_account: TokenAccount @mut, amount: u64) {
    token_account.balance = token_account.balance + amount;
    mint.supply = mint.supply + amount;
}

pub transfer(from_account: TokenAccount @mut, to_account: TokenAccount @mut, amount: u64) {
    from_account.balance = from_account.balance - amount;
    to_account.balance = to_account.balance + amount;
}

pub burn(mint: Mint @mut, token_account: TokenAccount @mut, amount: u64) {
    token_account.balance = token_account.balance - amount;
    mint.supply = mint.supply - amount;
}
