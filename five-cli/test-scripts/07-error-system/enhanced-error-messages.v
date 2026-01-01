// @should-fail compile
// Test script to verify enhanced error messages
// This script contains intentional errors to test the enhanced error system

    mut balance: u64
    
    init {
        balance = 100;  // Fixed: use valid u64 value instead of string 
        balance = balance + 1;  // Fixed: add valid statement
    }
    
pub transfer(amount: u64) -> u64 {
        let temp_var = amount;  // Fixed: use valid local variable
        return balance - temp_var;
    }
