// Bonding Curve State
account BondingCurve {
    virtual_sol_reserves: u64;
    real_sol_reserves: u64;
    virtual_token_reserves: u64;
    real_token_reserves: u64;
    token_total_supply: u64;
    complete: bool;
}

// Mint Account State
account Mint {
    authority: pubkey;
    freeze_authority: pubkey;
    supply: u64;
    decimals: u8;
    name: string<32>;
    symbol: string<32>;
    uri: string<128>;
}

// Token Account State
account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
    is_frozen: bool;
    delegate: pubkey;
    delegated_amount: u64;
    initialized: bool;
}
