// ============================================================================
// ESCROW CORE
// ============================================================================

pub init_escrow(
    escrow: Escrow @mut @init,
    maker: account @signer,
    taker: pubkey,
    mint: pubkey,
    amount: u64,
    expires_at: u64
) -> pubkey {
    require(amount > 0);
    escrow.maker = maker.key;
    escrow.taker = taker;
    escrow.mint = mint;
    escrow.amount = amount;
    escrow.expires_at = expires_at;
    escrow.is_fulfilled = false;
    escrow.is_cancelled = false;
    return escrow.key;
}

pub fulfill_escrow(
    escrow: Escrow @mut,
    taker: account @signer
) {
    require(!escrow.is_fulfilled);
    require(!escrow.is_cancelled);
    require(escrow.taker == taker.key);
    require(get_clock() <= escrow.expires_at);
    escrow.is_fulfilled = true;
}

pub cancel_escrow(
    escrow: Escrow @mut,
    maker: account @signer
) {
    require(!escrow.is_fulfilled);
    require(!escrow.is_cancelled);
    require(escrow.maker == maker.key);
    escrow.is_cancelled = true;
}
