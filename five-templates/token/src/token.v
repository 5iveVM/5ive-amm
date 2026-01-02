// Five Native Token Implementation
// A complete fungible token system built on Five DSL with mint, transfer, and burn operations
// Comprehensive SPL Token-compatible implementation in a single file
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
    symbol: string;              // Token symbol
    uri: string;                 // Token URI (metadata)
}

// TokenAccount represents a user's token balance for a specific mint
account TokenAccount {
    owner: pubkey;               // Owner of the token account
    mint: pubkey;                // Reference to the mint this account belongs to
    balance: u64;                // Current balance of tokens
    is_frozen: bool;             // Whether account is frozen (no transfers allowed)
    delegated_amount: u64;       // Amount that has been delegated to another account
    delegate: pubkey;            // Account that tokens were delegated to
    initialized: bool;           // Whether the account is initialized
}

// ============================================================================
// TOKEN INITIALIZATION
// ============================================================================

// Initialize a new mint - creates the token with metadata and authority
// Only the authority signer can create mints
pub init_mint(
    // Initialize the mint account
    // Explicitly specify authority as the payer for rent and space
    mint_account: Mint @mut @init(payer=authority, space=256) @signer,
    
    // The authority who can mint tokens
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

// Initialize a token account for a user - prepares them to hold tokens
pub init_token_account(
    token_account: TokenAccount @mut @init(payer=owner, space=192) @signer,
    owner: account @signer,
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

// Create an Associated Token Account (ATA)
// Convenience wrapper around init_token_account for parity with SPL Token
pub create_associated_token_account(
    ata_account: TokenAccount @mut @init(payer=owner, space=192) @signer,
    owner: account @signer,
    mint: pubkey
) -> pubkey {
    return init_token_account(ata_account, owner, mint);
}

// Create a deterministic token account without requiring the owner to sign
// Seeds: [mint, owner_pubkey]
// Anyone can create the account, but only at the derived PDA address.
pub create_deterministic_token_account(
    ata_account: TokenAccount @mut @init(space=192),
    owner: pubkey,
    mint: pubkey
) -> pubkey {
    let (expected, _bump) = derive_pda("token", mint, owner);
    require(ata_account.key == expected);

    ata_account.owner = owner;
    ata_account.mint = mint;
    ata_account.balance = 0;
    ata_account.is_frozen = false;
    ata_account.delegate = 0;
    ata_account.delegated_amount = 0;
    ata_account.initialized = true;

    return ata_account.key;
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
    require(mint_state.authority == mint_authority.key);

    // Verify the destination account belongs to this mint
    require(destination_account.mint == mint_state.key);

    // Verify the account is not frozen
    require(!destination_account.is_frozen);

    // Verify amount is not zero
    require(amount > 0);

    // Verify supply won't overflow
    require(mint_state.supply <= 9223372036854775807 - amount);

    // Verify balance won't overflow
    require(destination_account.balance <= 18446744073709551615 - amount);

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
    require(source_account.owner == owner.key);

    // Verify sufficient balance
    require(source_account.balance >= amount);

    // Verify the account belongs to this mint
    require(source_account.mint == mint_state.key);

    // Verify account is not frozen
    require(!source_account.is_frozen);

    // Verify amount is not zero
    require(amount > 0);

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
    require(source_account.owner == owner.key);

    // Verify sufficient balance
    require(source_account.balance >= amount);

    // Verify both accounts are for the same mint
    require(
        source_account.mint == destination_account.mint
    );

    // Verify neither account is frozen
    require(!source_account.is_frozen);
    require(!destination_account.is_frozen);

    // Verify amount is not zero
    require(amount > 0);

    // Verify destination won't overflow
    require(destination_account.balance <= 18446744073709551615 - amount);

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
    if (!is_owner) {
        require(source_account.delegate == authority.key);
        require(source_account.delegated_amount >= amount);
    }

    // Verify sufficient balance
    require(source_account.balance >= amount);

    // Verify both accounts are for the same mint
    require(
        source_account.mint == destination_account.mint
    );

    // Verify neither account is frozen
    require(!source_account.is_frozen);
    require(!destination_account.is_frozen);

    // Verify amount is not zero
    require(amount > 0);

    // Verify destination won't overflow
    require(destination_account.balance <= 18446744073709551615 - amount);

    // Update delegated amount if this is a delegated transfer
    if (!is_owner) {
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
    require(source_account.owner == owner.key);

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
    require(source_account.owner == owner.key);

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
        mint_state.freeze_authority == freeze_authority.key
    );

    // Verify account belongs to this mint
    require(account_to_freeze.mint == mint_state.key);

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
        mint_state.freeze_authority == freeze_authority.key
    );

    // Verify account belongs to this mint
    require(account_to_thaw.mint == mint_state.key);

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
        mint_state.authority == current_authority.key
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
        mint_state.freeze_authority == current_freeze_authority.key
    );

    // Update the freeze authority
    mint_state.freeze_authority = new_freeze_authority;
}

// Disable minting permanently by removing the mint authority
// This is irreversible - no one will be able to mint after this
// Useful for creating capped tokens
pub disable_mint(
    mint_state: Mint @mut,
    current_authority: account @signer
) {
    // Verify current authority
    require(mint_state.authority == current_authority.key);

    // Set authority to null (no one can mint)
    mint_state.authority = 0;
}

// Disable freezing permanently by removing the freeze authority
// No one will be able to freeze token accounts after this
pub disable_freeze(
    mint_state: Mint @mut,
    current_freeze_authority: account @signer
) {
    // Verify current freeze authority
    require(mint_state.freeze_authority == current_freeze_authority.key);

    // Set freeze authority to null (no one can freeze)
    mint_state.freeze_authority = 0;
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

// Verify mint and token account match
pub verify_account_mint(account: TokenAccount, mint: pubkey) {
    require(account.mint == mint);
}

// Verify token account owner
pub verify_account_owner(account: TokenAccount, owner: pubkey) {
    require(account.owner == owner);
}

// Verify both mint and owner
pub verify_account_full(account: TokenAccount, mint: pubkey, owner: pubkey) {
    verify_account_mint(account, mint);
    verify_account_owner(account, owner);
}
