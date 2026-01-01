    account AdvancedMutState {
        modification_count: u64;
        total_value_modified: u64;
        last_modifier: pubkey;
        version: u64;
    }

pub update_with_versioning(authority: account @signer, target: account @mut, state: AdvancedMutState @mut, new_value: u64) -> (u64, u64) {
        require(new_value > 0);
        require(authority.key != target.key);

        state.modification_count = state.modification_count + 1;
        state.total_value_modified = state.total_value_modified + new_value;
        state.last_modifier = authority.key;
        state.version = state.version + 1;

        return (state.version, new_value);
    }

pub bulk_update(authority: account @signer, account1: account @mut, account2: account @mut, account3: account @mut, state: AdvancedMutState @mut, value: u64) -> u64 {
        require(account1.key != account2.key);
        require(account2.key != account3.key);
        require(account1.key != account3.key);

        state.modification_count = state.modification_count + 3;
        state.total_value_modified = state.total_value_modified + (value * 3);
        state.last_modifier = authority.key;
        state.version = state.version + 1;

        return state.modification_count;
    }

pub get_state_info(state: AdvancedMutState) -> (u64, u64, u64, pubkey) {
        return (state.modification_count, state.total_value_modified, state.version, state.last_modifier);
    }
