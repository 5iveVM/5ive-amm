// Constant-product AMM skeleton (compilation-safe reference)

account Pool {
    token_a: u64;
    token_b: u64;
    total_shares: u64;
}

pub init_pool(state: Pool @mut) {
    state.token_a = 0;
    state.token_b = 0;
    state.total_shares = 0;
}

pub add_liquidity(state: Pool @mut, amount_a: u64, amount_b: u64) -> u64 {
    let shares = amount_a;
    state.token_a = state.token_a + amount_a;
    state.token_b = state.token_b + amount_b;
    state.total_shares = state.total_shares + shares;
    return shares;
}

pub remove_liquidity(state: Pool @mut, share: u64) -> u64 {
    state.total_shares = state.total_shares - share;
    return share;
}
