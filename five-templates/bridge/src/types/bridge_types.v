// ============================================================================
// BRIDGE TYPES
// ============================================================================

account Bridge {
    authority: pubkey;
    wrapped_mint: pubkey;
    total_supply: u64;
    is_paused: bool;
    name: string<32>;
}

account WrappedAccount {
    owner: pubkey;
    bridge: pubkey;
    balance: u64;
}
