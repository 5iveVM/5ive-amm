// Valid script paired with syntax-error.v for error-system demos
account Test {
    balance: u64;
}

pub initialize(t: Test @mut) {
    t.balance = 100;
    t.balance = t.balance + 1;
}

pub transfer(t: Test @mut, amount: u64) -> u64 {
    t.balance = t.balance + amount;
    return t.balance;
}
