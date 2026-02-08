// Field operations example (script fields and account type fields).

    // ============================================================================
    // SCRIPT-LEVEL FIELDS (stored in script bytecode account)
    // ============================================================================
    mut protocol_version: u64;
    mut total_transactions: u64;
    mut accumulated_fees: u64;
    mut last_update_timestamp: u64;
    
    // ============================================================================
    // ACCOUNT TYPE DEFINITIONS (external account field editing)
    // ============================================================================
    account TokenAccount {
        mint: u64;
        owner: u64;
        amount: u64;
        state: u8;
    }
    
    account UserProfile {
        user_id: u64;
        total_deposits: u64;
        reward_points: u64;
        last_activity: u64;
    }
    
    account SystemConfig {
        admin: u64;
        fee_rate: u64;
        max_transaction_size: u64;
        emergency_pause: bool;
    }
    
    // ============================================================================
    // MIXED FIELD OPERATIONS - Both script fields AND account fields
    // ============================================================================
    
    // Example 1: Script field editing only
initialize_protocol(version: u64, admin_fee: u64) {
        // These modify the SCRIPT'S OWN bytecode account fields
        protocol_version = version;
        total_transactions = 0;
        accumulated_fees = admin_fee;
        last_update_timestamp = 1000000; // Mock timestamp
    }
    
    // Example 2: Account field editing only  
initialize_token_account(
        token: TokenAccount,
        mint_address: u64,
        owner_address: u64,
        initial_amount: u64
    ) {
        // These modify the EXTERNAL token account's fields
        token.mint = mint_address;
        token.owner = owner_address;
        token.amount = initial_amount;
        token.state = 1; // Active state
    }
    
    // Example 3: BOTH script fields AND account fields in same function
process_deposit(
        user: UserProfile,
        token: TokenAccount,
        deposit_amount: u64,
        fee_amount: u64
    ) {
require(deposit_amount > fee_amount);
require(token.amount >= deposit_amount);
        
        // SCRIPT FIELD OPERATIONS (modify script's bytecode account)
        total_transactions = total_transactions + 1;
        accumulated_fees = accumulated_fees + fee_amount;
        last_update_timestamp = last_update_timestamp + 1;
        
        // ACCOUNT FIELD OPERATIONS (modify external accounts)
        token.amount = token.amount - deposit_amount;
        user.total_deposits = user.total_deposits + deposit_amount - fee_amount;
        user.reward_points = user.reward_points + 10; // Bonus points
        user.last_activity = last_update_timestamp; // Use script field value!
    }
    
    // Example 4: Complex interaction between script and account fields
calculate_user_rewards(
        user: UserProfile,
        config: SystemConfig
    ) -> u64 {
        // Read script fields for calculation base
require(protocol_version >= 1);
require(total_transactions > 0);
        
        // Read account fields
require(!config.emergency_pause);
require(user.total_deposits > 0);
        
        // Calculate rewards using both script and account data
        // Script fields: protocol_version, total_transactions
        // Account fields: user.total_deposits, config.fee_rate
        
        user.reward_points = user.reward_points + user.total_deposits / 100;
        user.last_activity = total_transactions; // Cross-reference script field
        
        // Update script tracking
        last_update_timestamp = total_transactions + user.total_deposits;
        
        return user.reward_points;
    }
    
    // Example 5: Batch operations on both types
batch_process_transactions(
        token1: TokenAccount,
        token2: TokenAccount,
        user1: UserProfile,
        user2: UserProfile,
        amount: u64
    ) {
        // Update script-level counters (script bytecode account)
        total_transactions = total_transactions + 2;
        accumulated_fees = accumulated_fees + (amount / 100);
        
        // Process first transaction (external accounts)
        token1.amount = token1.amount - amount;
        token2.amount = token2.amount + amount;
        user1.total_deposits = user1.total_deposits - amount;
        user2.total_deposits = user2.total_deposits + amount;
        
        // Update activity timestamps using script field
        user1.last_activity = total_transactions;
        user2.last_activity = total_transactions;
        
        // Final script field update
        last_update_timestamp = total_transactions * 1000;
    }
    
    // Example 6: Read-only operations demonstrating field access patterns
get_protocol_stats() -> (u64, u64, u64) {
        // Return script fields (read from script bytecode account)
return (protocol_version, total_transactions, accumulated_fees);
    }
    
get_account_summary(user: UserProfile, token: TokenAccount) -> (u64, u64) {
        // Return account fields (read from external accounts)
return (user.total_deposits, token.amount);
    }
    
    // Example 7: Conditional logic using both field types
emergency_pause_check(
        config: SystemConfig,
        user: UserProfile
    ) {
        // Check script field conditions
        if total_transactions > 10000 {
            // Update script field
            protocol_version = protocol_version + 1;
            
            // Update account field based on script state
            config.emergency_pause = true;
            config.max_transaction_size = accumulated_fees / 100;
        }
        
        // Cross-validate using both types
        if user.total_deposits > accumulated_fees {
            user.reward_points = user.reward_points * 2;
            total_transactions = total_transactions + user.total_deposits;
        }
    }
    
    // Example 8: Advanced field manipulation
advanced_field_operations(
        token: TokenAccount,
        user: UserProfile,
        config: SystemConfig,
        multiplier: u64
    ) {
        // Complex script field calculations
        protocol_version = (total_transactions + accumulated_fees) / multiplier;
        
        // Complex account field calculations  
        token.amount = (user.total_deposits * config.fee_rate) / 10000;
        user.reward_points = token.amount + protocol_version; // Mix both types!
        
        // Update all timestamps to script field value
        user.last_activity = last_update_timestamp;
        last_update_timestamp = protocol_version + total_transactions;
    }
