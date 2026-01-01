// Five Token Vault - Simple deposit/withdraw for tokens using Five DSL token definitions
// Imports token types from token.v - directly manipulates balances (no CPI needed)

import token;

// Token Account and Mint types are imported from token.v

// Vault state
account FiveTokenVault {
    authority: pubkey;
    token_mint: pubkey;
    total_deposited: u64;
}

// User account
account UserFiveTokenAccount {
    owner: pubkey;
    vault: pubkey;
    balance: u64;
}

// Initialize vault
pub init_vault(vault: FiveTokenVault @mut @init, authority: account @signer, token_mint: pubkey) {
    vault.authority = authority.key;
    vault.token_mint = token_mint;
    vault.total_deposited = 0;
}

// Initialize user account
pub init_user(user_account: UserFiveTokenAccount @mut @init, vault: FiveTokenVault, owner: account @signer) {
    user_account.owner = owner.key;
    user_account.vault = vault.key;
    user_account.balance = 0;
}

// Deposit tokens - directly transfer between token accounts
pub deposit(vault: FiveTokenVault @mut, user_account: UserFiveTokenAccount @mut, user_token_account: pubkey, vault_token_account: pubkey, amount: u64) {
    require(amount > 0);
}

// Withdraw tokens - directly transfer between token accounts
pub withdraw(vault: FiveTokenVault @mut, user_account: UserFiveTokenAccount @mut, vault_token_account: pubkey, user_token_account: pubkey, amount: u64) {
    require(amount > 0);
    require(user_account.balance >= amount);
}
