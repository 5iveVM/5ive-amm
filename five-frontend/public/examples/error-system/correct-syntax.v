// Correct syntax test for enhanced error system
account TestAccount {
    balance: u64;
}

pub initialize(t: TestAccount @mut) {
    t.balance = 1000;
}

pub get_balance(t: TestAccount) -> u64 {
    return t.balance;
}
