// ============================================================================
// ESCROW TYPES
// ============================================================================

account Escrow {
    maker: pubkey;
    taker: pubkey;
    mint: pubkey;
    amount: u64;
    expires_at: u64;
    is_fulfilled: bool;
    is_cancelled: bool;
}
