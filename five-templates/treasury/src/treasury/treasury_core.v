// ============================================================================
// TREASURY CORE
// ============================================================================

pub init_treasury(
    treasury: Treasury @mut @init,
    authority: account @signer,
    min_signers: u64,
    name: string
) -> pubkey {
    require(min_signers > 0);
    treasury.authority = authority.key;
    treasury.min_signers = min_signers;
    treasury.proposal_count = 0;
    treasury.is_paused = false;
    treasury.name = name;
    return treasury.key;
}

pub create_spend(
    treasury: Treasury @mut,
    proposal: SpendProposal @mut @init,
    proposer: account @signer,
    amount: u64,
    destination: pubkey
) -> pubkey {
    require(!treasury.is_paused);
    require(amount > 0);
    treasury.proposal_count = treasury.proposal_count + 1;
    proposal.treasury = treasury.key;
    proposal.proposer = proposer.key;
    proposal.amount = amount;
    proposal.destination = destination;
    proposal.approvals = 0;
    proposal.executed = false;
    return proposal.key;
}

pub approve_spend(
    proposal: SpendProposal @mut,
    approval: Approval @mut @init,
    signer: account @signer
) {
    require(!proposal.executed);
    approval.proposal = proposal.key;
    approval.signer = signer.key;
    proposal.approvals = proposal.approvals + 1;
}

pub execute_spend(
    treasury: Treasury,
    proposal: SpendProposal @mut
) {
    require(!proposal.executed);
    require(proposal.treasury == treasury.key);
    require(proposal.approvals >= treasury.min_signers);
    proposal.executed = true;
}
