    // Basic vault with proper Solana signer patterns
    // @should-fail compile
    
    mut balance: u64;
    mut authorized_user: pubkey;
    
    init {
        balance = 0; // Start with zero balance
        authorized_user = 0; // Default placeholder - will be set by set_authorized_user
    }
    
    // Set authorized user (requires signer authority)
set_authorized_user(authority: account @signer) {
        authorized_user = authority.key;
    }
    
    // Withdraw requires authorization
withdraw(authority: account @signer, amount: u64) {
require(authority.key == authorized_user);
require(balance >= amount);
        balance = balance - amount;
    }
    
    // Deposit to vault (anyone can deposit)
pub deposit(amount: u64) {
        balance = balance + amount;
    }
    
    // Get current balance
get_balance() -> u64 {
        return balance;
    }
    
    constraints {
require(balance >= 0);
    }
