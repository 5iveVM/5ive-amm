// Initialize user vote account (if needed)
pub fn init_vote(
    vote: Vote @mut @init,
    market: Market,
    owner: account @signer
) {
    vote.market = market.key;
    vote.owner = owner.key;
    vote.yes_shares = 0;
    vote.no_shares = 0;
    vote.claimed = false;
}

// Vote YES
// Input logic: User sends `amount` (e.g., SOL/USDC). 
// For this template, we assume `amount` is just added to the pot and user gets 1:1 shares.
// In a real AMM prediction market, price would vary.
pub fn vote_yes(
    market: Market @mut,
    vote: Vote @mut,
    voter: account @signer,
    amount: u64
) {
    require(!market.resolved);
    require(get_clock().slot < market.resolution_time);
    require(vote.market == market.key);
    require(vote.owner == voter.key);
    require(amount > 0);

    // Update Market
    market.total_yes_shares = market.total_yes_shares + amount;
    market.total_pot = market.total_pot + amount;

    // Update User
    vote.yes_shares = vote.yes_shares + amount;
}

// Vote NO
pub fn vote_no(
    market: Market @mut,
    vote: Vote @mut,
    voter: account @signer,
    amount: u64
) {
    require(!market.resolved);
    require(get_clock().slot < market.resolution_time);
    require(vote.market == market.key);
    require(vote.owner == voter.key);
    require(amount > 0);

    // Update Market
    market.total_no_shares = market.total_no_shares + amount;
    market.total_pot = market.total_pot + amount;

    // Update User
    vote.no_shares = vote.no_shares + amount;
}

// Claim Winnings
pub fn claim_winnings(
    market: Market @mut,
    vote: Vote @mut,
    user: account @signer
) -> u64 {
    require(market.resolved);
    require(vote.market == market.key);
    require(vote.owner == user.key);
    require(!vote.claimed);

    let payout = 0;

    if (market.outcome_yes) {
        // YES Won
        if (vote.yes_shares > 0) {
            // Payout = (UserShares / TotalWinningShares) * TotalPot
            // Integer math: (UserShares * TotalPot) / TotalWinningShares
            payout = (vote.yes_shares * market.total_pot) / market.total_yes_shares;
        }
    } else {
        // NO Won
        if (vote.no_shares > 0) {
            payout = (vote.no_shares * market.total_pot) / market.total_no_shares;
        }
    }

    require(payout > 0);

    vote.claimed = true;
    
    return payout;
    // Logic note: In a real system, we would trigger a token transfer here.
    // For the template, returning the amount signifies the payout event.
}
