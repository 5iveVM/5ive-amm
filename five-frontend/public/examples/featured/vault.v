// Basic vault example with signer-gated withdrawals.

account Vault {
    balance: u64;
    authorized_user: pubkey;
}

pub initialize(vault: Vault @mut @init, authority: account @signer) {
    vault.balance = 0;
    vault.authorized_user = authority.key;
}

pub set_authorized_user(vault: Vault @mut, authority: account @signer, new_authority: pubkey) {
    require(authority.key == vault.authorized_user);
    vault.authorized_user = new_authority;
}

pub withdraw(vault: Vault @mut, authority: account @signer, amount: u64) {
    require(authority.key == vault.authorized_user);
    require(vault.balance >= amount);
    vault.balance = vault.balance - amount;
}

pub deposit(vault: Vault @mut, amount: u64) {
    require(amount > 0);
    vault.balance = vault.balance + amount;
}

pub get_balance(vault: Vault) -> u64 {
    return vault.balance;
}
