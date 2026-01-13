import launchpad_types;

// Initialize a new Mint
pub fn init_mint(
    mint_account: launchpad_types::Mint @mut @init,
    authority: account @signer,
    freeze_authority: pubkey,
    decimals: u8,
    name: string,
    symbol: string,
    uri: string
) -> pubkey {
    require(decimals <= 20);
    
    mint_account.authority = authority.key;
    mint_account.freeze_authority = freeze_authority;
    mint_account.supply = 0;
    mint_account.decimals = decimals;
    mint_account.name = name;
    mint_account.symbol = symbol;
    mint_account.uri = uri;

    return mint_account.key;
}

// Mint tokens
pub fn mint_to(
    mint_state: launchpad_types::Mint @mut,
    destination_account: launchpad_types::TokenAccount @mut,
    mint_authority: account @signer,
    amount: u64
) {
    require(amount > 0);
    require(mint_state.authority == mint_authority.key);
    require(destination_account.mint == mint_state.key);
    require(!destination_account.is_frozen);

    mint_state.supply = mint_state.supply + amount;
    destination_account.balance = destination_account.balance + amount;
}
