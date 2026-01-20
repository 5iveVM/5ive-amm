/**
 * Five DSL Templates
 * Counter and Token templates from five-templates
 */

export const COUNTER_TEMPLATE = `// Counter Program - Five DSL E2E Test
// Demonstrates state persistence with account-based storage
// Uses @init constraint for CPI-based account creation

// ============================================================================
// ACCOUNT DEFINITIONS
// ============================================================================

// Counter account holds the state
account Counter {
    authority: pubkey;       // Owner of this counter
    count: u64;          // Current count value
    initialized: u64;    // Whether the counter is initialized (1 = true, 0 = false)
}

// ============================================================================
// COUNTER OPERATIONS
// ============================================================================

// Initialize a new counter account
// Uses @init constraint to create account via CPI
// Counter is a PDA derived from ["counter", owner.key]
pub initialize(
    counter: Counter @mut @init(payer=owner, space=56, seeds=["counter", owner.key]),
    owner: account @signer
) -> pubkey {
    counter.authority = owner.key;
    counter.count = 0;
    counter.initialized = 1;

    return counter.key;
}

// Increment the counter by 1
pub increment(
    counter: Counter @mut,
    owner: account @signer
) {
    // Verify ownership
    require(counter.authority == owner.key);
    require(counter.initialized);

    // Increment the counter
    counter.count = counter.count + 1;
}

// Decrement the counter by 1 (with underflow protection)
pub decrement(
    counter: Counter @mut,
    owner: account @signer
) {
    // Verify ownership
    require(counter.authority == owner.key);
    require(counter.initialized);

    // Decrement the counter (with underflow protection)
    if (counter.count > 0) {
        counter.count = counter.count - 1;
    }
}

// Add a specific amount to the counter
pub add_amount(
    counter: Counter @mut,
    owner: account @signer,
    amount: u64
) {
    // Verify ownership
    require(counter.authority == owner.key);
    require(counter.initialized);

    // Add amount to counter
    counter.count = counter.count + amount;
}

// Get the current count value
pub get_count(counter: Counter) -> u64 {
    return counter.count;
}

// Reset counter to zero
pub reset(
    counter: Counter @mut,
    owner: account @signer
) {
    // Verify ownership
    require(counter.authority == owner.key);
    require(counter.initialized);

    // Reset to zero
    counter.count = 0;
}
`;

export const TOKEN_TEMPLATE = `// Token Implementation
// @test-params

account Mint {
    authority: pubkey;
    freeze_authority: pubkey;
    supply: u64;
    decimals: u8;
    name: string<32>;
    symbol: string<32>;
    uri: string<32>;
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
    name: string<32>,
    symbol: string<32>,
    uri: string<32>
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
`;

export interface Template {
    id: string;
    name: string;
    description: string;
    code: string;
    icon: string;
}

export const TEMPLATES: Template[] = [
    {
        id: 'counter',
        name: 'Counter',
        description: 'Simple counter with increment, decrement, and reset',
        code: COUNTER_TEMPLATE,
        icon: '🔢'
    },
    {
        id: 'token',
        name: 'Token',
        description: 'SPL-like token with mint, transfer, and burn',
        code: TOKEN_TEMPLATE,
        icon: '🪙'
    }
];
