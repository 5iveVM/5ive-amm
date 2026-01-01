// memecoin_lean.v — Single-file Five DSL memecoin (ultra-lean)
// transfer() includes tax + pause gate. No burn, no delegate, no freeze, no lists, no locks.

account TokenMint {
  supply: u64;
  decimals: u8;
  mint_authority: pubkey;
}

account TokenAccount {
  mint: pubkey;
  owner: pubkey;
  amount: u64;
}

account MemeConfig {
  mint: pubkey;
  owner: pubkey;
  treasury: pubkey;
  paused: bool;
  tax_bps: u16; // 0..1000 (<=10%)
}

pub mint_to(mint: TokenMint @mut, dest: TokenAccount @mut, auth: pubkey @signer, amount: u64) {
  require(mint.mint_authority == auth);
  require(dest.mint == mint.key);
  require(amount > 0);
  mint.supply = mint.supply + amount;
  dest.amount = dest.amount + amount;
}

// SINGLE transfer with inline tax -> treasury
pub transfer(cfg: MemeConfig @mut,
             mint: TokenMint @mut,
             src: TokenAccount @mut,
             dst: TokenAccount @mut,
             treasury: TokenAccount @mut,
             owner: pubkey @signer,
             amount: u64) {
  require(cfg.mint == mint.key);
  require(treasury.key == cfg.treasury);
  require(src.owner == owner);
  require(src.mint == mint.key && dst.mint == mint.key && treasury.mint == mint.key);
  require(amount > 0 && src.amount >= amount);
  if (cfg.paused) { require(owner == cfg.owner); }

  let tax: u64 = (amount * (cfg.tax_bps as u64)) / 10_000u64;
  let send: u64 = amount - tax;

  src.amount = src.amount - amount;
  dst.amount = dst.amount + send;
  if (tax > 0) { treasury.amount = treasury.amount + tax; }
}

// --- Admin ---

pub init_memecoin(cfg: MemeConfig @mut,
                  mint: TokenMint @mut,
                  owner: pubkey @signer,
                  treasury_acc: TokenAccount,
                  decimals: u8,
                  initial_supply: u64,
                  tax_bps: u16) {
  require(treasury_acc.mint == mint.key);
  require(tax_bps <= 1_000);
  mint.decimals = decimals;
  mint.mint_authority = owner;

  cfg.mint = mint.key;
  cfg.owner = owner;
  cfg.treasury = treasury_acc.key;
  cfg.paused = true;      // start paused; open after seeding
  cfg.tax_bps = tax_bps;

  if (initial_supply > 0) { mint_to(mint, treasury_acc @mut, owner, initial_supply); }
}

pub set_paused(cfg: MemeConfig @mut, owner: pubkey @signer, v: bool) {
  require(owner == cfg.owner);
  cfg.paused = v;
}

pub set_tax(cfg: MemeConfig @mut, owner: pubkey @signer, v: u16) {
  require(owner == cfg.owner && v <= 1_000);
  cfg.tax_bps = v;
}

pub set_treasury(cfg: MemeConfig @mut, owner: pubkey @signer, t: pubkey) {
  require(owner == cfg.owner);
  cfg.treasury = t;
}

pub set_owner(cfg: MemeConfig @mut, owner: pubkey @signer, n: pubkey) {
  require(owner == cfg.owner);
  cfg.owner = n;
}
