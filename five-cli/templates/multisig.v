// Multisig wallet template (simplified)

account MultisigState {
    threshold: u8;
    approvals: u64;
    last_proposal_id: u64;
    proposal_hash: u64; // placeholder hash/instruction id
    executed: bool;
}

// Initialize multisig settings
pub init_multisig(state: MultisigState @mut, threshold: u8) {
    state.threshold = threshold;
    state.approvals = 0;
    state.last_proposal_id = 0;
    state.proposal_hash = 0;
    state.executed = false;
}

// Open a new proposal (reset counters)
pub open_proposal(state: MultisigState @mut, proposal_hash: u64) {
    state.last_proposal_id = state.last_proposal_id + 1;
    state.proposal_hash = proposal_hash;
    state.approvals = 0;
    state.executed = false;
}

// Approve current proposal (no per-signer dedup for simplicity)
pub approve(state: MultisigState @mut) {
    state.approvals = state.approvals + 1;
}

// Execute if threshold met (no side-effects beyond flag)
pub execute(state: MultisigState @mut) {
    require(!state.executed);
    require(state.approvals >= state.threshold);
    state.executed = true;
}
