// Five Token - Self-contained token implementation
// Provides full token functionality: mint, transfer, burn, initialize

// Token Mint account - represents a specific token type
account Mint {
    authority: pubkey;
    supply: u64;
    decimals: u8;
}

// Token Account - user's balance for a specific token
account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
    approved_delegate: pubkey;
    approved_amount: u64;
    is_frozen: bool;
}

// ============================================================================
// INITIALIZATION
// ============================================================================

// Initialize a new token mint
pub init_mint(mint: Mint @mut @init, authority: account @signer, decimals: u8) {
    mint.authority = authority.key;
    mint.supply = 0;
    mint.decimals = decimals;
}

// Initialize a new token account for a user
pub init_token_account(token_account: TokenAccount @mut @init, owner: account @signer, mint: Mint) {
    token_account.owner = owner.key;
    token_account.mint = mint.key;
    token_account.balance = 0;
    token_account.approved_amount = 0;
    token_account.is_frozen = false;
}

// ============================================================================
// MINT/BURN
// ============================================================================

// Mint new tokens to an account
pub mint(mint: Mint @mut @has(authority), token_account: TokenAccount @mut, authority: account @signer, amount: u64) {
    require(token_account.mint == mint.key);
    require(amount > 0);
    require(!token_account.is_frozen);

    token_account.balance = token_account.balance + amount;
    mint.supply = mint.supply + amount;
}

// Burn tokens from an account
pub burn(mint: Mint @mut, token_account: TokenAccount @mut @has(owner), owner: account @signer, amount: u64) {
    require(token_account.mint == mint.key);
    require(amount > 0);
    require(!token_account.is_frozen);
    require(token_account.balance >= amount);

    token_account.balance = token_account.balance - amount;
    mint.supply = mint.supply - amount;
}

// ============================================================================
// TRANSFER
// ============================================================================

// Transfer tokens from one account to another
pub transfer(from_account: TokenAccount @mut @has(owner), to_account: TokenAccount @mut, owner: account @signer, amount: u64) {
    require(from_account.mint == to_account.mint);
    require(amount > 0);
    require(!from_account.is_frozen);
    require(!to_account.is_frozen);
    require(from_account.balance >= amount);

    from_account.balance = from_account.balance - amount;
    to_account.balance = to_account.balance + amount;
}

// ============================================================================
// APPROVAL & DELEGATION
// ============================================================================

// Approve a delegate to spend tokens on behalf of owner
pub approve(token_account: TokenAccount @mut @has(owner), owner: account @signer, delegate: account, amount: u64) {
    require(amount > 0);
    token_account.approved_delegate = delegate.key;
    token_account.approved_amount = amount;
}

// Revoke all approvals
pub revoke(token_account: TokenAccount @mut @has(owner), owner: account @signer) {
    token_account.approved_amount = 0;
}

// Transfer tokens using delegation approval
pub transfer_approved(from_account: TokenAccount @mut, to_account: TokenAccount @mut, delegate: account @signer, amount: u64) {
    require(from_account.mint == to_account.mint);
    require(amount > 0);
    require(!from_account.is_frozen);
    require(!to_account.is_frozen);
    require(from_account.balance >= amount);
    require(delegate.key == from_account.approved_delegate);
    require(from_account.approved_amount >= amount);

    from_account.balance = from_account.balance - amount;
    to_account.balance = to_account.balance + amount;
    from_account.approved_amount = from_account.approved_amount - amount;
}

// ============================================================================
// ACCOUNT MANAGEMENT
// ============================================================================

// Freeze a token account (only mint authority can do this)
pub freeze_account(mint: Mint @mut @has(authority), token_account: TokenAccount @mut, authority: account @signer) {
    require(token_account.mint == mint.key);
    token_account.is_frozen = true;
}

// Thaw a frozen token account (only mint authority can do this)
pub thaw_account(mint: Mint @mut @has(authority), token_account: TokenAccount @mut, authority: account @signer) {
    require(token_account.mint == mint.key);
    token_account.is_frozen = false;
}

// Close a token account (only owner can close, must have zero balance)
pub close_account(token_account: TokenAccount @mut @has(owner), owner: account @signer) {
    require(token_account.balance == 0);
    require(token_account.approved_amount == 0);
}

// ============================================================================
// AUTHORITY MANAGEMENT
// ============================================================================

// Transfer mint authority to a new authority
pub set_mint_authority(mint: Mint @mut @has(authority), authority: account @signer, new_authority: account) {
    mint.authority = new_authority.key;
}

