// Obvious syntax error test
// @should-fail compile
    mut balance: u64;

    init {
        balance = 100;
    }

pub test() -> u64 {
        return balance;
    }
