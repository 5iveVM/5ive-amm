// Basic vault example.

account Vault {
    balance: u64;
    min_withdraw: u64;
}

pub initialize(vault: Vault @mut, min_withdraw: u64) {
    vault.balance = 0;
    vault.min_withdraw = min_withdraw;
}

pub withdraw(vault: Vault @mut, amount: u64) {
    vault.balance = vault.balance - amount;
}

pub deposit(vault: Vault @mut, amount: u64) {
    vault.balance = vault.balance + amount;
}

pub get_balance(vault: Vault) -> u64 {
    return vault.balance;
}
