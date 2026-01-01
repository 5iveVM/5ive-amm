// Five Native Token Implementation
// A complete fungible token system built on Five DSL with mint, transfer, and burn operations
// @test-params

// ============================================================================
// ACCOUNT DEFINITIONS
// ============================================================================

// Mint account holds the token metadata and supply information
account Mint {
    authority: pubkey;           // Minting authority - can mint new tokens
    freeze_authority: pubkey;    // Authority to freeze token accounts
    supply: u64;                 // Total supply of tokens in circulation
    decimals: u8;                // Number of decimal places (e.g., 8 for USDC-like)
    name: string;                // Token name
}

// TokenAccount represents a user's token balance for a specific mint
account TokenAccount {
    owner: pubkey;               // Owner of the token account
    mint: pubkey;                // Reference to the mint this account belongs to
    balance: u64;                // Current balance of tokens
    is_frozen: bool;             // Whether account is frozen (no transfers allowed)
    delegated_amount: u64;       // Amount that has been delegated to another account
    delegate: pubkey;            // Account that tokens were delegated to
}

// ============================================================================
// TOKEN INITIALIZATION
// ============================================================================

// Initialize a new mint - creates the token with metadata and authority
pub init_mint(
    mint_account: Mint @mut @init,
    authority: account @signer,
    freeze_authority: pubkey,
    decimals: u8,
    name: string
) -> pubkey {
    mint_account.authority = authority.key;
    mint_account.freeze_authority = freeze_authority;
    mint_account.supply = 0;
    mint_account.decimals = decimals;
    mint_account.name = name;

    return get_key(mint_account);
}

// Initialize a token account for a user - prepares them to hold tokens
pub init_token_account(
    token_account: TokenAccount @mut @init,
    owner: account @signer,
    mint: pubkey
) -> pubkey {
    token_account.owner = owner.key;
    token_account.mint = mint;
    token_account.balance = 0;
    token_account.is_frozen = false;
    token_account.delegated_amount = 0;
    token_account.delegate = 0;

    return get_key(token_account);
}

// ============================================================================
// MINTING & BURNING
// ============================================================================

// Mint new tokens into a specified account
// Only the mint authority can mint tokens
pub mint_to(
    mint_state: Mint @mut,
    destination_account: TokenAccount @mut,
    mint_authority: account @signer,
    amount: u64
) {
    // Verify the mint authority is correct
    require(mint_state.authority == mint_authority.key, "Only mint authority can mint");

    // Verify the destination account belongs to this mint
    require(destination_account.mint == get_key(mint_state), "Destination account mint mismatch");

    // Verify the account is not frozen
    require(!destination_account.is_frozen, "Destination account is frozen");

    // Verify amount is not zero
    require(amount > 0, "Mint amount must be greater than zero");

    // Update the supply and balance
    mint_state.supply = mint_state.supply + amount;
    destination_account.balance = destination_account.balance + amount;
}

// Burn (destroy) tokens from an account
// Only the owner can burn their tokens
pub burn(
    mint_state: Mint @mut,
    source_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    // Verify ownership
    require(source_account.owner == owner.key, "Only owner can burn tokens");

    // Verify sufficient balance
    require(source_account.balance >= amount, "Insufficient balance to burn");

    // Verify the account belongs to this mint
    require(source_account.mint == get_key(mint_state), "Account mint mismatch");

    // Verify account is not frozen
    require(!source_account.is_frozen, "Account is frozen");

    // Verify amount is not zero
    require(amount > 0, "Burn amount must be greater than zero");

    // Update the supply and balance
    mint_state.supply = mint_state.supply - amount;
    source_account.balance = source_account.balance - amount;
}

// ============================================================================
// TRANSFERS
// ============================================================================

// Transfer tokens from one account to another
// Only the owner of the source account can initiate transfers
pub transfer(
    source_account: TokenAccount @mut,
    destination_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    // Verify ownership of source account
    require(source_account.owner == owner.key, "Only account owner can initiate transfer");

    // Verify sufficient balance
    require(source_account.balance >= amount, "Insufficient balance");

    // Verify both accounts are for the same mint
    require(
        source_account.mint == destination_account.mint,
        "Cannot transfer between different mints"
    );

    // Verify neither account is frozen
    require(!source_account.is_frozen, "Source account is frozen");
    require(!destination_account.is_frozen, "Destination account is frozen");

    // Verify amount is not zero
    require(amount > 0, "Transfer amount must be greater than zero");

    // Execute the transfer
    source_account.balance = source_account.balance - amount;
    destination_account.balance = destination_account.balance + amount;
}

// Transfer tokens from a source account, allowing delegation
// This allows delegated authorities to transfer on behalf of owners
pub transfer_from(
    source_account: TokenAccount @mut,
    destination_account: TokenAccount @mut,
    authority: account @signer,
    amount: u64
) {
    // Verify the authority is either the owner or a delegated authority
    let is_owner = source_account.owner == authority.key;
    let is_delegated = source_account.delegate == authority.key &&
                      source_account.delegated_amount >= amount;

    require(is_owner || is_delegated, "Unauthorized transfer");

    // Verify sufficient balance
    require(source_account.balance >= amount, "Insufficient balance");

    // Verify both accounts are for the same mint
    require(
        source_account.mint == destination_account.mint,
        "Cannot transfer between different mints"
    );

    // Verify neither account is frozen
    require(!source_account.is_frozen, "Source account is frozen");
    require(!destination_account.is_frozen, "Destination account is frozen");

    // Verify amount is not zero
    require(amount > 0, "Transfer amount must be greater than zero");

    // Update delegated amount if this is a delegated transfer
    if (is_delegated) {
        source_account.delegated_amount = source_account.delegated_amount - amount;
    }

    // Execute the transfer
    source_account.balance = source_account.balance - amount;
    destination_account.balance = destination_account.balance + amount;
}

// ============================================================================
// DELEGATION & FREEZING
// ============================================================================

// Approve another account to transfer tokens on behalf of this account
pub approve(
    source_account: TokenAccount @mut,
    owner: account @signer,
    delegate: pubkey,
    amount: u64
) {
    // Verify ownership
    require(source_account.owner == owner.key, "Only owner can approve delegation");

    // Set the delegate and amount
    source_account.delegate = delegate;
    source_account.delegated_amount = amount;
}

// Revoke delegation - prevent any further transfers from delegated authority
pub revoke(
    source_account: TokenAccount @mut,
    owner: account @signer
) {
    // Verify ownership
    require(source_account.owner == owner.key, "Only owner can revoke delegation");

    // Clear the delegation
    source_account.delegate = 0;
    source_account.delegated_amount = 0;
}

// Freeze a token account (prevent transfers)
pub freeze_account(
    mint_state: Mint,
    account_to_freeze: TokenAccount @mut,
    freeze_authority: account @signer
) {
    // Verify freeze authority
    require(
        mint_state.freeze_authority == freeze_authority.key,
        "Only freeze authority can freeze accounts"
    );

    // Verify account belongs to this mint
    require(account_to_freeze.mint == get_key(mint_state), "Account mint mismatch");

    // Freeze the account
    account_to_freeze.is_frozen = true;
}

// Thaw a frozen token account (re-enable transfers)
pub thaw_account(
    mint_state: Mint,
    account_to_thaw: TokenAccount @mut,
    freeze_authority: account @signer
) {
    // Verify freeze authority
    require(
        mint_state.freeze_authority == freeze_authority.key,
        "Only freeze authority can thaw accounts"
    );

    // Verify account belongs to this mint
    require(account_to_thaw.mint == get_key(mint_state), "Account mint mismatch");

    // Thaw the account
    account_to_thaw.is_frozen = false;
}

// ============================================================================
// AUTHORITY MANAGEMENT
// ============================================================================

// Change the mint authority for a token
pub set_mint_authority(
    mint_state: Mint @mut,
    current_authority: account @signer,
    new_authority: pubkey
) {
    // Verify current authority
    require(
        mint_state.authority == current_authority.key,
        "Only current mint authority can change authority"
    );

    // Update the mint authority
    mint_state.authority = new_authority;
}

// Change the freeze authority for a token
pub set_freeze_authority(
    mint_state: Mint @mut,
    current_freeze_authority: account @signer,
    new_freeze_authority: pubkey
) {
    // Verify current freeze authority
    require(
        mint_state.freeze_authority == current_freeze_authority.key,
        "Only current freeze authority can change authority"
    );

    // Update the freeze authority
    mint_state.freeze_authority = new_freeze_authority;
}

// ============================================================================
// READ-ONLY QUERIES
// ============================================================================

// Get the total token supply
pub get_supply(mint: Mint) -> u64 {
    return mint.supply;
}

// Get the balance of a token account
pub get_balance(account: TokenAccount) -> u64 {
    return account.balance;
}

// Get the owner of a token account
pub get_owner(account: TokenAccount) -> pubkey {
    return account.owner;
}

// Get the mint of a token account
pub get_mint(account: TokenAccount) -> pubkey {
    return account.mint;
}

// Check if an account is frozen
pub is_frozen(account: TokenAccount) -> bool {
    return account.is_frozen;
}

// Get token metadata
pub get_mint_metadata(mint: Mint) -> string {
    return mint.name;
}

// Get decimals for a mint
pub get_decimals(mint: Mint) -> u8 {
    return mint.decimals;
}

// Get the delegated amount for an account
pub get_delegated_amount(account: TokenAccount) -> u64 {
    return account.delegated_amount;
}

// Get the delegate address for an account
pub get_delegate(account: TokenAccount) -> pubkey {
    return account.delegate;
}
