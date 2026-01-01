    account AdvancedSignerState {
        owner: pubkey;
        authorized_signers: u64;
        last_signer: pubkey;
        transaction_count: u64;
    }

pub multi_sig_operation(primary: account @signer, secondary: account @signer, state: AdvancedSignerState @mut, amount: u64) -> u64 {
        require(amount > 0);
        require(amount <= 10000);
        require(primary.key != secondary.key);
        require(primary.key == state.owner);

        state.transaction_count = state.transaction_count + 1;
        state.last_signer = secondary.key;

        return amount * state.transaction_count;
    }

pub authorize_signer(owner: account @signer, new_signer: pubkey, state: AdvancedSignerState @mut) -> u64 {
        require(owner.key == state.owner);

        state.authorized_signers = state.authorized_signers + 1;
        state.last_signer = new_signer;

        return state.authorized_signers;
    }

pub delegated_operation(delegate: account @signer, state: AdvancedSignerState @mut, amount: u64) -> (u64, pubkey) {
        require(state.authorized_signers > 0);
        require(amount <= 5000);

        state.transaction_count = state.transaction_count + 1;
        state.last_signer = delegate.key;

        return (state.transaction_count, delegate.key);
    }

pub get_signer_state(state: AdvancedSignerState) -> (u64, u64, pubkey, pubkey) {
        return (state.authorized_signers, state.transaction_count, state.owner, state.last_signer);
    }
