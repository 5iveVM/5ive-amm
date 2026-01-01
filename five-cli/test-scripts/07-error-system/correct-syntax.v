// @should-fail compile
// Correct syntax test for enhanced error system - global state without init params
    mut balance: u64;

    init {
        balance = 1000;
    }

pub get_balance() -> u64 {
        return balance;
    }
