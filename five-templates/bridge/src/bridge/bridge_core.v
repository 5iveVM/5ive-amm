// ============================================================================
// BRIDGE CORE
// ============================================================================

pub init_bridge(
    bridge: Bridge @mut @init,
    authority: account @signer,
    wrapped_mint: pubkey,
    name: string
) -> pubkey {
    bridge.authority = authority.key;
    bridge.wrapped_mint = wrapped_mint;
    bridge.total_supply = 0;
    bridge.is_paused = false;
    bridge.name = name;
    return bridge.key;
}

pub init_wrapped_account(
    account: WrappedAccount @mut @init,
    owner: account @signer,
    bridge: pubkey
) -> pubkey {
    account.owner = owner.key;
    account.bridge = bridge;
    account.balance = 0;
    return account.key;
}

pub wrap(
    bridge: Bridge @mut,
    account: WrappedAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(!bridge.is_paused);
    require(account.owner == owner.key);
    require(account.bridge == bridge.key);
    require(amount > 0);
    require(bridge.total_supply <= 18446744073709551615 - amount);
    require(account.balance <= 18446744073709551615 - amount);

    bridge.total_supply = bridge.total_supply + amount;
    account.balance = account.balance + amount;
}

pub unwrap(
    bridge: Bridge @mut,
    account: WrappedAccount @mut,
    owner: account @signer,
    amount: u64
) {
    require(!bridge.is_paused);
    require(account.owner == owner.key);
    require(account.bridge == bridge.key);
    require(account.balance >= amount);
    require(bridge.total_supply >= amount);

    bridge.total_supply = bridge.total_supply - amount;
    account.balance = account.balance - amount;
}

pub pause_bridge(bridge: Bridge @mut, authority: account @signer) {
    require(bridge.authority == authority.key);
    bridge.is_paused = true;
}

pub unpause_bridge(bridge: Bridge @mut, authority: account @signer) {
    require(bridge.authority == authority.key);
    bridge.is_paused = false;
}
