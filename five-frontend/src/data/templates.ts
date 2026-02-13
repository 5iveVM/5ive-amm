/**
 * Five DSL Templates
 * Counter and Token templates from five-templates
 */

export const COUNTER_TEMPLATE = `// Counter Template (security-hardened)

account Counter {
    authority: pubkey;
    count: u64;
    initialized: u64;
}

pub initialize(counter: Counter @mut, owner: account @signer) {
    require(counter.initialized == 0);
    counter.authority = owner.key;
    counter.count = 0;
    counter.initialized = 1;
}

pub increment(counter: Counter @mut, owner: account @signer) {
    require(counter.initialized > 0);
    require(counter.authority == owner.key);
    counter.count = counter.count + 1;
}

pub decrement(counter: Counter @mut, owner: account @signer) {
    require(counter.initialized > 0);
    require(counter.authority == owner.key);
    if (counter.count > 0) {
        counter.count = counter.count - 1;
    }
}

pub add_amount(counter: Counter @mut, owner: account @signer, amount: u64) {
    require(counter.initialized > 0);
    require(counter.authority == owner.key);
    require(amount > 0);
    counter.count = counter.count + amount;
}

pub get_count(counter: Counter) -> u64 {
    return counter.count;
}

pub reset(counter: Counter @mut, owner: account @signer) {
    require(counter.initialized > 0);
    require(counter.authority == owner.key);
    counter.count = 0;
}
`;

export const TOKEN_TEMPLATE = `// Token Example (adapted from five-templates/token.v)

account Mint {
    authority: pubkey;
    supply: u64;
    decimals: u8;
    initialized: u64;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
    initialized: u64;
}

pub init_mint(mint: Mint @mut, authority: account @signer, decimals: u8) {
    require(mint.initialized == 0);
    require(decimals < 19);
    mint.authority = authority.key;
    mint.supply = 0;
    mint.decimals = decimals;
    mint.initialized = 1;
}

pub init_token_account(token_account: TokenAccount @mut, owner: account @signer, mint: account) {
    require(token_account.initialized == 0);
    token_account.owner = owner.key;
    token_account.mint = mint.key;
    token_account.balance = 0;
    token_account.initialized = 1;
}

pub mint_to(
    mint: Mint @mut,
    mint_account: account,
    token_account: TokenAccount @mut,
    authority: account @signer,
    amount: u64
) {
    require(mint.initialized > 0);
    require(token_account.initialized > 0);
    require(mint.authority == authority.key);
    require(token_account.mint == mint_account.key);
    require(amount > 0);
    token_account.balance = token_account.balance + amount;
    mint.supply = mint.supply + amount;
}

pub transfer(from_account: TokenAccount @mut, to_account: TokenAccount @mut, owner: account @signer, amount: u64) {
    require(from_account.initialized > 0);
    require(to_account.initialized > 0);
    require(from_account.owner == owner.key);
    require(from_account.mint == to_account.mint);
    require(amount > 0);
    require(from_account.balance > amount - 1);
    from_account.balance = from_account.balance - amount;
    to_account.balance = to_account.balance + amount;
}

pub burn(
    mint: Mint @mut,
    mint_account: account,
    token_account: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(mint.initialized > 0);
    require(token_account.initialized > 0);
    require(token_account.mint == mint_account.key);
    require(token_account.owner == owner.key);
    require(amount > 0);
    require(token_account.balance > amount - 1);
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
