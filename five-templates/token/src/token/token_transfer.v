// ============================================================================
// TOKEN TRANSFER
// ============================================================================

pub transfer(
    source: TokenAccount @mut,
    destination: TokenAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(source.owner == owner.key);
    require(source.mint == destination.mint);
    require(!source.is_frozen);
    require(!destination.is_frozen);
    require(source.balance >= amount);
    require(destination.balance <= 18446744073709551615 - amount);

    source.balance = source.balance - amount;
    destination.balance = destination.balance + amount;
}

pub approve(
    source: TokenAccount @mut,
    owner: account @signer,
    delegate: pubkey,
    amount: u64
) {
    require(source.owner == owner.key);
    source.delegate = delegate;
    source.delegated_amount = amount;
}

pub transfer_from(
    source: TokenAccount @mut,
    destination: TokenAccount @mut,
    delegate: account @signer,
    amount: u64
) {
    require(source.delegate == delegate.key);
    require(source.delegated_amount >= amount);
    require(source.mint == destination.mint);
    require(!source.is_frozen);
    require(!destination.is_frozen);
    require(source.balance >= amount);
    require(destination.balance <= 18446744073709551615 - amount);

    source.delegated_amount = source.delegated_amount - amount;
    source.balance = source.balance - amount;
    destination.balance = destination.balance + amount;
}
