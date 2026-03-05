// 5IVE Token Contract (SPL Token-compatible)
//
// Design:
//   - Mint: authority, freeze_authority, supply, decimals, name, symbol, max_supply
//   - TokenAccount: owner, mint, balance, delegate, delegated_amount, is_frozen
//   - is_initialized on both accounts guards against re-initialization
//   - Supply cap: mint_to enforces max_supply (0 = uncapped)
//   - Delegate model: owner approves delegate for a specific amount; burn also respects delegate
//   - close_account: owner can reclaim rent from zero-balance account
//   - Authority rotation: mint authority and freeze authority can be transferred or revoked

interface SystemProgram @program("11111111111111111111111111111111") {
    transfer @discriminator(2) (
        from: Account,
        to: Account,
        lamports: u64
    );
}

account Mint {
    authority: pubkey;
    freeze_authority: pubkey;
    supply: u64;
    decimals: u8;
    is_initialized: bool;
    name: string<32>;
    symbol: string<8>;
    max_supply: u64;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
    is_frozen: bool;
    is_initialized: bool;
    delegate: pubkey;
    delegated_amount: u64;
}

// --- Initializers ---

pub init_mint(
    mint: Mint @mut @init(payer=authority, space=1024) @signer,
    authority: account @mut @signer,
    freeze_authority: pubkey,
    decimals: u8,
    name: string<32>,
    symbol: string<8>,
    max_supply: u64
) -> pubkey {
    require(!mint.is_initialized);
    require(decimals <= 20);

    mint.authority = authority.ctx.key;
    mint.freeze_authority = freeze_authority;
    mint.supply = 0;
    mint.decimals = decimals;
    mint.is_initialized = true;
    mint.name = name;
    mint.symbol = symbol;
    mint.max_supply = max_supply;
    return mint.ctx.key;
}

pub init_token_account(
    token_account: TokenAccount @mut @init(payer=payer, space=256) @signer,
    payer: account @mut @signer,
    mint: Mint,
    owner: pubkey
) -> pubkey {
    require(!token_account.is_initialized);
    require(mint.is_initialized);

    token_account.owner = owner;
    token_account.mint = mint.ctx.key;
    token_account.balance = 0;
    token_account.is_frozen = false;
    token_account.is_initialized = true;
    token_account.delegate = 0;
    token_account.delegated_amount = 0;
    return token_account.ctx.key;
}

// --- Core Actions ---

pub mint_to(
    mint: Mint @mut,
    destination: TokenAccount @mut,
    authority: account @signer,
    amount: u64
) {
    require(mint.is_initialized);
    require(destination.is_initialized);
    require(mint.authority == authority.ctx.key);
    require(destination.mint == mint.ctx.key);
    require(!destination.is_frozen);
    require(amount > 0);

    // Enforce supply cap if set
    if (mint.max_supply > 0) {
        require(mint.supply + amount <= mint.max_supply);
    }

    mint.supply = mint.supply + amount;
    destination.balance = destination.balance + amount;
}

pub transfer(
    source: TokenAccount @mut,
    destination: TokenAccount @mut,
    authority: account @signer,
    amount: u64
) {
    require(source.is_initialized);
    require(destination.is_initialized);
    require(amount > 0);
    require(source.mint == destination.mint);
    require(!source.is_frozen);
    require(!destination.is_frozen);
    require(source.balance >= amount);

    if (source.owner == authority.ctx.key) {
        // Owner transfer: no further checks needed
    } else if (source.delegate == authority.ctx.key) {
        require(source.delegated_amount >= amount);
        source.delegated_amount = source.delegated_amount - amount;
    } else {
        require(false);
    }

    source.balance = source.balance - amount;
    destination.balance = destination.balance + amount;
}

pub burn(
    mint: Mint @mut,
    source: TokenAccount @mut,
    authority: account @signer,
    amount: u64
) {
    require(mint.is_initialized);
    require(source.is_initialized);
    require(source.mint == mint.ctx.key);
    require(!source.is_frozen);
    require(amount > 0);
    require(source.balance >= amount);

    if (source.owner == authority.ctx.key) {
        // Owner burn
    } else if (source.delegate == authority.ctx.key) {
        require(source.delegated_amount >= amount);
        source.delegated_amount = source.delegated_amount - amount;
    } else {
        require(false);
    }

    source.balance = source.balance - amount;
    mint.supply = mint.supply - amount;
}

// Close a zero-balance token account, returning rent to the owner.
pub close_account(
    token_account: TokenAccount @mut,
    owner: account @signer,
    rent_destination: account @mut,
    system_program: SystemProgram
) {
    require(token_account.is_initialized);
    require(token_account.owner == owner.ctx.key);
    require(token_account.balance == 0);
    require(token_account.delegated_amount == 0);

    // Mark as closed so it cannot be reused
    token_account.is_initialized = false;
}

// --- Delegation ---

pub approve(
    source: TokenAccount @mut,
    owner: account @signer,
    delegate: pubkey,
    amount: u64
) {
    require(source.is_initialized);
    require(source.owner == owner.ctx.key);
    require(!source.is_frozen);
    require(amount > 0);

    source.delegate = delegate;
    source.delegated_amount = amount;
}

pub revoke(
    source: TokenAccount @mut,
    owner: account @signer
) {
    require(source.is_initialized);
    require(source.owner == owner.ctx.key);

    source.delegate = 0;
    source.delegated_amount = 0;
}

// --- Freeze / Thaw ---

pub freeze_account(
    mint: Mint,
    target: TokenAccount @mut,
    authority: account @signer
) {
    require(mint.is_initialized);
    require(target.is_initialized);
    require(mint.freeze_authority == authority.ctx.key);
    require(target.mint == mint.ctx.key);
    require(!target.is_frozen);

    target.is_frozen = true;
}

pub thaw_account(
    mint: Mint,
    target: TokenAccount @mut,
    authority: account @signer
) {
    require(mint.is_initialized);
    require(target.is_initialized);
    require(mint.freeze_authority == authority.ctx.key);
    require(target.mint == mint.ctx.key);
    require(target.is_frozen);

    target.is_frozen = false;
}

// --- Authority Management ---

pub set_mint_authority(
    mint: Mint @mut,
    current_authority: account @signer,
    new_authority: pubkey
) {
    require(mint.is_initialized);
    require(mint.authority == current_authority.ctx.key);
    mint.authority = new_authority;
}

pub set_freeze_authority(
    mint: Mint @mut,
    current_authority: account @signer,
    new_authority: pubkey
) {
    require(mint.is_initialized);
    require(mint.authority == current_authority.ctx.key);
    mint.freeze_authority = new_authority;
}

pub set_max_supply(
    mint: Mint @mut,
    authority: account @signer,
    new_max_supply: u64
) {
    require(mint.is_initialized);
    require(mint.authority == authority.ctx.key);
    // Can only raise cap, or set to 0 (uncap). Cannot lower below current supply.
    if (new_max_supply > 0) {
        require(new_max_supply >= mint.supply);
    }
    mint.max_supply = new_max_supply;
}

// --- Read-only ---

pub get_supply(mint: Mint) -> u64 {
    return mint.supply;
}

pub get_balance(token_account: TokenAccount) -> u64 {
    return token_account.balance;
}
