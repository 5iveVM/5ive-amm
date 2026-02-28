// ============================================================================
// TOKEN MINT
// ============================================================================

pub mint_to(
    mint: Mint @mut,
    destination: TokenAccount @mut,
    authority: account @signer,
    amount: u64
) {
    require(mint.authority == authority.ctx.key);
    require(destination.mint == mint.ctx.key);
    require(!destination.is_frozen);
    require(mint.supply <= 18446744073709551615 - amount);
    require(destination.balance <= 18446744073709551615 - amount);

    mint.supply = mint.supply + amount;
    destination.balance = destination.balance + amount;
}

pub burn(
    mint: Mint @mut,
    source: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(source.owner == owner.ctx.key);
    require(source.mint == mint.ctx.key);
    require(source.balance >= amount);
    require(mint.supply >= amount);

    source.balance = source.balance - amount;
    mint.supply = mint.supply - amount;
}
