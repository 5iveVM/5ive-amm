// Token Implementation
// @test-params

account Mint {
    authority: pubkey;
    freeze_authority: pubkey;
    supply: u64;
    decimals: u8;
    name: string;
    symbol: string;
    uri: string;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
    is_frozen: bool;
    delegated_amount: u64;
    delegate: pubkey;
    initialized: bool;
}

pub init_mint(
    mint_account: Mint @mut @init(payer=authority, space=256) @signer,
    authority: account @mut @signer,
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

pub init_token_account(
    token_account: TokenAccount @mut @init(payer=owner, space=192) @signer,
    owner: account @mut @signer,
    mint: pubkey
) -> pubkey {
    token_account.owner = owner.key;
    token_account.mint = mint;
    token_account.balance = 0;
    token_account.is_frozen = false;
    token_account.delegated_amount = 0;
    token_account.delegate = 0;
    token_account.initialized = true;
    return token_account.key;
}

pub mint_to(
    mint_state: Mint @mut,
    destination_account: TokenAccount @mut,
    mint_authority: account @signer,
    amount: u64
) {
    require(mint_state.authority == mint_authority.key);
    require(destination_account.mint == mint_state.key);
    require(!destination_account.is_frozen);
    require(amount > 0);
    mint_state.supply = mint_state.supply + amount;
    destination_account.balance = destination_account.balance + amount;
}

pub transfer(
    source_account: TokenAccount @mut,
    destination_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(source_account.owner == owner.key);
    require(source_account.balance >= amount);
    require(source_account.mint == destination_account.mint);
    require(!source_account.is_frozen);
    require(!destination_account.is_frozen);
    require(amount > 0);
    source_account.balance = source_account.balance - amount;
    destination_account.balance = destination_account.balance + amount;
}

pub transfer_from(
    source_account: TokenAccount @mut,
    destination_account: TokenAccount @mut,
    authority: account @signer,
    amount: u64
) {
    let is_owner = source_account.owner == authority.key;
    if (!is_owner) {
        require(source_account.delegate == authority.key);
        require(source_account.delegated_amount >= amount);
    }
    require(source_account.balance >= amount);
    require(source_account.mint == destination_account.mint);
    require(!source_account.is_frozen);
    require(!destination_account.is_frozen);
    require(amount > 0);
    if (!is_owner) {
        source_account.delegated_amount = source_account.delegated_amount - amount;
    }
    source_account.balance = source_account.balance - amount;
    destination_account.balance = destination_account.balance + amount;
}

pub approve(
    source_account: TokenAccount @mut,
    owner: account @signer,
    delegate: pubkey,
    amount: u64
) {
    require(source_account.owner == owner.key);
    source_account.delegate = delegate;
    source_account.delegated_amount = amount;
}

pub revoke(
    source_account: TokenAccount @mut,
    owner: account @signer
) {
    require(source_account.owner == owner.key);
    source_account.delegate = 0;
    source_account.delegated_amount = 0;
}

pub burn(
    mint_state: Mint @mut,
    source_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(source_account.owner == owner.key);
    require(source_account.balance >= amount);
    require(source_account.mint == mint_state.key);
    require(!source_account.is_frozen);
    require(amount > 0);
    mint_state.supply = mint_state.supply - amount;
    source_account.balance = source_account.balance - amount;
}

pub freeze_account(
    mint_state: Mint,
    account_to_freeze: TokenAccount @mut,
    freeze_authority: account @signer
) {
    require(mint_state.freeze_authority == freeze_authority.key);
    require(account_to_freeze.mint == mint_state.key);
    account_to_freeze.is_frozen = true;
}

pub thaw_account(
    mint_state: Mint,
    account_to_thaw: TokenAccount @mut,
    freeze_authority: account @signer
) {
    require(mint_state.freeze_authority == freeze_authority.key);
    require(account_to_thaw.mint == mint_state.key);
    account_to_thaw.is_frozen = false;
}

pub set_mint_authority(
    mint_state: Mint @mut,
    current_authority: account @signer,
    new_authority: pubkey
) {
    require(mint_state.authority == current_authority.key);
    mint_state.authority = new_authority;
}

pub set_freeze_authority(
    mint_state: Mint @mut,
    current_freeze_authority: account @signer,
    new_freeze_authority: pubkey
) {
    require(mint_state.freeze_authority == current_freeze_authority.key);
    mint_state.freeze_authority = new_freeze_authority;
}

pub disable_mint(
    mint_state: Mint @mut,
    current_authority: account @signer
) {
    require(mint_state.authority == current_authority.key);
    mint_state.authority = 0;
}

pub disable_freeze(
    mint_state: Mint @mut,
    current_freeze_authority: account @signer
) {
    require(mint_state.freeze_authority == current_freeze_authority.key);
    mint_state.freeze_authority = 0;
}
