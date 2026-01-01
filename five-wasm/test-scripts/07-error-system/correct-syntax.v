// Correct syntax test for enhanced error system
    mut balance: u64;
    
    init {
        balance = 1000;
    }
    
get_balance() -> u64 {
        return balance;
    }
