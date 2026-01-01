// Simple syntax error test - invalid syntax
    mut balance: u64
    
    init {
        balance = 100;
        invalid_token_here #%@  // Actually invalid syntax
    }
