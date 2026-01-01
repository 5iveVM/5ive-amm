    account MutState {
        modification_count: u64;
    }
    
update_account(authority: account @signer, target: account @mut, state: MutState @mut, new_value: u64) -> u64 {
        // The @mut constraint allows us to modify the target account
        state.modification_count = state.modification_count + 1;
        
require(new_value > 0);
require(authority.key != target.key); // Authority cannot modify itself
        
        return new_value;
    }
    
batch_update(authority: account @signer, account1: account @mut, account2: account @mut, state: MutState @mut, value: u64) -> u64 {
        // Multiple mutable accounts can be modified in one transaction
        state.modification_count = state.modification_count + 2;
        
require(account1.key != account2.key); // Accounts must be different
        
        return state.modification_count;
    }
    
get_modification_count(state: MutState) -> u64 {
        return state.modification_count;
    }
