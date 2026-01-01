// ============================================================================
// ORACLE CORE
// ============================================================================

pub init_oracle(
    feed: OracleFeed @mut @init,
    authority: account @signer,
    max_staleness_slots: u64,
    name: string
) -> pubkey {
    feed.authority = authority.key;
    feed.price = 0;
    feed.last_update_slot = 0;
    feed.max_staleness_slots = max_staleness_slots;
    feed.is_paused = false;
    feed.name = name;
    return feed.key;
}

pub update_price(
    feed: OracleFeed @mut,
    authority: account @signer,
    price: u64
) {
    require(feed.authority == authority.key);
    require(!feed.is_paused);
    feed.price = price;
    feed.last_update_slot = get_clock();
}

pub get_price(feed: OracleFeed) -> u64 {
    if (feed.last_update_slot == 0) {
        return 0;
    }
    let now: u64 = get_clock();
    require(now >= feed.last_update_slot);
    require((now - feed.last_update_slot) <= feed.max_staleness_slots);
    return feed.price;
}
