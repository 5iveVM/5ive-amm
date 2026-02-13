account InitState {
    admin: pubkey;
    created_accounts: u64;
    last_created: pubkey;
    initialized: u64;
}

pub create_account(
    payer: account @signer,
    new_account: account,
    state: InitState @mut,
    initial_value: u64
) -> pubkey {
    if (state.initialized == 0) {
        state.admin = payer.key;
        state.initialized = 1;
    }
    require(state.admin == payer.key);
    require(initial_value > 0);
    require(new_account.key != payer.key);
    state.created_accounts = state.created_accounts + 1;
    state.last_created = new_account.key;
    return new_account.key;
}
