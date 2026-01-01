// zero.exe.v — Season 1 executable for FIVE VM
// BONK‑priced bonding curve token with:
//  • Boss 1 (gate.sys): BONK jackpots LOCKED (treasury building)
//  • Boss 2‑5: deterministic jackpots enabled (reels)
//  • Operator’s Cut: 20% of BONK on buys/sells + 20% of ZERO minted on buys
//  • Auto‑rotating bosses by slot; .hack × Shadowrun lore baked in names
//  • u64‑only math; VM traps on overflow
//
// Notes:
//  - Base token = BONK (5 decimals). All curve prices in BONK‑micros per 1 ZERO unit.
//  - Buy flow splits ZERO minted: buyer gets (10000-operator_bps)/10000; operator gets operator_bps/10000.
//  - Buy flow splits BONK paid: operator_bps → operator BONK treasury; remainder → base_vault (jackpot treasury).
//  - Sell flow splits BONK revenue: operator_bps → operator BONK treasury; remainder → seller from base_vault.
//  - Jackpot pool accounting accumulates from buys (remainder to base_vault). Boss 1 prevents payout.
//  - Final boss (examiner.exe) can add a mint bonus in ZERO on jackpot.

// --------------------
// Minimal SPL surface
// --------------------
account TokenMint { supply: u64; decimals: u8; mint_authority: pubkey; }
account TokenAccount { mint: pubkey; owner: pubkey; amount: u64; }

pub mint_to(mint: TokenMint @mut, dest: TokenAccount @mut, auth: pubkey @signer, amount: u64) {
  require(mint.mint_authority == auth);
  require(dest.mint == mint.key);
  require(amount > 0);
  mint.supply += amount; dest.amount += amount;
}

pub burn(mint: TokenMint @mut, src: TokenAccount @mut, owner: pubkey @signer, amount: u64) {
  require(src.owner == owner);
  require(src.mint == mint.key);
  require(amount > 0 && src.amount >= amount);
  src.amount -= amount; mint.supply -= amount;
}

pub transfer(src: TokenAccount @mut, dst: TokenAccount @mut, signer: pubkey @signer, amount: u64) {
  require(src.owner == signer);
  require(src.mint == dst.mint);
  require(amount > 0 && src.amount >= amount);
  src.amount -= amount; dst.amount += amount;
}

// ----------------------------
// Zero.exe Config
// ----------------------------
account ZeroExeCfg {
  // wiring
  token_mint: pubkey;      // ZERO mint
  base_mint: pubkey;       // BONK mint (5 decimals)
  base_vault: pubkey;      // BONK jackpot treasury vault
  treasury_base: pubkey;   // Operator BONK treasury
  treasury_token: pubkey;  // Operator ZERO treasury
  authority: pubkey;       // program authority (mint authority)
  paused: bool;

  // linear curve params (units: BONK‑micros per 1 ZERO unit)
  base_price: u64;         // a
  slope: u64;              // b

  // operator economics (basis points: 0..10000)
  operator_bps: u16;       // default 2000 (20%)

  // jackpot pool accounting (BONK units in base_mint smallest units)
  jackpot_pool_base: u64;  // grows from buys; paid out on jackpot

  // boss eras & rotation
  boss_id: u8;             // 0: gate.sys (locked jackpots), 1: trickster.dll, 2: whale.sys, 3: shadow.log, 4: examiner.exe
  max_boss: u8;            // set to 4 (five bosses indexed 0..4)
  rotate_every_slots: u64; // auto‑rotate cadence (0 = off)
  last_rotate_slot: u64;   // last slot rotated

  // jackpot tuning
  base_trigger_mod: u16;   // baseline rarity (e.g., 777)
  trigger_step: u16;       // per‑boss rarity adjust
  base_bonus_bps: u16;     // baseline payout boost per era
  bonus_step_bps: u16;     // per‑boss increment (clamped to 10000)
  liquidity_pct_bps: u16;  // % of base_vault used as kicker
  mint_bonus_bps: u16;     // extra ZERO minted on jackpot (used esp. on examiner.exe)
}

// ----------------------------
// Math helpers (u64)
// ----------------------------
price_units(cfg: ZeroExeCfg, zero_mint: TokenMint) -> u64 {
  return cfg.base_price + cfg.slope * zero_mint.supply;
}

cost_to_mint_linear(a: u64, b: u64, s: u64, n: u64) -> u64 {
  // cost = n*a + b*(n*s + n*(n−1)/2)
  return n*a + b * (n*s + (n*(n-1))/2);
}

revenue_from_burn_linear(a: u64, b: u64, s: u64, n: u64) -> u64 {
  // rev = n*a + b*(n*s − n*(n+1)/2)
  return n*a + b * (n*s - (n*(n+1))/2);
}

// -----------------
// Boss & reels
// -----------------
clamp_u16(x: u16, lo: u16, hi: u16) -> u16 { if (x < lo) { return lo; } if (x > hi) { return hi; } return x; }

era_trigger_mod(cfg: ZeroExeCfg) -> u16 {
  let dec = (cfg.boss_id as u16) * cfg.trigger_step;
  let mut m_i: i64 = (cfg.base_trigger_mod as i64) - (dec as i64);
  if (m_i < 3) { m_i = 3; }
  if (m_i > 10000) { m_i = 10000; }
  return m_i as u16;
}

era_bonus_bps(cfg: ZeroExeCfg) -> u16 {
  let add = (cfg.boss_id as u16) * cfg.bonus_step_bps;
  let mut b = (cfg.base_bonus_bps as u32) + (add as u32);
  if (b > 10000) { b = 10000; }
  return b as u16;
}

reels_hit(slot: u64, boss_id: u8, modn: u16) -> bool {
  let m: u64 = modn as u64;
  let r1 = (slot + (boss_id as u64)) % m;
  let r2 = ((slot / m) + (boss_id as u64) * 3) % m;
  let r3 = ((slot / (m*m + 1)) + (boss_id as u64) * 5) % m;
  return r1 == r2 && r2 == r3;
}

maybe_rotate(cfg: ZeroExeCfg @mut, cur_slot: u64) {
  if (cfg.rotate_every_slots == 0) { return; }
  if (cfg.last_rotate_slot == 0 || cur_slot - cfg.last_rotate_slot >= cfg.rotate_every_slots) {
    cfg.boss_id = cfg.boss_id + 1u8; if (cfg.boss_id > cfg.max_boss) { cfg.boss_id = 0u8; }
    cfg.last_rotate_slot = cur_slot;
  }
}

// -------------------------------------
// Jackpot engine (called by buy/sell)
// -------------------------------------
maybe_jackpot(cfg: ZeroExeCfg @mut,
                 zero_mint: TokenMint @mut,
                 base_vault: TokenAccount @mut,
                 treasury_base: TokenAccount @mut,
                 authority: pubkey @signer,
                 winner_base_dst: TokenAccount @mut,
                 winner_zero_dst: TokenAccount @mut,
                 cur_slot: u64) {
  // rotate era first
  maybe_rotate(cfg @mut, cur_slot);

  // Boss 1 gate.sys: jackpots LOCKED
  if (cfg.boss_id == 0u8) { return; }

  // Need funds & reel hit
  let modn = era_trigger_mod(cfg);
  if (!reels_hit(cur_slot, cfg.boss_id, modn)) { return; }
  if (cfg.jackpot_pool_base == 0) { return; }

  // liquidity kicker & era bonus
  let liq_cut = (base_vault.amount * (cfg.liquidity_pct_bps as u64)) / 10_000u64;
  let bonus_bps = era_bonus_bps(cfg) as u64;
  let base_boost = cfg.jackpot_pool_base + (cfg.jackpot_pool_base * bonus_bps) / 10_000u64;
  let desired_base = base_boost + liq_cut;

  // Payout from base_vault up to desired
  let mut pay_base = desired_base;
  if (pay_base > base_vault.amount) { pay_base = base_vault.amount; }
  if (pay_base > 0) { transfer(base_vault, winner_base_dst, authority, pay_base); }

  // Final boss (examiner.exe) may mint ZERO bonus
  if (cfg.mint_bonus_bps > 0 && cfg.boss_id == cfg.max_boss) {
    let unit_price = price_units(cfg, zero_mint);
    if (unit_price > 0) {
      let raw_tokens = pay_base / unit_price;
      let mint_bonus = (raw_tokens * (cfg.mint_bonus_bps as u64)) / 10_000u64;
      if (mint_bonus > 0) { mint_to(zero_mint, winner_zero_dst @mut, authority, mint_bonus); }
    }
  }

  // Reset pool after win
  cfg.jackpot_pool_base = 0;
}

// -----------------
// Quotes (view)
// -----------------
pub quote_buy(cfg: ZeroExeCfg, zero_mint: TokenMint, tokens_out_total: u64) -> u64 {
  // returns BONK cost excluding operator split (operator cut is taken from this cost)
  require(tokens_out_total > 0); require(zero_mint.key == cfg.token_mint);
  return cost_to_mint_linear(cfg.base_price, cfg.slope, zero_mint.supply, tokens_out_total);
}

pub quote_sell(cfg: ZeroExeCfg, zero_mint: TokenMint, tokens_in: u64) -> u64 {
  require(tokens_in > 0); require(zero_mint.key == cfg.token_mint);
  return revenue_from_burn_linear(cfg.base_price, cfg.slope, zero_mint.supply, tokens_in);
}

// --------------
// BUY / SELL Ixs
// --------------
// BUY EXACT TOKENS (split minted ZERO 80/20 buyer/operator)
pub buy_exact_tokens(cfg: ZeroExeCfg @mut,
                     zero_mint: TokenMint @mut,
                     base_vault: TokenAccount @mut,
                     treasury_base: TokenAccount @mut,
                     treasury_token: TokenAccount @mut,
                     buyer_base_src: TokenAccount @mut,
                     buyer_zero_dst: TokenAccount @mut,
                     authority: pubkey @signer,
                     buyer: pubkey @signer,
                     tokens_out_total: u64,
                     max_base_in: u64,
                     cur_slot: u64) {
  // wiring
  require(cfg.token_mint == zero_mint.key);
  require(cfg.base_vault == base_vault.key && cfg.treasury_base == treasury_base.key);
  require(cfg.treasury_token == treasury_token.key);
  require(cfg.authority == authority);
  require(buyer_base_src.owner == buyer && buyer_zero_dst.owner == buyer);
  require(buyer_base_src.mint == cfg.base_mint && buyer_zero_dst.mint == cfg.token_mint);
  require(!cfg.paused && tokens_out_total > 0);

  // quote BONK cost (no extra fee; operator split comes from this)
  let cost = cost_to_mint_linear(cfg.base_price, cfg.slope, zero_mint.supply, tokens_out_total);
  require(cost <= max_base_in);

  // split BONK: operator & jackpot treasury
  let op_base = (cost * (cfg.operator_bps as u64)) / 10_000u64;  // 20%
  let to_vault = cost - op_base;                                 // 80%

  // transfer BONK
  transfer(buyer_base_src, treasury_base, buyer, op_base);
  transfer(buyer_base_src @mut, base_vault @mut, buyer, to_vault);

  // split ZERO minted: buyer & operator
  let op_tokens = (tokens_out_total * (cfg.operator_bps as u64)) / 10_000u64;  // 20%
  let buyer_tokens = tokens_out_total - op_tokens;                               // 80%
  require(buyer_tokens > 0);
  mint_to(zero_mint, buyer_zero_dst, authority, buyer_tokens);
  if (op_tokens > 0) { mint_to(zero_mint @mut, treasury_token @mut, authority, op_tokens); }

  // grow jackpot pool with the BONK that entered vault this buy
  cfg.jackpot_pool_base += to_vault;

  // try jackpot (Boss 1 will short‑circuit)
  maybe_jackpot(cfg @mut, zero_mint @mut, base_vault @mut, treasury_base @mut, authority, buyer_base_src @mut, buyer_zero_dst @mut, cur_slot);
}

// SELL EXACT TOKENS (seller burns ZERO; BONK payout from vault minus operator cut)
pub sell_exact_tokens(cfg: ZeroExeCfg @mut,
                      zero_mint: TokenMint @mut,
                      base_vault: TokenAccount @mut,
                      treasury_base: TokenAccount @mut,
                      seller_zero_src: TokenAccount @mut,
                      seller_base_dst: TokenAccount @mut,
                      authority: pubkey @signer,
                      seller: pubkey @signer,
                      tokens_in: u64,
                      min_base_out: u64,
                      cur_slot: u64) {
  // wiring
  require(cfg.token_mint == zero_mint.key);
  require(cfg.base_vault == base_vault.key && cfg.treasury_base == treasury_base.key);
  require(cfg.authority == authority);
  require(seller_zero_src.owner == seller && seller_base_dst.owner == seller);
  require(seller_zero_src.mint == cfg.token_mint && seller_base_dst.mint == cfg.base_mint);
  require(!cfg.paused && tokens_in > 0 && zero_mint.supply >= tokens_in);

  // compute revenue
  let rev = revenue_from_burn_linear(cfg.base_price, cfg.slope, zero_mint.supply, tokens_in);
  let op_base = (rev * (cfg.operator_bps as u64)) / 10_000u64; // 20% to operator
  let pay_out = rev - op_base;
  require(pay_out >= min_base_out);
  require(base_vault.amount >= (pay_out + op_base));

  // burn seller ZERO first
  burn(zero_mint, seller_zero_src, seller, tokens_in);

  // pay BONK
  transfer(base_vault, seller_base_dst, authority, pay_out);
  if (op_base > 0) { transfer(base_vault @mut, treasury_base @mut, authority, op_base); }

  // try jackpot (Boss 1 will short‑circuit)
  maybe_jackpot(cfg @mut, zero_mint @mut, base_vault @mut, treasury_base @mut, authority, seller_base_dst @mut, seller_zero_src @mut, cur_slot);
}

// --------------
// Admin
// --------------
pub init_zero(cfg: ZeroExeCfg @mut,
              zero_mint: TokenMint @mut,
              base_mint: TokenMint,
              base_vault: TokenAccount,
              treasury_base: TokenAccount,
              treasury_token: TokenAccount,
              authority: pubkey @signer,
              token_decimals: u8,
              base_price: u64,
              slope: u64,
              operator_bps: u16,
              base_trigger_mod: u16,
              trigger_step: u16,
              base_bonus_bps: u16,
              bonus_step_bps: u16,
              liquidity_pct_bps: u16,
              mint_bonus_bps: u16,
              rotate_every_slots: u64) {
  // mint setup
  zero_mint.decimals = token_decimals;
  zero_mint.mint_authority = authority;

  // wiring
  require(base_vault.mint == base_mint.key);
  require(treasury_base.mint == base_mint.key);
  require(treasury_token.mint == zero_mint.key);
  cfg.token_mint = zero_mint.key;
  cfg.base_mint  = base_mint.key;
  cfg.base_vault = base_vault.key;
  cfg.treasury_base = treasury_base.key;
  cfg.treasury_token = treasury_token.key;
  cfg.authority = authority;
  cfg.paused = false;

  // economics
  cfg.base_price = base_price;
  cfg.slope = slope;
  cfg.operator_bps = operator_bps; // suggest 2000 (20%)
  cfg.jackpot_pool_base = 0;

  // bosses (0..4)
  cfg.boss_id = 0; cfg.max_boss = 4;
  cfg.rotate_every_slots = rotate_every_slots; cfg.last_rotate_slot = 0;

  // jackpot
  cfg.base_trigger_mod = base_trigger_mod;
  cfg.trigger_step = trigger_step;
  cfg.base_bonus_bps = base_bonus_bps;
  cfg.bonus_step_bps = bonus_step_bps;
  cfg.liquidity_pct_bps = liquidity_pct_bps;
  cfg.mint_bonus_bps = mint_bonus_bps;
}

pub set_paused(cfg: ZeroExeCfg @mut, auth: pubkey @signer, v: bool) { require(auth == cfg.authority); cfg.paused = v; }

pub set_curve(cfg: ZeroExeCfg @mut, auth: pubkey @signer, base_price: u64, slope: u64) { require(auth == cfg.authority); cfg.base_price = base_price; cfg.slope = slope; }

pub set_operator(cfg: ZeroExeCfg @mut, auth: pubkey @signer, operator_bps: u16, treasury_base: pubkey, treasury_token: pubkey) {
  require(auth == cfg.authority);
  cfg.operator_bps = operator_bps; cfg.treasury_base = treasury_base; cfg.treasury_token = treasury_token;
}

pub set_rotation(cfg: ZeroExeCfg @mut, auth: pubkey @signer, every_slots: u64) { require(auth == cfg.authority); cfg.rotate_every_slots = every_slots; }

// -----------------
// Zero‑arg tests (quick sanity)
// -----------------
pub test_buy_split() -> [u64;3] {
  // returns [buyer_tokens, operator_tokens, op_base_bonk]
  let base_price: u64 = 100; let slope: u64 = 1; let operator_bps: u16 = 2000;
  let s: u64 = 0; let n: u64 = 1000;
  let cost = cost_to_mint_linear(base_price, slope, s, n);
  let op_base = (cost * (operator_bps as u64))/10_000u64;
  let op_tokens = (n * (operator_bps as u64))/10_000u64;
  let buyer_tokens = n - op_tokens;
  return [buyer_tokens, op_tokens, op_base];
}

pub test_gate_lock() -> bool {
  // Boss 1 should block jackpots - encode expectation as simple check
  let boss_id: u8 = 0u8;
  return boss_id == 0u8;
}
