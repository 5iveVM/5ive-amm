// Basic vault example.

account Vault {
    authority: pubkey;
    balance: u64;
    min_withdraw: u64;
    initialized: u64;
}

pub initialize(vault: Vault @mut, authority: account @signer, min_withdraw: u64) {
    require(vault.initialized == 0);
    require(min_withdraw > 0);
    vault.authority = authority.key;
    vault.balance = 0;
    vault.min_withdraw = min_withdraw;
    vault.initialized = 1;
}

pub withdraw(vault: Vault @mut, authority: account @signer, amount: u64) {
    require(vault.initialized > 0);
    require(vault.authority == authority.key);
    require(amount > 0);
    require(amount > vault.min_withdraw - 1);
    require(vault.balance > amount - 1);
    vault.balance = vault.balance - amount;
}

pub deposit(vault: Vault @mut, authority: account @signer, amount: u64) {
    require(vault.initialized > 0);
    require(vault.authority == authority.key);
    require(amount > 0);
    vault.balance = vault.balance + amount;
}

pub get_balance(vault: Vault) -> u64 {
    return vault.balance;
}
