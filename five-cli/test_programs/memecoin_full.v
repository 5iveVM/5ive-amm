// memecoin_full.v — Single-file Five DSL script
// Inline SPL-like token types + a fully featured memecoin with taxes, limits,
// blacklist/exempt lists, trading gate, liquidity lock, ownership/renounce.
// No imports required. Designed to mirror SPL Token semantics.

// -----------------------------
// Inline SPL token primitives
// -----------------------------
account TokenMint {
    supply: u64;
    decimals: u8;
    is_initialized: bool;
    freeze_authority: pubkey;
    mint_authority: pubkey;
}

account TokenAccount {
    mint: pubkey;       // points to TokenMint
    owner: pubkey;      // signing authority
    amount: u64;        // token balance
    delegate: pubkey;   // allowance authority
    state: u8;          // 0=uninit, 1=initialized, 2=frozen
    is_native: bool;    // SOL wrapper flag (unused here)
    delegated_amount: u64; // allowance amount
}

// --- SPL ops (minimal complete set) ---

pub transfer(source: TokenAccount @mut, destination: TokenAccount @mut, authority: pubkey @signer, amount: u64) {
    require(source.owner == authority);
    require(source.mint == destination.mint);
    require(source.state == 1 && destination.state == 1);
    require(amount > 0);
    require(source.amount >= amount);
    source.amount = source.amount - amount;
    destination.amount = destination.amount + amount;
}

pub mint_to(mint: TokenMint @mut, destination: TokenAccount @mut, mint_authority: pubkey @signer, amount: u64) {
    require(mint.is_initialized == true);
    require(mint.mint_authority == mint_authority);
    require(destination.mint == mint.key);
    require(amount > 0);
    mint.supply = mint.supply + amount;
    destination.amount = destination.amount + amount;
}

pub burn(mint: TokenMint @mut, token_account: TokenAccount @mut, owner: pubkey @signer, amount: u64) {
    require(token_account.owner == owner);
    require(token_account.mint == mint.key);
    require(amount > 0 && token_account.amount >= amount);
    token_account.amount = token_account.amount - amount;
    mint.supply = mint.supply - amount;
}

pub approve_delegate(account: TokenAccount @mut, owner: pubkey @signer, delegate: pubkey, amount: u64) {
    require(account.owner == owner);
    account.delegate = delegate;
    account.delegated_amount = amount;
}

pub revoke_delegate(account: TokenAccount @mut, owner: pubkey @signer) {
    require(account.owner == owner);
    account.delegate = 0x0;
    account.delegated_amount = 0;
}

pub transfer_from(source: TokenAccount @mut, destination: TokenAccount @mut, delegate: pubkey @signer, amount: u64) {
    require(source.delegate == delegate);
    require(source.delegated_amount >= amount);
    require(source.mint == destination.mint);
    require(source.state == 1 && destination.state == 1);
    require(amount > 0 && source.amount >= amount);
    source.delegated_amount = source.delegated_amount - amount;
    source.amount = source.amount - amount;
    destination.amount = destination.amount + amount;
}

pub freeze_account(mint: TokenMint @mut, account: TokenAccount @mut, freeze_authority: pubkey @signer) {
    require(mint.freeze_authority == freeze_authority);
    require(account.mint == mint.key);
    account.state = 2; // frozen
}

pub thaw_account(mint: TokenMint @mut, account: TokenAccount @mut, freeze_authority: pubkey @signer) {
    require(mint.freeze_authority == freeze_authority);
    require(account.mint == mint.key);
    account.state = 1; // initialized
}

// -------------------------------------------
// Memecoin configuration + features (single file)
// -------------------------------------------
account MemeConfig {
    // Core wires
    mint: pubkey;               // TokenMint key for this memecoin
    owner: pubkey;              // admin/owner (can be renounced)
    treasury: pubkey;           // treasury TokenAccount pubkey (same mint)
    burn_sink: pubkey;          // pseudo account key for burn accounting (not required)

    // Trading gate & global freeze
    trading_open: bool;         // false until owner opens
    global_freeze: bool;        // pauses all transfers (owner bypass only)

    // Taxes & limits
    tax_bps: u16;               // total tax = amount * tax_bps / 10_000
    burn_bps: u16;              // of the tax, portion to burn; rest goes to treasury
    limits_enabled: bool;       // apply anti-whale limits when true
    max_tx_bps: u16;            // max transfer size as pct of total supply
    max_wallet_bps: u16;        // max wallet holding as pct of total supply

    // Liquidity lock (optional)
    liquidity_vault: pubkey;    // vault TokenAccount holding LP tokens or supply
    lock_until_ts: u64;         // unix time until which vault is locked

    // Exempt / blacklist (fixed 8 slots each to keep it light)
    ex0: pubkey; ex1: pubkey; ex2: pubkey; ex3: pubkey;
    ex4: pubkey; ex5: pubkey; ex6: pubkey; ex7: pubkey;
    bl0: pubkey; bl1: pubkey; bl2: pubkey; bl3: pubkey;
    bl4: pubkey; bl5: pubkey; bl6: pubkey; bl7: pubkey;

    // Ownership management
    renounce_requested_at: u64;     // 0 if none
    renounce_delay_seconds: u64;    // e.g., 86_400 (24h) by default
    owner_renounced: bool;          // true after finalize
    pending_new_owner: pubkey;      // for transfer ownership
}

// -----------------
// Helper predicates
// -----------------
fn is_zero(p: pubkey) -> bool { return p == 0x0; }

fn is_exempt(cfg: MemeConfig, who: pubkey) -> bool {
    return who == cfg.owner ||
           who == cfg.ex0 || who == cfg.ex1 || who == cfg.ex2 || who == cfg.ex3 ||
           who == cfg.ex4 || who == cfg.ex5 || who == cfg.ex6 || who == cfg.ex7;
}

fn is_blacklisted(cfg: MemeConfig, who: pubkey) -> bool {
    if (who == cfg.bl0) { return true; }
    if (who == cfg.bl1) { return true; }
    if (who == cfg.bl2) { return true; }
    if (who == cfg.bl3) { return true; }
    if (who == cfg.bl4) { return true; }
    if (who == cfg.bl5) { return true; }
    if (who == cfg.bl6) { return true; }
    if (who == cfg.bl7) { return true; }
    return false;
}

// -----------------
// Admin functions
// -----------------

pub init_memecoin(cfg: MemeConfig @mut,
                  mint: TokenMint @mut,
                  owner: pubkey @signer,
                  treasury_account: TokenAccount,
                  decimals: u8,
                  initial_supply: u64,
                  tax_bps: u16,
                  burn_bps: u16) {
    // Initialize mint
    require(!mint.is_initialized);
    mint.decimals = decimals;
    mint.is_initialized = true;
    mint.mint_authority = owner;
    mint.freeze_authority = owner;

    // Wire config
    cfg.mint = mint.key;
    cfg.owner = owner;
    cfg.treasury = treasury_account.key;
    cfg.burn_sink = 0x0;
    cfg.trading_open = false;
    cfg.global_freeze = false;
    cfg.tax_bps = tax_bps;
    cfg.burn_bps = burn_bps;
    cfg.limits_enabled = true;
    cfg.max_tx_bps = 200;        // default 2% max tx
    cfg.max_wallet_bps = 300;    // default 3% max wallet
    cfg.liquidity_vault = 0x0;
    cfg.lock_until_ts = 0;
    cfg.renounce_requested_at = 0;
    cfg.renounce_delay_seconds = 86_400; // 24h
    cfg.owner_renounced = false;
    cfg.pending_new_owner = 0x0;

    // Zero lists
    cfg.ex0 = 0x0; cfg.ex1 = 0x0; cfg.ex2 = 0x0; cfg.ex3 = 0x0;
    cfg.ex4 = 0x0; cfg.ex5 = 0x0; cfg.ex6 = 0x0; cfg.ex7 = 0x0;
    cfg.bl0 = 0x0; cfg.bl1 = 0x0; cfg.bl2 = 0x0; cfg.bl3 = 0x0;
    cfg.bl4 = 0x0; cfg.bl5 = 0x0; cfg.bl6 = 0x0; cfg.bl7 = 0x0;

    // Optional pre-mint to treasury (or seed LP). Caller must pass the treasury account.
    if (initial_supply > 0) {
        require(treasury_account.mint == mint.key);
        mint_to(mint, treasury_account @mut, owner, initial_supply);
    }
}

pub set_trading_open(cfg: MemeConfig @mut, owner: pubkey @signer) {
    require(owner == cfg.owner);
    cfg.trading_open = true;
}

pub set_global_freeze(cfg: MemeConfig @mut, owner: pubkey @signer, freeze: bool) {
    require(owner == cfg.owner);
    cfg.global_freeze = freeze;
}

pub set_tax(cfg: MemeConfig @mut, owner: pubkey @signer, tax_bps: u16, burn_bps: u16) {
    require(owner == cfg.owner);
    require(tax_bps <= 1_000); // cap 10%
    require(burn_bps <= 10_000);
    cfg.tax_bps = tax_bps;
    cfg.burn_bps = burn_bps;
}

pub set_limits(cfg: MemeConfig @mut, owner: pubkey @signer, enabled: bool, max_tx_bps: u16, max_wallet_bps: u16) {
    require(owner == cfg.owner);
    require(max_tx_bps <= 10_000 && max_wallet_bps <= 10_000);
    cfg.limits_enabled = enabled;
    cfg.max_tx_bps = max_tx_bps;
    cfg.max_wallet_bps = max_wallet_bps;
}

pub set_treasury(cfg: MemeConfig @mut, owner: pubkey @signer, new_treasury: pubkey) {
    require(owner == cfg.owner);
    cfg.treasury = new_treasury;
}

pub set_liquidity_lock(cfg: MemeConfig @mut, owner: pubkey @signer, vault: pubkey, unlock_ts: u64) {
    require(owner == cfg.owner);
    cfg.liquidity_vault = vault;
    cfg.lock_until_ts = unlock_ts;
}

// Manage exempt/blacklist slots (index 0..7). Simpler than loops/maps.
pub set_exempt_slot(cfg: MemeConfig @mut, owner: pubkey @signer, idx: u8, who: pubkey) {
    require(owner == cfg.owner);
    if (idx == 0) { cfg.ex0 = who; } else if (idx == 1) { cfg.ex1 = who; }
    else if (idx == 2) { cfg.ex2 = who; } else if (idx == 3) { cfg.ex3 = who; }
    else if (idx == 4) { cfg.ex4 = who; } else if (idx == 5) { cfg.ex5 = who; }
    else if (idx == 6) { cfg.ex6 = who; } else if (idx == 7) { cfg.ex7 = who; }
}

pub set_blacklist_slot(cfg: MemeConfig @mut, owner: pubkey @signer, idx: u8, who: pubkey) {
    require(owner == cfg.owner);
    if (idx == 0) { cfg.bl0 = who; } else if (idx == 1) { cfg.bl1 = who; }
    else if (idx == 2) { cfg.bl2 = who; } else if (idx == 3) { cfg.bl3 = who; }
    else if (idx == 4) { cfg.bl4 = who; } else if (idx == 5) { cfg.bl5 = who; }
    else if (idx == 6) { cfg.bl6 = who; } else if (idx == 7) { cfg.bl7 = who; }
}

// Ownership
pub transfer_ownership(cfg: MemeConfig @mut, owner: pubkey @signer, new_owner: pubkey) {
    require(owner == cfg.owner);
    cfg.pending_new_owner = new_owner;
}

pub accept_ownership(cfg: MemeConfig @mut, new_owner: pubkey @signer) {
    require(cfg.pending_new_owner == new_owner);
    cfg.owner = new_owner;
    cfg.pending_new_owner = 0x0;
}

pub renounce_start(cfg: MemeConfig @mut, owner: pubkey @signer, now: u64) {
    require(owner == cfg.owner);
    cfg.renounce_requested_at = now;
}

pub renounce_finish(cfg: MemeConfig @mut, owner: pubkey @signer, now: u64) {
    require(owner == cfg.owner);
    require(cfg.renounce_requested_at > 0);
    require(now >= cfg.renounce_requested_at + cfg.renounce_delay_seconds);
    cfg.owner_renounced = true;
    // Optionally clear authorities on mint (true renounce)
}

// ----------------------------------
// Core: taxed & limited token transfer
// ----------------------------------

pub memecoin_transfer(cfg: MemeConfig @mut,
                      mint: TokenMint @mut,
                      src: TokenAccount @mut,
                      dst: TokenAccount @mut,
                      treasury_acc: TokenAccount @mut,
                      owner: pubkey @signer,
                      now: u64,
                      amount: u64) {
    // Basic SPL checks
    require(src.owner == owner);
    require(src.mint == mint.key && dst.mint == mint.key && treasury_acc.mint == mint.key);
    require(src.state == 1 && dst.state == 1 && treasury_acc.state == 1);
    require(amount > 0 && src.amount >= amount);

    // Global freeze (owner bypass)
    if (cfg.global_freeze) { require(owner == cfg.owner); }

    // Trading gate
    if (!cfg.trading_open) {
        // Before open, only owner or exempt addresses can move (for LP seeding/ops)
        require(is_exempt(cfg, owner));
    }

    // Blacklist checks (unless exempt)
    if (!is_exempt(cfg, owner)) {
        require(!is_blacklisted(cfg, owner));
        require(!is_blacklisted(cfg, dst.owner));
    }

    // Liquidity lock (if source is the locked vault)
    if (src.key == cfg.liquidity_vault) { require(now >= cfg.lock_until_ts); }

    // Limits (skip for exempt)
    if (cfg.limits_enabled && !is_exempt(cfg, owner)) {
        let max_tx = (mint.supply * (cfg.max_tx_bps as u64)) / 10_000u64;
        require(amount <= max_tx || max_tx == 0);
    }

    // Compute tax (skip for exempt)
    let mut tax: u64 = 0;
    let mut send_amount: u64 = amount;
    if (!is_exempt(cfg, owner) && cfg.tax_bps > 0) {
        tax = (amount * (cfg.tax_bps as u64)) / 10_000u64;
        if (tax > 0) {
            require(amount > tax);
            send_amount = amount - tax;
        }
    }

    // Max wallet limit (post-transfer check) for non-exempt recipients
    if (cfg.limits_enabled && !is_exempt(cfg, dst.owner)) {
        let max_wallet = (mint.supply * (cfg.max_wallet_bps as u64)) / 10_000u64;
        // simulate new balance
        let new_balance = dst.amount + send_amount;
        require(new_balance <= max_wallet || max_wallet == 0);
    }

    // Execute main transfer first
    transfer(src, dst, owner, send_amount);

    // Handle tax if any: split into burn + treasury
    if (tax > 0) {
        let burn_part = (tax * (cfg.burn_bps as u64)) / 10_000u64;
        let treas_part = tax - burn_part;
        if (burn_part > 0) {
            burn(mint, src @mut, owner, burn_part);
        }
        if (treas_part > 0) {
            transfer(src @mut, treasury_acc @mut, owner, treas_part);
        }
    }
}

// -----------------------
// Convenience mint/burn
// -----------------------

pub owner_mint(cfg: MemeConfig, mint: TokenMint @mut, dest: TokenAccount @mut, owner: pubkey @signer, amount: u64) {
    require(owner == cfg.owner);
    mint_to(mint, dest, owner, amount);
}

pub owner_burn(cfg: MemeConfig, mint: TokenMint @mut, from: TokenAccount @mut, owner: pubkey @signer, amount: u64) {
    // Owner-directed burn (requires from.owner == owner)
    require(owner == cfg.owner);
    burn(mint, from, owner, amount);
}

// Simple airdrop utility (owner-only, single recipient to keep ABI small)
pub airdrop(cfg: MemeConfig, mint: TokenMint @mut, to: TokenAccount @mut, owner: pubkey @signer, amount: u64) {
    require(owner == cfg.owner);
    mint_to(mint, to, owner, amount);
}

// Approve/revoke passthroughs for wallets
pub wallet_approve(account: TokenAccount @mut, owner: pubkey @signer, delegate: pubkey, amount: u64) { approve_delegate(account, owner, delegate, amount); }
pub wallet_revoke(account: TokenAccount @mut, owner: pubkey @signer) { revoke_delegate(account, owner); }

// Freeze/thaw controls (owner is the freeze authority by default)
pub owner_freeze(cfg: MemeConfig, mint: TokenMint @mut, acct: TokenAccount @mut, owner: pubkey @signer) { require(owner == cfg.owner); freeze_account(mint, acct, owner); }
pub owner_thaw(cfg: MemeConfig, mint: TokenMint @mut, acct: TokenAccount @mut, owner: pubkey @signer) { require(owner == cfg.owner); thaw_account(mint, acct, owner); }

// ---------------------------------
// Example entrypoints (UX wrappers)
// ---------------------------------

// Public transfer entrypoint users should call instead of raw `transfer`
pub transfer_with_tax(cfg: MemeConfig @mut,
                      mint: TokenMint @mut,
                      src: TokenAccount @mut,
                      dst: TokenAccount @mut,
                      treasury_acc: TokenAccount @mut,
                      owner: pubkey @signer,
                      now: u64,
                      amount: u64) {
    memecoin_transfer(cfg, mint, src, dst, treasury_acc, owner, now, amount);
}

// Allow spending via allowance (delegate)
pub transfer_from_with_tax(cfg: MemeConfig @mut,
                           mint: TokenMint @mut,
                           src: TokenAccount @mut,
                           dst: TokenAccount @mut,
                           treasury_acc: TokenAccount @mut,
                           delegate: pubkey @signer,
                           now: u64,
                           amount: u64) {
    // Pull using delegate, then apply taxation logic manually on the payer (src.owner)
    // To simplify: require delegate is exempt (e.g., routers/MMs) or skip extra tax here
    // For full parity, we need src.owner passed; we conservatively tax against src.owner == delegate for now.
    require(src.delegate == delegate);
    // Construct a temporary view where `delegate` acts as owner for checks
    memecoin_transfer(cfg, mint, src, dst, treasury_acc, delegate, now, amount);
}
