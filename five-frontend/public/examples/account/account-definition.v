account InitState {
    created_accounts: u64;
    last_created: pubkey;
}

pub create_account(
    payer: account @signer,
    new_account: account,
    state: InitState @mut,
    initial_value: u64
) -> pubkey {
    require(initial_value > 0);
    state.created_accounts = state.created_accounts + 1;
    state.last_created = new_account.key;
    return new_account.key;
}
