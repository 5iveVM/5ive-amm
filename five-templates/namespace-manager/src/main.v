// 5NS Namespace Manager
// This program is intended to be deployed with privileged permissions
// so special namespace symbols (! @ # $ %) can be managed without VM patching.

account NamespaceConfig {
    admin: pubkey;
    treasury: pubkey;
    at_price_lamports: u64;
    bang_price_lamports: u64;
    hash_price_lamports: u64;
    dollar_price_lamports: u64;
    percent_price_lamports: u64;
    version_nonce: u64;
}

account TldRecord {
    symbol: string<4>;
    domain: string<64>;
    owner: pubkey;
    registered_at: u64;
    updated_at: u64;
}

account SubprogramBinding {
    symbol: string<4>;
    domain: string<64>;
    subprogram: string<64>;
    script_account: pubkey;
    owner: pubkey;
    version: u64;
    updated_at: u64;
}

account BindingHistory {
    symbol: string<4>;
    domain: string<64>;
    subprogram: string<64>;
    version: u64;
    script_account: pubkey;
    updated_at: u64;
}

pub init_manager(
    cfg: NamespaceConfig @mut @init(payer=admin, seeds=["5ns_config"]),
    admin: account @signer,
    treasury: pubkey
) {
    cfg.admin = admin.ctx.key;
    cfg.treasury = treasury;
    cfg.at_price_lamports = 1_000_000_000;
    cfg.bang_price_lamports = 2_000_000_000;
    cfg.hash_price_lamports = 1_500_000_000;
    cfg.dollar_price_lamports = 10_000_000_000;
    cfg.percent_price_lamports = 1_250_000_000;
    cfg.version_nonce = 0;
}

pub set_symbol_price(
    cfg: NamespaceConfig @mut,
    admin: account @signer,
    symbol: string,
    price_lamports: u64
) {
    require(admin.ctx.key == cfg.admin);
    if (symbol == "@") { cfg.at_price_lamports = price_lamports; }
    if (symbol == "!") { cfg.bang_price_lamports = price_lamports; }
    if (symbol == "#") { cfg.hash_price_lamports = price_lamports; }
    if (symbol == "$") { cfg.dollar_price_lamports = price_lamports; }
    if (symbol == "%") { cfg.percent_price_lamports = price_lamports; }
}

pub get_symbol_price(cfg: NamespaceConfig, symbol: string) -> u64 {
    if (symbol == "@") { return cfg.at_price_lamports; }
    if (symbol == "!") { return cfg.bang_price_lamports; }
    if (symbol == "#") { return cfg.hash_price_lamports; }
    if (symbol == "$") { return cfg.dollar_price_lamports; }
    if (symbol == "%") { return cfg.percent_price_lamports; }
    return 0;
}

pub register_tld(
    cfg: NamespaceConfig @mut,
    tld: TldRecord @mut @init(payer=owner, seeds=["5ns_tld", symbol, domain]),
    owner: account @mut @signer,
    treasury_account: account @mut,
    symbol: string,
    domain: string,
    now: u64
) {
    require(treasury_account.ctx.key == cfg.treasury);
    let price_lamports = get_symbol_price(cfg, symbol);
    require(price_lamports > 0);
    transfer_lamports(owner, treasury_account, price_lamports);

    tld.symbol = symbol;
    tld.domain = domain;
    tld.owner = owner.ctx.key;
    tld.registered_at = now;
    tld.updated_at = now;
    cfg.version_nonce = cfg.version_nonce + 1;
}

pub bind_subprogram(
    tld: TldRecord,
    binding: SubprogramBinding @mut @init(payer=owner, seeds=["5ns_binding", symbol, domain, subprogram]),
    owner: account @mut @signer,
    symbol: string,
    domain: string,
    subprogram: string,
    script_account: pubkey,
    now: u64
) {
    require(tld.owner == owner.ctx.key);

    binding.symbol = symbol;
    binding.domain = domain;
    binding.subprogram = subprogram;
    binding.script_account = script_account;
    binding.owner = owner.ctx.key;
    binding.version = binding.version + 1;
    binding.updated_at = now;
}

pub update_subprogram(
    binding: SubprogramBinding @mut,
    history: BindingHistory @mut @init(payer=owner, seeds=["5ns_history", binding.symbol, binding.domain, binding.subprogram, binding.version]),
    owner: account @mut @signer,
    next_script_account: pubkey,
    now: u64
) {
    require(binding.owner == owner.ctx.key);

    history.symbol = binding.symbol;
    history.domain = binding.domain;
    history.subprogram = binding.subprogram;
    history.version = binding.version;
    history.script_account = binding.script_account;
    history.updated_at = now;

    binding.script_account = next_script_account;
    binding.version = binding.version + 1;
    binding.updated_at = now;
}

pub resolve(binding: SubprogramBinding) -> pubkey {
    return binding.script_account;
}
