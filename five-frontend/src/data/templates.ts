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

export const TOKEN_TEMPLATE = `// Token Example (adapted from five-templates/token.v)

account Mint {
    authority: pubkey;
    supply: u64;
    decimals: u8;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
}

pub init_mint(mint: Mint @mut, authority: account @signer, decimals: u8) {
    mint.authority = authority.key;
    mint.supply = 0;
    mint.decimals = decimals;
}

pub init_token_account(token_account: TokenAccount @mut, owner: account @signer, mint: account) {
    token_account.owner = owner.key;
    token_account.mint = mint.key;
    token_account.balance = 0;
}

pub mint_to(mint: Mint @mut, token_account: TokenAccount @mut, amount: u64) {
    token_account.balance = token_account.balance + amount;
    mint.supply = mint.supply + amount;
}

pub transfer(from_account: TokenAccount @mut, to_account: TokenAccount @mut, amount: u64) {
    from_account.balance = from_account.balance - amount;
    to_account.balance = to_account.balance + amount;
}

pub burn(mint: Mint @mut, token_account: TokenAccount @mut, amount: u64) {
    token_account.balance = token_account.balance - amount;
    mint.supply = mint.supply - amount;
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
