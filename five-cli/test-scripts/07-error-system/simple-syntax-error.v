// @should-fail compile
// Simple syntax error test - missing semicolon
    mut balance: u64

    init {
        balance = 100  // Missing semicolon
    }

    pub test() -> u64 {
        return balance;
    }
