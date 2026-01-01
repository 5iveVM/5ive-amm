// AMM Pool State
account AMMPool {
    token_a_mint: pubkey;
    token_b_mint: pubkey;
    
    token_a_reserve: u64;
    token_b_reserve: u64;
    
    fee_bps: u64;           // Basis points (e.g. 30 = 0.3%)
    protocol_fee_bps: u64;  // Portion of fee going to protocol
    
    lp_mint: pubkey;        // Mint for LP tokens representing shares
    total_liquidity: u64;
    
    is_paused: bool;
    
    // Invariant tracking
    last_k: u64;
}

// LP Token Account (User's share)
account LPTokenAccount {
    pool: pubkey;
    owner: pubkey;
    balance: u64;
}
