// ============================================================================
// GOVERNANCE VOTING
// ============================================================================

pub vote(
    proposal: Proposal @mut,
    vote_record: VoteRecord @mut @init,
    voter: account @signer,
    weight: u64,
    support: bool
) {
    require(get_clock() >= proposal.start_slot);
    require(get_clock() <= proposal.end_slot);
    require(weight > 0);

    vote_record.proposal = proposal.key;
    vote_record.voter = voter.key;
    vote_record.weight = weight;
    vote_record.has_voted = true;

    if (support) {
        proposal.for_votes = proposal.for_votes + weight;
    } else {
        proposal.against_votes = proposal.against_votes + weight;
    }
}
