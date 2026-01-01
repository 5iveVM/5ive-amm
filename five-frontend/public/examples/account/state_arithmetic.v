    account MutState {
        modification_count: u64;
    }
    
update_account(authority: account @signer, target: account @mut, state: MutState @mut, new_value: u64) -> u64 {
        state.modification_count = state.modification_count + 1;
require(new_value > 0);
require(authority.key != target.key);
        return new_value;
    }
