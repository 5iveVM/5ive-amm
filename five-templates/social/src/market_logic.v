// Initialize a new Market
pub fn create_market(
    market: Market @mut @init,
    creator: account @signer,
    question: string,
    duration_seconds: u64
) {
    market.creator = creator.key;
    market.question = question;
    market.resolution_time = get_clock().slot + duration_seconds;
    market.resolved = false;
    market.outcome_yes = false; // Default
    
    market.total_yes_shares = 0;
    market.total_no_shares = 0;
    market.total_pot = 0;
}

// Resolve the market (Admin only for template simplicity)
pub fn resolve_market(
    market: Market @mut,
    resolver: account @signer,
    outcome_yes: bool
) {
    // Permission check (only creator resolve for this template)
    require(resolver.key == market.creator);
    
    // State check
    require(!market.resolved);
    require(get_clock().slot >= market.resolution_time);

    market.resolved = true;
    market.outcome_yes = outcome_yes;
}
