script register_fused_test {
    field balance: u64;

    pub instruction test_fused(amount: u64) {
        // Pattern 1: let b = balance -> LOAD_FIELD_REG
        let b = balance;
        
        // Pattern 2: balance = balance - b -> SUB_FIELD_REG
        balance = balance - b;
        
        // Pattern 3: balance = balance + b -> ADD_FIELD_REG
        balance = balance + b;
        
        // Pattern 4: balance = b -> STORE_FIELD_REG
        balance = b;
    }
}
