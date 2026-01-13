import launchpad_types;

// Initialize a new buying curve
pub fn init_curve(
    curve: launchpad_types::BondingCurve @mut @init
) {
    curve.virtual_sol_reserves = 30_000_000_000; // 30 SOL initial virtual liquidity
    curve.real_sol_reserves = 0;
    curve.virtual_token_reserves = 273_000_000_000_000; // Offset to set initial price
    curve.real_token_reserves = 793_100_000_000_000;    // Tokens available for sale (~80%)
    curve.token_total_supply = 1_000_000_000_000_000; // 1B tokens (6 decimals)
    curve.complete = false;
}

// Buy Tokens (Input SOL, Output Tokens)
pub fn buy(
    curve: launchpad_types::BondingCurve @mut,
    amount_sol_in: u64,
    min_tokens_out: u64
) -> u64 {
    require(!curve.complete);
    require(amount_sol_in > 0);

    // Calculate Output
    let sol_pool = curve.virtual_sol_reserves + curve.real_sol_reserves;
    let token_pool = curve.virtual_token_reserves + curve.real_token_reserves;
    let k = sol_pool * token_pool;

    let new_sol_pool = sol_pool + amount_sol_in;
    let new_token_pool = k / new_sol_pool;
    
    let tokens_out = token_pool - new_token_pool;

    // Validation
    require(tokens_out >= min_tokens_out);
    require(tokens_out <= curve.real_token_reserves);

    // Update State
    curve.real_sol_reserves = curve.real_sol_reserves + amount_sol_in;
    curve.real_token_reserves = curve.real_token_reserves - tokens_out;

    // Check completion
    if (curve.real_sol_reserves >= 85_000_000_000) { // Target ~85 SOL
        curve.complete = true;
    }

    return tokens_out;
}

// Sell Tokens (Input Tokens, Output SOL)
pub fn sell(
    curve: launchpad_types::BondingCurve @mut,
    amount_tokens_in: u64,
    min_sol_out: u64
) -> u64 {
    require(!curve.complete);
    require(amount_tokens_in > 0);

    // Calculate Output
    let sol_pool = curve.virtual_sol_reserves + curve.real_sol_reserves;
    let token_pool = curve.virtual_token_reserves + curve.real_token_reserves;
    let k = sol_pool * token_pool;

    let new_token_pool = token_pool + amount_tokens_in;
    let new_sol_pool = k / new_token_pool;
    
    let sol_out = sol_pool - new_sol_pool;

    // Validation
    require(sol_out >= min_sol_out);
    require(sol_out <= curve.real_sol_reserves);

    // Update State
    curve.real_sol_reserves = curve.real_sol_reserves - sol_out;
    curve.real_token_reserves = curve.real_token_reserves + amount_tokens_in;

    return sol_out;
}
