// English auction template (simplified)

account AuctionState {
    seller: pubkey;
    end_time: u64;
    min_increment: u64;
    highest_bid: u64;
    highest_bidder: pubkey;
    settled: bool;
}

// Initialize auction parameters
init_auction(state: AuctionState @mut, seller: pubkey, end_time: u64, min_increment: u64, reserve: u64) {
    state.seller = seller;
    state.end_time = end_time;
    state.min_increment = min_increment;
    state.highest_bid = reserve;
    state.highest_bidder = seller;
    state.settled = false;
}

// Place bid (accounting only)
bid(state: AuctionState @mut, bidder: pubkey, amount: u64) {
    let now = get_clock();
    require(now < state.end_time);
    require(amount >= state.highest_bid + state.min_increment);
    state.highest_bid = amount;
    state.highest_bidder = bidder;
}

// Settle after auction end (no transfers here; just flag)
settle(state: AuctionState @mut) {
    let now = get_clock();
    require(now >= state.end_time);
    require(!state.settled);
    state.settled = true;
}

