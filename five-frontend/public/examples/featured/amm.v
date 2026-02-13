// Constant-product AMM skeleton (compilation-safe reference)

account Pool {
    authority: pubkey;
    token_a: u64;
    token_b: u64;
    total_shares: u64;
    initialized: u64;
}

pub init_pool(state: Pool @mut, authority: account @signer) {
    require(state.initialized == 0);
    state.authority = authority.key;
    state.token_a = 0;
    state.token_b = 0;
    state.total_shares = 0;
    state.initialized = 1;
}

pub add_liquidity(state: Pool @mut, authority: account @signer, amount_a: u64, amount_b: u64) -> u64 {
    require(state.initialized > 0);
    require(state.authority == authority.key);
    require(amount_a > 0);
    require(amount_b > 0);
    let shares = amount_a;
    state.token_a = state.token_a + amount_a;
    state.token_b = state.token_b + amount_b;
    state.total_shares = state.total_shares + shares;
    return shares;
}

pub quote_swap_out(state: Pool, amount_in: u64, a_for_b: bool) -> u64 {
    require(state.initialized > 0);
    require(amount_in > 0);
    if (a_for_b) {
        require(state.token_a > 0);
        require(state.token_b > 0);
        return (amount_in * state.token_b) / (state.token_a + amount_in);
    }
    require(state.token_b > 0);
    require(state.token_a > 0);
    return (amount_in * state.token_a) / (state.token_b + amount_in);
}

pub swap(state: Pool @mut, authority: account @signer, amount_in: u64, a_for_b: bool) -> u64 {
    require(state.initialized > 0);
    require(state.authority == authority.key);
    let amount_out = quote_swap_out(state, amount_in, a_for_b);
    require(amount_out > 0);
    if (a_for_b) {
        require(state.token_b > amount_out - 1);
        state.token_a = state.token_a + amount_in;
        state.token_b = state.token_b - amount_out;
    } else {
        require(state.token_a > amount_out - 1);
        state.token_b = state.token_b + amount_in;
        state.token_a = state.token_a - amount_out;
    }
    return amount_out;
}

pub remove_liquidity(state: Pool @mut, authority: account @signer, share: u64) -> u64 {
    require(state.initialized > 0);
    require(state.authority == authority.key);
    require(share > 0);
    require(state.total_shares > share - 1);
    state.total_shares = state.total_shares - share;
    return share;
}
