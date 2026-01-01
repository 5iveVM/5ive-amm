// ============================================================================
// TOKEN TYPES
// ============================================================================

account Mint {
    authority: pubkey;
    supply: u64;
    decimals: u8;
    name: string;
    symbol: string;
}

account TokenAccount {
    owner: pubkey;
    mint: pubkey;
    balance: u64;
    is_frozen: bool;
    delegate: pubkey;
    delegated_amount: u64;
    initialized: bool;
}
