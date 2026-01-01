// Test script to verify enhanced error messages

    account Test {
        balance: u64;
    }
    
    pub initialize(@init t: Test) {
        t.balance = 100;
        t.balance = t.balance + 1;
    }
    
    pub transfer(t: Test @mut, amount: u64) -> u64 {
        let temp_var = amount;
        return t.balance - temp_var;
    }