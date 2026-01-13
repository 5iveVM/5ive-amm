// ============================================================================
// GOVERNANCE TYPES
// ============================================================================

account Governance {
    authority: pubkey;
    token_mint: pubkey;
    proposal_count: u64;
    quorum_bps: u64;
    voting_period_slots: u64;
    is_paused: bool;
    name: string<32>;
}

account Proposal {
    governance: pubkey;
    proposer: pubkey;
    start_slot: u64;
    end_slot: u64;
    for_votes: u64;
    against_votes: u64;
    executed: bool;
    description: string<128>;
}

account VoteRecord {
    proposal: pubkey;
    voter: pubkey;
    weight: u64;
    has_voted: bool;
}
