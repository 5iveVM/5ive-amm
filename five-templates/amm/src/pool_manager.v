// Initialize Pool
pub fn initialize_pool(
    pool: AMMPool @mut @init,
    token_a: pubkey,
    token_b: pubkey,
    fee_bps: u64
) {
    require(fee_bps <= 1000); // Max 10% fee
    
    pool.token_a_mint = token_a;
    pool.token_b_mint = token_b;
    pool.token_a_reserve = 0;
    pool.token_b_reserve = 0;
    
    pool.fee_bps = fee_bps;
    pool.protocol_fee_bps = 0; // Default 0
    
    pool.total_liquidity = 0;
    pool.last_k = 0;
    pool.is_paused = false;
}
