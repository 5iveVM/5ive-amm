// ============================================================================
// ORDERBOOK TYPES
// ============================================================================

account Orderbook {
    authority: pubkey;
    base_mint: pubkey;
    quote_mint: pubkey;
    next_order_id: u64;
    is_paused: bool;
    name: string;
}

account Order {
    orderbook: pubkey;
    owner: pubkey;
    side: u8;          // 0 = bid, 1 = ask
    price: u64;
    amount: u64;
    filled: u64;
    is_cancelled: bool;
}
