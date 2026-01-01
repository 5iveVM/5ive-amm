    account AdvancedInitState {
        total_accounts: u64;
        total_funded: u64;
        last_created: pubkey;
    }

pub create_and_fund_account(payer: account @signer, new_account: account @init, state: AdvancedInitState @mut, initial_value: u64, funded_amount: u64) -> (pubkey, u64) {
        require(initial_value > 0);
        require(funded_amount > 0);

        state.total_accounts = state.total_accounts + 1;
        state.total_funded = state.total_funded + funded_amount;
        state.last_created = new_account.key;

        let total_allocated = initial_value + funded_amount;
        return (new_account.key, total_allocated);
    }

pub get_account_stats(state: AdvancedInitState) -> (u64, u64, pubkey) {
        return (state.total_accounts, state.total_funded, state.last_created);
    }
