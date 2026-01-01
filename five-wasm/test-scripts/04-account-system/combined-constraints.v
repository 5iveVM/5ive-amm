    account CombinedState {
        total_operations: u64;
        admin: pubkey;
    }
    
create_and_fund(payer: account @signer, new_account: account @init, state: CombinedState @mut, initial_amount: u64) -> pubkey {
        // @init ensures account doesn't exist and allows modification (implicit @mut)
require(payer.key == state.admin);
require(initial_amount >= 100);
        
        state.total_operations = state.total_operations + 1;
        
        return new_account.key;
    }
    
transfer_with_authority(authority: account @signer, from: account @mut, to: account @mut, state: CombinedState @mut, amount: u64) -> bool {
        // Authority must sign, both accounts must be mutable
require(authority.key == state.admin);
require(from.key != to.key);
require(amount > 0);
        
        state.total_operations = state.total_operations + 1;
        
        return true;
    }
    
admin_override(current_admin: account @signer, new_admin_account: account @init, state: CombinedState @mut) -> pubkey {
        // Change admin to a new account that must be initialized
require(current_admin.key == state.admin);
        
        state.admin = new_admin_account.key;
        state.total_operations = state.total_operations + 1;
        
        return state.admin;
    }
