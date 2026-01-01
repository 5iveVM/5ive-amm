    account InitState {
        created_accounts: u64;
    }
    
create_account(payer: account @signer, new_account: account @init, state: InitState @mut, initial_value: u64) -> pubkey {
        state.created_accounts = state.created_accounts + 1;
        
        // Initialize the new account with data
        // The @init constraint ensures the account doesn't already exist
require(initial_value > 0);
        
        return new_account;
    }
    
get_created_count(state: InitState) -> u64 {
        return state.created_accounts;
    }
