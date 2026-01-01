// Initialize Token Account
pub fn init_token_account(
    token_account: TokenAccount @mut @init,
    owner: account @signer, // or PDA owner passed as param if authorized
    mint: pubkey
) -> pubkey {
    // For template simplicity, we'll assume owner matches signer or is handled by PDA logic
    token_account.owner = owner.key;
    token_account.mint = mint;
    token_account.balance = 0;
    token_account.is_frozen = false;
    token_account.delegate = 0;
    token_account.delegated_amount = 0;
    token_account.initialized = true;

    return token_account.key;
}

// Initialize Token Account with explicit owner pubkey (for PDA usage)
pub fn init_token_account_pda(
    token_account: TokenAccount @mut @init,
    owner_pubkey: pubkey,
    mint: pubkey
) {
    token_account.owner = owner_pubkey;
    token_account.mint = mint;
    token_account.balance = 0;
    token_account.is_frozen = false;
    token_account.initialized = true;
}

// Transfer tokens
pub fn transfer(
    source_account: TokenAccount @mut,
    destination_account: TokenAccount @mut,
    owner: account @signer,
    mint: Mint, 
    amount: u64
) {
    require(amount > 0);
    require(source_account.owner == owner.key);
    require(source_account.mint == destination_account.mint);
    require(source_account.mint == mint.key);
    require(!source_account.is_frozen);
    require(!destination_account.is_frozen);
    require(source_account.balance >= amount);

    source_account.balance = source_account.balance - amount;
    destination_account.balance = destination_account.balance + amount;
}
