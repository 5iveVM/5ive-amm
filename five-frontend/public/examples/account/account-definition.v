    account InitState {
        created_accounts: u64;
    }
    
create_account(payer: account @signer, @init new_account: account, state: InitState @mut, initial_value: u64) -> pubkey {
        state.created_accounts = state.created_accounts + 1;
require(initial_value > 0);
        return new_account;
    }
