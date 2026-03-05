// Token template contract tuned for standalone project compilation.

account Mint {
    mint_authority: pubkey;
    freeze_key: pubkey;
    total_supply: u64;
    decimals: u8;
    is_disabled: bool;
}

account TokenAccount {
    owner_key: pubkey;
    mint_key: pubkey;
    balance: u64;
    frozen: bool;
    delegate_key: pubkey;
    allowance: u64;
}

pub init_mint(
    mint_account: Mint @mut @init(payer=authority, space=256) @signer,
    authority: account @mut @signer,
    freeze_authority: pubkey,
    decimals: u8,
    name: string<32>,
    symbol: string<32>,
    uri: string<32>
) -> pubkey {
    require(decimals <= 20);

    mint_account.mint_authority = authority.ctx.key;
    mint_account.freeze_key = freeze_authority;
    mint_account.total_supply = 0;
    mint_account.decimals = decimals;
    mint_account.is_disabled = false;

    return mint_account.ctx.key;
}

pub init_token_account(
    token_account: TokenAccount @mut @init(payer=owner, space=192) @signer,
    owner: account @mut @signer,
    mint: pubkey
) -> pubkey {
    token_account.owner_key = owner.ctx.key;
    token_account.mint_key = mint;
    token_account.balance = 0;
    token_account.frozen = false;
    token_account.delegate_key = owner.ctx.key;
    token_account.allowance = 0;

    return token_account.ctx.key;
}

pub mint_to(
    mint_state: Mint @mut,
    destination_account: TokenAccount @mut,
    mint_authority: account @signer,
    amount: u64
) {
    require(!mint_state.is_disabled);
    require(mint_state.mint_authority == mint_authority.ctx.key);
    require(destination_account.mint_key == mint_state.ctx.key);
    require(!destination_account.frozen);
    require(amount > 0);

    mint_state.total_supply = mint_state.total_supply + amount;
    destination_account.balance = destination_account.balance + amount;
}

pub transfer(
    source_account: TokenAccount @mut,
    destination_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(source_account.owner_key == owner.ctx.key);
    require(source_account.balance >= amount);
    require(source_account.mint_key == destination_account.mint_key);
    require(!source_account.frozen);
    require(!destination_account.frozen);
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
    require(source_account.delegate_key == authority.ctx.key);
    require(source_account.allowance >= amount);
    require(source_account.balance >= amount);
    require(source_account.mint_key == destination_account.mint_key);
    require(!source_account.frozen);
    require(!destination_account.frozen);
    require(amount > 0);

    source_account.allowance = source_account.allowance - amount;
    source_account.balance = source_account.balance - amount;
    destination_account.balance = destination_account.balance + amount;
}

pub approve(
    source_account: TokenAccount @mut,
    owner: account @signer,
    delegate: pubkey,
    amount: u64
) {
    require(source_account.owner_key == owner.ctx.key);

    source_account.delegate_key = delegate;
    source_account.allowance = amount;
}

pub revoke(
    source_account: TokenAccount @mut,
    owner: account @signer
) {
    require(source_account.owner_key == owner.ctx.key);

    source_account.allowance = 0;
}

pub burn(
    mint_state: Mint @mut,
    source_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(source_account.owner_key == owner.ctx.key);
    require(source_account.balance >= amount);
    require(source_account.mint_key == mint_state.ctx.key);
    require(!source_account.frozen);
    require(amount > 0);

    mint_state.total_supply = mint_state.total_supply - amount;
    source_account.balance = source_account.balance - amount;
}

pub freeze_account(
    mint_state: Mint,
    account_to_freeze: TokenAccount @mut,
    freeze_authority: account @signer
) {
    require(mint_state.freeze_key == freeze_authority.ctx.key);
    require(account_to_freeze.mint_key == mint_state.ctx.key);

    account_to_freeze.frozen = true;
}

pub thaw_account(
    mint_state: Mint,
    account_to_thaw: TokenAccount @mut,
    freeze_authority: account @signer
) {
    require(mint_state.freeze_key == freeze_authority.ctx.key);
    require(account_to_thaw.mint_key == mint_state.ctx.key);

    account_to_thaw.frozen = false;
}

pub disable_mint(
    mint_state: Mint @mut,
    current_authority: account @signer
) {
    require(mint_state.mint_authority == current_authority.ctx.key);
    mint_state.is_disabled = true;
}
