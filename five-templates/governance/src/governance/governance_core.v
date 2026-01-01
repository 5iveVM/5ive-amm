// ============================================================================
// GOVERNANCE CORE
// ============================================================================

pub init_governance(
    governance: Governance @mut @init,
    authority: account @signer,
    token_mint: pubkey,
    quorum_bps: u64,
    voting_period_slots: u64,
    name: string
) -> pubkey {
    require(quorum_bps <= 10000);
    governance.authority = authority.key;
    governance.token_mint = token_mint;
    governance.proposal_count = 0;
    governance.quorum_bps = quorum_bps;
    governance.voting_period_slots = voting_period_slots;
    governance.is_paused = false;
    governance.name = name;
    return governance.key;
}

pub create_proposal(
    governance: Governance @mut,
    proposal: Proposal @mut @init,
    proposer: account @signer,
    description: string
) -> pubkey {
    require(!governance.is_paused);
    governance.proposal_count = governance.proposal_count + 1;
    proposal.governance = governance.key;
    proposal.proposer = proposer.key;
    proposal.start_slot = get_clock();
    proposal.end_slot = get_clock() + governance.voting_period_slots;
    proposal.for_votes = 0;
    proposal.against_votes = 0;
    proposal.executed = false;
    proposal.description = description;
    return proposal.key;
}

pub finalize_proposal(
    proposal: Proposal @mut,
    authority: account @signer
) {
    require(authority.key != 0);
    require(get_clock() >= proposal.end_slot);
    proposal.executed = true;
}
