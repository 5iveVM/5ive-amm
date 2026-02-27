account FeaturedVault {
    authority: pubkey;
    balance: u64;
}

pub create_featured_vault(vault: FeaturedVault @mut, authority: pubkey, amount: u64) -> u64 {
    vault.authority = authority;
    vault.balance = amount;
    return vault.balance;
}
