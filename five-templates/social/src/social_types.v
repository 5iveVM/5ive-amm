// Market Account
account Market {
    creator: pubkey;
    question: string;
    resolution_time: u64;
    resolved: bool;
    outcome_yes: bool;     // true = YES won, false = NO won (only valid if resolved=true)
    
    // Pool State
    total_yes_shares: u64;
    total_no_shares: u64;
    total_pot: u64;        // Total funds (usually normalized to base currency units)
}

// User Vote Account
account Vote {
    market: pubkey;        // Which market this vote is for
    owner: pubkey;         // Who owns this vote
    yes_shares: u64;       // Number of YES shares
    no_shares: u64;        // Number of NO shares
    claimed: bool;         // Has the user claimed winnings?
}
