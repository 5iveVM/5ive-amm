// Correct syntax test for enhanced error system
account TestAccount {
    authority: pubkey;
    balance: u64;
    initialized: u64;
}

pub initialize(t: TestAccount @mut, authority: account @signer) {
    require(t.initialized == 0);
    t.authority = authority.key;
    t.balance = 1000;
    t.initialized = 1;
}

pub get_balance(t: TestAccount) -> u64 {
    return t.balance;
}
