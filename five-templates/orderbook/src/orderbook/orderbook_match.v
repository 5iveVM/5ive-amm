// ============================================================================
// ORDERBOOK MATCHING
// ============================================================================

pub match_orders(
    bid: Order @mut,
    ask: Order @mut,
    amount: u64
) {
    require(!bid.is_cancelled);
    require(!ask.is_cancelled);
    require(bid.side == 0);
    require(ask.side == 1);
    require(bid.price >= ask.price);
    require(amount > 0);
    require(bid.amount >= bid.filled + amount);
    require(ask.amount >= ask.filled + amount);

    bid.filled = bid.filled + amount;
    ask.filled = ask.filled + amount;
}
