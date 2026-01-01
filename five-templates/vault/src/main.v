// Native SOL Vault - Simple deposit/withdraw for native Solana tokens

// System Program interface for native SOL transfers via CPI
interface SystemProgram @program("11111111111111111111111111111112") {
    transfer @discriminator(2) (from: pubkey, to: pubkey, amount: u64);
}

// Vault state
account Vault {
    authority: pubkey;
    total_deposited: u64;
}

// User account
account UserAccount {
    owner: pubkey;
    vault: pubkey;
    balance: u64;
}

// Initialize vault
pub init_vault(vault: Vault @mut @init, authority: account @signer) {
    vault.authority = authority.key;
    vault.total_deposited = 0;
}

// Initialize user account
pub init_user(user_account: UserAccount @mut @init, vault: Vault, owner: account @signer) {
    user_account.owner = owner.key;
    user_account.vault = vault.key;
    user_account.balance = 0;
}

// Deposit native SOL
pub deposit(vault: Vault @mut, user_account: UserAccount @mut, payer: account @signer @mut, vault_pda: account @mut, amount: u64) {
    require(amount > 0);

    SystemProgram.transfer(payer, vault_pda, amount);

    vault.total_deposited = vault.total_deposited + amount;
    user_account.balance = user_account.balance + amount;
}

// Withdraw native SOL
pub withdraw(vault: Vault @mut, user_account: UserAccount @mut, withdrawer: account @signer, vault_pda: account @mut, amount: u64) {
    require(amount > 0);
    require(user_account.balance >= amount);

    SystemProgram.transfer(vault_pda, withdrawer, amount);

    vault.total_deposited = vault.total_deposited - amount;
    user_account.balance = user_account.balance - amount;
}
