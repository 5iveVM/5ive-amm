// ============================================================================
// TREASURY TYPES
// ============================================================================

account Treasury {
    authority: pubkey;
    min_signers: u64;
    proposal_count: u64;
    is_paused: bool;
    name: string;
}

account SpendProposal {
    treasury: pubkey;
    proposer: pubkey;
    amount: u64;
    destination: pubkey;
    approvals: u64;
    executed: bool;
}

account Approval {
    proposal: pubkey;
    signer: pubkey;
}
