// Constant-product AMM skeleton (x * y = k)
// Focused on state transitions; token settlement is typically handled via
// external bytecode calls or interface CPI in caller instructions.

account Pool {
    token_a: u64;
    token_b: u64;
    total_shares: u64;
    fee_bps: u64;
}

pub init_pool(state: Pool @mut, fee_bps: u64) {
    state.token_a = 0;
    state.token_b = 0;
    state.total_shares = 0;
    state.fee_bps = fee_bps;
}

pub add_liquidity(state: Pool @mut, amount_a: u64, amount_b: u64) -> u64 {
    let shares = amount_a;
    state.token_a = state.token_a + amount_a;
    state.token_b = state.token_b + amount_b;
    state.total_shares = state.total_shares + shares;
    return shares;
}

pub swap(state: Pool @mut, amount_in: u64, a_for_b: bool) -> u64 {
    let fee = (amount_in * state.fee_bps) / 10000;
    let net_in = amount_in - fee;

    if (a_for_b) {
        state.token_a = state.token_a + net_in;
    } else {
        state.token_b = state.token_b + net_in;
    }

    return net_in;
}

pub quote_add_liquidity(state: Pool, amount_a: u64, amount_b: u64) -> u64 {
    if (amount_b < amount_a) {
        return amount_b;
    }
    return amount_a;
}

pub remove_liquidity(state: Pool @mut, share: u64) -> u64 {
    require(state.total_shares >= share);
    state.total_shares = state.total_shares - share;
    return share;
}
