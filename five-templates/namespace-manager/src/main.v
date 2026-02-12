// 5NS Namespace Manager (template)
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
    symbol: string;
    domain: string;
    owner: pubkey;
    registered_at: u64;
    updated_at: u64;
}

account SubprogramBinding {
    symbol: string;
    domain: string;
    subprogram: string;
    script_account: pubkey;
    owner: pubkey;
    version: u64;
    updated_at: u64;
}

account BindingHistory {
    symbol: string;
    domain: string;
    subprogram: string;
    version: u64;
    script_account: pubkey;
    updated_at: u64;
}

pub init_manager(
    cfg: NamespaceConfig @mut @init(payer=admin, seeds=["5ns_config"]),
    admin: account @signer,
    treasury: pubkey
) {
    cfg.admin = admin.key;
    cfg.treasury = treasury;
    cfg.at_price_lamports = 1_000_000_000;
    cfg.bang_price_lamports = 2_000_000_000;
    cfg.hash_price_lamports = 1_500_000_000;
    cfg.dollar_price_lamports = 1_250_000_000;
    cfg.percent_price_lamports = 1_250_000_000;
    cfg.version_nonce = 0;
}

pub set_symbol_price(
    cfg: NamespaceConfig @mut,
    admin: account @signer,
    symbol: string,
    lamports: u64
) {
    require(admin.key == cfg.admin);
    if (symbol == "@") { cfg.at_price_lamports = lamports; }
    if (symbol == "!") { cfg.bang_price_lamports = lamports; }
    if (symbol == "#") { cfg.hash_price_lamports = lamports; }
    if (symbol == "$") { cfg.dollar_price_lamports = lamports; }
    if (symbol == "%") { cfg.percent_price_lamports = lamports; }
}

pub register_tld(
    cfg: NamespaceConfig,
    tld: TldRecord @mut @init(payer=owner, seeds=["5ns_tld", symbol, domain]),
    owner: account @signer,
    symbol: string,
    domain: string,
    now: u64
) {
    require(tld.owner == 0 || tld.owner == owner.key);
    // Payment transfer to treasury is expected to be enforced by host integration.
    tld.symbol = symbol;
    tld.domain = domain;
    tld.owner = owner.key;
    tld.registered_at = now;
    tld.updated_at = now;
}

pub bind_subprogram(
    tld: TldRecord,
    binding: SubprogramBinding @mut @init(payer=owner, seeds=["5ns_binding", symbol, domain, subprogram]),
    owner: account @signer,
    symbol: string,
    domain: string,
    subprogram: string,
    script_account: pubkey,
    now: u64
) {
    require(tld.owner == owner.key);
    require(tld.symbol == symbol);
    require(tld.domain == domain);

    binding.symbol = symbol;
    binding.domain = domain;
    binding.subprogram = subprogram;
    binding.script_account = script_account;
    binding.owner = owner.key;
    binding.version = binding.version + 1;
    binding.updated_at = now;
}

pub update_subprogram(
    binding: SubprogramBinding @mut,
    history: BindingHistory @mut @init(payer=owner, seeds=["5ns_history", binding.symbol, binding.domain, binding.subprogram, binding.version]),
    owner: account @signer,
    next_script_account: pubkey,
    now: u64
) {
    require(binding.owner == owner.key);

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
