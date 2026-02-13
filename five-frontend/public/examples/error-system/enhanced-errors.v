// Valid script paired with syntax-error.v for error-system demos
account Test {
    authority: pubkey;
    balance: u64;
    initialized: u64;
}

pub initialize(t: Test @mut, authority: account @signer) {
    require(t.initialized == 0);
    t.authority = authority.key;
    t.balance = 100;
    t.balance = t.balance + 1;
    t.initialized = 1;
}

pub transfer(t: Test @mut, authority: account @signer, amount: u64) -> u64 {
    require(t.initialized > 0);
    require(t.authority == authority.key);
    require(amount > 0);
    require(t.balance > amount - 1);
    t.balance = t.balance - amount;
    return t.balance;
}
