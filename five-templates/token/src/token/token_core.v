// ============================================================================
// TOKEN CORE
// ============================================================================

pub init_mint(
    mint: Mint @mut @init,
    authority: account @signer,
    decimals: u8,
    name: string,
    symbol: string
) -> pubkey {
    mint.authority = authority.key;
    mint.supply = 0;
    mint.decimals = decimals;
    mint.name = name;
    mint.symbol = symbol;
    return mint.key;
}

pub init_token_account(
    account: TokenAccount @mut @init,
    owner: account @signer,
    mint: pubkey
) -> pubkey {
    account.owner = owner.key;
    account.mint = mint;
    account.balance = 0;
    account.is_frozen = false;
    account.delegate = 0;
    account.delegated_amount = 0;
    account.initialized = true;
    return account.key;
}
