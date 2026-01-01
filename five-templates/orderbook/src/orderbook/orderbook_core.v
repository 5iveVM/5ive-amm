// ============================================================================
// ORDERBOOK CORE
// ============================================================================

pub init_orderbook(
    orderbook: Orderbook @mut @init,
    authority: account @signer,
    base_mint: pubkey,
    quote_mint: pubkey,
    name: string
) -> pubkey {
    orderbook.authority = authority.key;
    orderbook.base_mint = base_mint;
    orderbook.quote_mint = quote_mint;
    orderbook.next_order_id = 1;
    orderbook.is_paused = false;
    orderbook.name = name;
    return orderbook.key;
}

pub place_order(
    orderbook: Orderbook @mut,
    order: Order @mut @init,
    owner: account @signer,
    side: u8,
    price: u64,
    amount: u64
) -> pubkey {
    require(!orderbook.is_paused);
    require(side == 0 || side == 1);
    require(price > 0);
    require(amount > 0);

    order.orderbook = orderbook.key;
    order.owner = owner.key;
    order.side = side;
    order.price = price;
    order.amount = amount;
    order.filled = 0;
    order.is_cancelled = false;

    orderbook.next_order_id = orderbook.next_order_id + 1;
    return order.key;
}

pub cancel_order(order: Order @mut, owner: account @signer) {
    require(!order.is_cancelled);
    require(order.owner == owner.key);
    order.is_cancelled = true;
}
