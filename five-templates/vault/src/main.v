use "11111111111111111111111111111111"::{transfer};

account VaultStrategy {
    authority: pubkey;
    asset_mint: pubkey;
    strategy_target: pubkey;
    total_assets: u64;
    total_shares: u64;
    performance_fee_bps: u64;
    is_paused: bool;
}

account VaultPosition {
    owner: pubkey;
    vault: pubkey;
    shares: u64;
    last_deposit_slot: u64;
}

pub fn init_vault(
    vault: VaultStrategy @mut @init,
    authority: account @signer,
    asset_mint: pubkey,
    strategy_target: pubkey,
    performance_fee_bps: u64
) {
    require(performance_fee_bps <= 2000);

    vault.authority = authority.key;
    vault.asset_mint = asset_mint;
    vault.strategy_target = strategy_target;
    vault.total_assets = 0;
    vault.total_shares = 0;
    vault.performance_fee_bps = performance_fee_bps;
    vault.is_paused = false;
}

pub fn init_position(
    position: VaultPosition @mut @init,
    vault: VaultStrategy,
    owner: account @signer
) {
    position.owner = owner.key;
    position.vault = vault.key;
    position.shares = 0;
    position.last_deposit_slot = 0;
}

pub fn deposit(
    vault: VaultStrategy @mut,
    position: VaultPosition @mut,
    owner: account @signer,
    owner_asset_token: account @mut,
    vault_asset_token: account @mut,
    amount: u64,
    min_shares_out: u64,
    token_bytecode: account
) -> u64 {
    require(!vault.is_paused);
    require(position.owner == owner.key);
    require(position.vault == vault.key);
    require(amount > 0);

    let mut shares_out: u64 = 0;
    if (vault.total_assets == 0 || vault.total_shares == 0) {
        shares_out = amount;
    } else {
        shares_out = (amount * vault.total_shares) / vault.total_assets;
    }

    require(shares_out >= min_shares_out);
    require(shares_out > 0);

    transfer(owner_asset_token, vault_asset_token, owner, amount);

    vault.total_assets = vault.total_assets + amount;
    vault.total_shares = vault.total_shares + shares_out;
    position.shares = position.shares + shares_out;
    position.last_deposit_slot = get_clock().slot;

    return shares_out;
}

pub fn withdraw(
    vault: VaultStrategy @mut,
    position: VaultPosition @mut,
    owner: account @signer,
    vault_authority: account @signer,
    vault_asset_token: account @mut,
    owner_asset_token: account @mut,
    shares_in: u64,
    min_assets_out: u64,
    token_bytecode: account
) -> u64 {
    require(position.owner == owner.key);
    require(position.vault == vault.key);
    require(shares_in > 0);
    require(position.shares >= shares_in);
    require(vault.total_shares >= shares_in);
    require(vault.total_assets > 0);

    let assets_out: u64 = (shares_in * vault.total_assets) / vault.total_shares;
    require(assets_out >= min_assets_out);
    require(vault.total_assets >= assets_out);

    position.shares = position.shares - shares_in;
    vault.total_shares = vault.total_shares - shares_in;
    vault.total_assets = vault.total_assets - assets_out;

    transfer(vault_asset_token, owner_asset_token, vault_authority, assets_out);

    return assets_out;
}

pub fn rebalance(
    vault: VaultStrategy @mut,
    authority: account @signer,
    new_strategy_target: pubkey,
    delta_bps: u64
) {
    require(vault.authority == authority.key);
    require(delta_bps <= 10000);

    vault.strategy_target = new_strategy_target;
}

pub fn harvest_yield(
    vault: VaultStrategy @mut,
    authority: account @signer,
    vault_authority: account @signer,
    vault_asset_token: account @mut,
    fee_receiver_token: account @mut,
    gross_yield: u64,
    token_bytecode: account
) {
    require(vault.authority == authority.key);
    require(gross_yield > 0);

    let fee_amount: u64 = (gross_yield * vault.performance_fee_bps) / 10000;
    let net_yield: u64 = gross_yield - fee_amount;

    vault.total_assets = vault.total_assets + net_yield;

    if (fee_amount > 0) {
        transfer(vault_asset_token, fee_receiver_token, vault_authority, fee_amount);
    }
}
