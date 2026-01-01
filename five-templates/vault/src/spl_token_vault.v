// SPL Token Vault - Simple deposit/withdraw for any SPL token

// SPL Token Program interface for token transfers via CPI
interface SPLToken @program("TokenkegQfeZyiNwAJsyFbPVwwQQfzzTtKF2WwZvD") {
    transfer @discriminator(3) (source: pubkey, destination: pubkey, authority: pubkey, amount: u64);
}

// Vault state
account TokenVault {
    authority: pubkey;
    token_mint: pubkey;
    total_deposited: u64;
}

// User account
account UserTokenAccount {
    owner: pubkey;
    vault: pubkey;
    balance: u64;
}

// Initialize vault
pub init_vault(vault: TokenVault @mut @init, authority: account @signer, token_mint: pubkey) {
    vault.authority = authority.key;
    vault.token_mint = token_mint;
    vault.total_deposited = 0;
}

// Initialize user account
pub init_user(user_account: UserTokenAccount @mut @init, vault: TokenVault, owner: account @signer) {
    user_account.owner = owner.key;
    user_account.vault = vault.key;
    user_account.balance = 0;
}

// Deposit tokens
pub deposit(vault: TokenVault @mut, user_account: UserTokenAccount @mut, user_token_account: pubkey, vault_token_account: pubkey, owner: account @signer, amount: u64) {
    require(amount > 0);

    SPLToken.transfer(user_token_account, vault_token_account, owner, amount);

    vault.total_deposited = vault.total_deposited + amount;
    user_account.balance = user_account.balance + amount;
}

// Withdraw tokens
pub withdraw(vault: TokenVault @mut, user_account: UserTokenAccount @mut, vault_token_account: pubkey, user_token_account: pubkey, vault_authority: account @signer, amount: u64) {
    require(amount > 0);
    require(user_account.balance >= amount);

    SPLToken.transfer(vault_token_account, user_token_account, vault_authority, amount);

    vault.total_deposited = vault.total_deposited - amount;
    user_account.balance = user_account.balance - amount;
}
