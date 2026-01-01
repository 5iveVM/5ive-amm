    // Basic vault with proper Solana signer patterns
    // @should-fail compile
    
    // Basic vault with proper Solana signer patterns
    
    account Vault {
        balance: u64;
        authorized_user: pubkey;
    }
    
    pub initialize(@init vault: Vault, authority: account @signer) {
        vault.balance = 0;
        vault.authorized_user = authority.key;
    }
    
    // Set authorized user (requires signer authority)
    pub set_authorized_user(vault: Vault @mut, authority: account @signer, new_authority: pubkey) {
        require(authority.key == vault.authorized_user);
        vault.authorized_user = new_authority;
    }
    
    // Simplified withdraw - requires authorization
    pub withdraw(vault: Vault @mut, authority: account @signer, amount: u64) {
        require(authority.key == vault.authorized_user);
        require(vault.balance >= amount);
        vault.balance = vault.balance - amount;
    }
    
    // Deposit to vault (anyone can deposit)
    pub deposit(vault: Vault @mut, amount: u64) {
        vault.balance = vault.balance + amount;
    }
    
    // Get current balance
    pub get_balance(vault: Vault) -> u64 {
        return vault.balance;
    }