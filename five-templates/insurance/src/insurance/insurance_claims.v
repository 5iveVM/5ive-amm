// ============================================================================
// INSURANCE CLAIMS
// ============================================================================

pub file_claim(
    policy: Policy,
    claim: Claim @mut @init,
    claimant: account @signer,
    amount: u64
) -> pubkey {
    require(policy.active);
    require(policy.holder == claimant.key);
    require(amount > 0);
    require(amount <= policy.coverage_amount);

    claim.policy = policy.key;
    claim.claimant = claimant.key;
    claim.amount = amount;
    claim.approved = false;
    return claim.key;
}

pub approve_claim(
    pool: InsurancePool,
    claim: Claim @mut,
    authority: account @signer
) {
    require(pool.authority == authority.key);
    require(!claim.approved);
    claim.approved = true;
}
