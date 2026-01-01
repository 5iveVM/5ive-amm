/// Five DSL Security Rules Example
/// 
/// This file demonstrates secure and insecure patterns in Five DSL,
/// showing how the compiler enforces security rules at compile-time.

// Import external contract's functions and fields
use token_contract::{balance, total_supply, transfer, approve};
use governance_contract::{proposal_count, vote, create_proposal};

// Define our contract's state  
account_data: u64 = 0;
owner: pubkey = @signer;
is_active: bool = true;

// ✅ SECURE PATTERNS - These will compile successfully

/// Read external state (Rule 1: Read-only external fields)
instruction get_external_info() -> (u64, u64, u64) {
    let user_balance = balance;          // ✅ Reading imported field
    let supply = total_supply;           // ✅ Reading imported field  
    let proposals = proposal_count;      // ✅ Reading imported field
    
    return (user_balance, supply, proposals);
}

/// Call external functions (Rule 2: Explicit function calls)
instruction safe_transfer(@signer account, recipient: pubkey, amount: u64) -> bool {
    // ✅ Explicit external function call
    let result = transfer(recipient, amount);
    
    // ✅ Update our own state
    if result {
        account_data = account_data + 1;  // Track successful transfers
    }
    
    return result;
}

/// Combine external reads with local logic  
instruction calculate_share(@signer account) -> u64 {
    let user_balance = balance;           // ✅ Read external state
    let supply = total_supply;            // ✅ Read external state
    
    // ✅ Local computation using external data
    if supply > 0 {
        return (user_balance * 100) / supply;  // Calculate percentage
    } else {
        return 0;
    }
}

/// Complex external interactions
instruction governance_action(@signer account, proposal_title: string, choice: u8) {
    // ✅ Create new proposal
    let proposal_id = create_proposal(proposal_title);
    
    // ✅ Vote on existing proposals  
    vote(proposal_id, choice);
    
    // ✅ Update local state
    is_active = true;
}

// ❌ INSECURE PATTERNS - These will cause compile-time errors

/// SECURITY VIOLATION: Direct external field mutation
instruction insecure_balance_mutation() {
    // ❌ This will cause a compile error:
    // "Cannot assign to imported field 'balance' - imported fields are read-only"
    // balance = 1000000;  // BLOCKED by Rule 1
}

/// SECURITY VIOLATION: Attempted external state corruption  
instruction insecure_supply_manipulation() {
    // ❌ This will cause a compile error:
    // "Cannot assign to imported field 'total_supply' - imported fields are read-only"  
    // total_supply = 0;  // BLOCKED by Rule 1
}

/// SECURITY VIOLATION: Multiple external field mutations
instruction multiple_violations(@signer account) {
    // ❌ All of these would be blocked:
    // balance = balance + 1000;           // BLOCKED - external field mutation
    // total_supply = total_supply * 2;    // BLOCKED - external field mutation
    // proposal_count = 999;               // BLOCKED - external field mutation
    
    // ✅ Correct approach: Use external functions
    transfer(account, 1000);              // Calls external contract's transfer function
}

// ✅ SECURE WORKAROUNDS - How to achieve desired outcomes securely

/// Secure way to modify external contract state
instruction secure_token_operations(@signer account, recipient: pubkey, amount: u64) {
    // Instead of: balance = balance - amount (BLOCKED)
    // Use: External function that enforces validation
    transfer(recipient, amount);  // ✅ Secure - goes through contract's logic
    
    // Instead of: total_supply = total_supply + amount (BLOCKED)  
    // Use: mint() function that handles supply increases
    // mint(amount);  // Would call external contract's mint function
}

/// Reading multiple external values safely
instruction portfolio_analysis() -> (u64, u64, bool) {
    // ✅ All reads are safe and allowed
    let my_balance = balance;
    let total = total_supply;
    let active_governance = proposal_count > 0;
    
    // ✅ Local computation with external data
    let is_whale = my_balance > (total / 100);  // Check if >1% holder
    
    return (my_balance, total, is_whale);
}

/// Complex multi-contract workflow (all secure)
instruction defi_operation(@signer account, amount: u64) -> bool {
    // ✅ Read current state
    let current_balance = balance;
    
    // ✅ Validate locally  
    require(current_balance >= amount);
    
    // ✅ Execute external operations through proper functions
    let transfer_success = transfer(account, amount);
    let approval_success = approve(account, amount * 2);
    
    // ✅ Update local state
    if transfer_success && approval_success {
        account_data = account_data + amount;
        return true;
    } else {
        return false;
    }
}

// Expected compiler output for security violations:
//
// 🔒 FIVE DSL SECURITY VIOLATIONS DETECTED:
// ==========================================
// 
// 1. 🚫 RULE 1 VIOLATION: External State Immutability
//    Cannot assign to imported field 'balance' from account token_contract  
//    💡 SOLUTION: To modify external contract state, call a function on account token_contract instead of directly assigning to field 'balance'
//    📖 REFERENCE: FIVE_SECURITY_RULES.md - Rule 1
//
// 📋 SECURITY REVIEW REQUIRED:
// - Review all violations above before deployment
// - Follow Five DSL Security Rules for secure patterns  
// - See FIVE_SECURITY_RULES.md for detailed guidance
// ==========================================