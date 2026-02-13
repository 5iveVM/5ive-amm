account MutState {
    authority: pubkey;
    modification_count: u64;
    last_value: u64;
    initialized: u64;
}

pub update_account(
    authority: account @signer,
    target: account @mut,
    state: MutState @mut,
    new_value: u64
) -> u64 {
    if (state.initialized == 0) {
        state.authority = authority.key;
        state.initialized = 1;
    }
    require(state.authority == authority.key);
    require(new_value > 0);
    require(authority.key != target.key);
    state.modification_count = state.modification_count + 1;
    state.last_value = new_value;
    return state.last_value;
}
