import amm_types;
import amm_math;

// Add Liquidity
pub fn add_liquidity(
    pool: amm_types::AMMPool @mut,
    lp_account: amm_types::LPTokenAccount @mut,
    provider: account @signer,
    amount_a_desired: u64,
    amount_b_desired: u64,
    min_liquidity: u64
) -> u64 {
    require(!pool.is_paused);
    
    let mut liquidity_minted = 0;
    
    if (pool.total_liquidity == 0) {
        // First deposit: K = x * y
        // Liquidity = sqrt(x * y)
        pool.token_a_reserve = amount_a_desired;
        pool.token_b_reserve = amount_b_desired;
        
        liquidity_minted = amm_math::sqrt(amount_a_desired * amount_b_desired);
        
        // Lock minimal liquidity (1000) to prevent inflation attack (Uniswap v2 style)
        // For simple template, we just mint it all to provider but careful users should burn some.
    } else {
        // Subsequent deposits: Proportional
        // amount_a_optimal = (amount_b_desired * reserve_a) / reserve_b
        let amount_a_optimal = (amount_b_desired * pool.token_a_reserve) / pool.token_b_reserve;
        
        require(amount_a_optimal <= amount_a_desired); // Ensure user has enough A
        
        // Actually deposit optimal amounts (simplified: take desired here for template brevity)
        // In robust DEX, we calculate exact amounts.
        
        // Liquidity = min((amount_a * total) / reserve_a, (amount_b * total) / reserve_b)
        let share_a = (amount_a_desired * pool.total_liquidity) / pool.token_a_reserve;
        let share_b = (amount_b_desired * pool.total_liquidity) / pool.token_b_reserve;
        
        if (share_a < share_b) {
            liquidity_minted = share_a;
        } else {
            liquidity_minted = share_b;
        }
        
        pool.token_a_reserve = pool.token_a_reserve + amount_a_desired;
        pool.token_b_reserve = pool.token_b_reserve + amount_b_desired;
    }
    
    require(liquidity_minted >= min_liquidity);
    
    // Update Global State
    pool.total_liquidity = pool.total_liquidity + liquidity_minted;
    pool.last_k = pool.token_a_reserve * pool.token_b_reserve;
    
    // Update User State
    lp_account.balance = lp_account.balance + liquidity_minted;
    
    return liquidity_minted;
}

// Remove Liquidity
pub fn remove_liquidity(
    pool: amm_types::AMMPool @mut,
    lp_account: amm_types::LPTokenAccount @mut,
    provider: account @signer,
    liquidity_amount: u64,
    min_a: u64,
    min_b: u64
) {
    require(lp_account.owner == provider.key);
    require(lp_account.balance >= liquidity_amount);
    
    // amount_a = (liquidity * reserve_a) / total_liquidity
    let amount_a = (liquidity_amount * pool.token_a_reserve) / pool.total_liquidity;
    let amount_b = (liquidity_amount * pool.token_b_reserve) / pool.total_liquidity;
    
    require(amount_a >= min_a);
    require(amount_b >= min_b);
    
    // Update Reserves
    pool.token_a_reserve = pool.token_a_reserve - amount_a;
    pool.token_b_reserve = pool.token_b_reserve - amount_b;
    pool.total_liquidity = pool.total_liquidity - liquidity_amount;
    pool.last_k = pool.token_a_reserve * pool.token_b_reserve;
    
    // Burn LP Tokens
    lp_account.balance = lp_account.balance - liquidity_amount;
}
