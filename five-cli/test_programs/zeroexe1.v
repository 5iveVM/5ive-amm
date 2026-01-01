// zero.exe.v — Season 1 (complete)
// Features:
//  • Linear BONK bonding curve (u64 safe) → ZERO
//  • 20% operator cut: BONK on buys/sells + ZERO on buys
//  • Boss 0..4 with auto-rotation and deterministic jackpots
//    - Boss 0 (gate.sys): jackpots MINT ZERO ONLY (BONK stays locked)
//    - Boss 1 (trickster.dll): wobble odds
//    - Boss 2 (whale.sys): rarer, big-boost flavor
//    - Boss 3 (shadow.log): doubling pressure until hit
//    - Boss 4 (examiner.exe): rare + optional ZERO mint bonus
//  • No transfer tax
//  • Optional spin_overload (Z1<->Z2) with toggle to make spins trigger jackpots (OFF by default)
//  • On-chain counters: total_attacks/buys, total_defenses/sells, total_spins
//
// Conventions:
//  - BONK & ZERO both use 5 decimals.
//  - Curve params (base_price, slope) are specified as whole-BONK-per-whole-ZERO units.
//  - 1 whole ZERO = 100_000 ZERO units; supply accounted in units.

account TokenMint { supply: u64; decimals: u8; mint_authority: pubkey; }
account TokenAccount { mint: pubkey; owner: pubkey; amount: u64; }

pub mint_to(mint: TokenMint @mut, dest: TokenAccount @mut, auth: pubkey @signer, amount: u64) {
  require(mint.mint_authority == auth);
  require(dest.mint == mint.key);
  require(amount > 0u64);
  mint.supply += amount; dest.amount += amount;
}

pub burn(mint: TokenMint @mut, src: TokenAccount @mut, owner: pubkey @signer, amount: u64) {
  require(src.owner == owner);
  require(src.mint == mint.key);
  require(amount > 0u64 && src.amount >= amount);
  src.amount -= amount; mint.supply -= amount;
}

pub transfer(src: TokenAccount @mut, dst: TokenAccount @mut, signer: pubkey @signer, amount: u64) {
  require(src.owner == signer);
  require(src.mint == dst.mint);
  require(amount > 0u64 && src.amount >= amount);
  src.amount -= amount; dst.amount += amount;
}

// ---------------------------------
// Zero.exe config & accounting
// ---------------------------------
account ZeroExeCfg {
  // wiring
  token_mint: pubkey;      // ZERO mint
  base_mint: pubkey;       // BONK mint
  base_vault: pubkey;      // BONK jackpot vault
  treasury_base: pubkey;   // operator BONK treasury
  treasury_token: pubkey;  // operator ZERO treasury
  authority: pubkey;
  paused: bool;

  // curve (whole BONK per whole ZERO @ s=0)
  base_price: u64;         // a
  slope: u64;              // b (per whole ZERO)

  // operator cut (bps)
  operator_bps: u16;       // 2000 = 20%

  // jackpot pool in BONK units (accumulates from buys)
  jackpot_pool_base: u64;

  // bosses
  boss_id: u8;             // 0..max_boss
  max_boss: u8;            // default 4
  rotate_every_slots: u64; // 0 = off
  last_rotate_slot: u64;

  // jackpot tuning
  base_trigger_mod: u16;   // rarity modulus (lower = more frequent)
  trigger_step: u16;       // wobble / era delta
  base_bonus_bps: u16;     // baseline payout boost
  bonus_step_bps: u16;     // per-boss increment (kept simple)
  liquidity_pct_bps: u16;  // % of base_vault used as kicker
  mint_bonus_bps: u16;     // extra ZERO on hit (boss 4)

  // shadow.log state
  shadow_pow: u8;          // doubling exponent (0..8)

  // spins toggle (off for launch)
  spins_trigger_jp: bool;  // if true, spin_overload will call jackpot

  // simple telemetry
  total_attacks: u64;
  total_defenses: u64;
  total_spins: u64;
}

// ----------------------------
// Math helpers (u64 only)
// ----------------------------
fn price_units(cfg: ZeroExeCfg, zm: TokenMint) -> u64 {
  // price per ZERO unit (scaled by 1/100_000) is derived from whole-zero price at current supply
  // We use whole ZERO supply for slope steps
  let s_whole = zm.supply / 100_000u64;
  return cfg.base_price + cfg.slope * s_whole;
}

fn cost_to_mint_linear(a: u64, b: u64, s_units: u64, n_units: u64) -> u64 {
  let s = s_units / 100_000u64;            // whole ZERO supply
  let n = n_units / 100_000u64;            // whole ZERO to mint
  let rem = n_units % 100_000u64;          // tail units
  let mut cost = n*a + b*(n*s + (n*(n-1))/2);
  let tail_price = a + b*(s + n);
  cost += (tail_price * rem)/100_000u64;
  return cost;
}

fn revenue_from_burn_linear(a: u64, b: u64, s_units: u64, n_units: u64) -> u64 {
  let s = s_units / 100_000u64;
  let n = n_units / 100_000u64;
  let rem = n_units % 100_000u64;
  let mut rev = n*a + b*(n*s - (n*(n+1))/2);
  let tail_price = a + b*(s - n);
  rev += (tail_price * rem)/100_000u64;
  return rev;
}

fn era_trigger_mod(cfg: ZeroExeCfg, slot: u64) -> u16 {
  // m = baseline rarity, adjusted by boss identity
  let mut m = cfg.base_trigger_mod as u32;
  if (cfg.boss_id == 1u8) { // trickster.dll → wobble
    let wobble = (slot % 13u64) as u32;
    if (wobble % 2u32 == 0u32) { m += (cfg.trigger_step as u32); } else { if (m > (cfg.trigger_step as u32)) { m -= (cfg.trigger_step as u32); } }
  } else if (cfg.boss_id == 2u8) { // whale.sys → make rarer
    m = m + (2u32 * (cfg.trigger_step as u32)) + 300u32;
  } else if (cfg.boss_id == 3u8) { // shadow.log → base rarity; payout scales
    // unchanged
  } else if (cfg.boss_id == 4u8) { // examiner.exe → slightly rarer
    m = m + (cfg.trigger_step as u32) + 200u32;
  }
  if (m < 3u32) { m = 3u32; } if (m > 10_000u32) { m = 10_000u32; }
  return m as u16;
}

fn era_bonus_bps(cfg: ZeroExeCfg) -> u16 {
  let mut b = cfg.base_bonus_bps as u32;
  if (cfg.boss_id == 2u8) { b += 800u32; }  // whale.sys
  if (cfg.boss_id == 4u8) { b += 300u32; }  // examiner.exe
  if (b > 10_000u32) { b = 10_000u32; }
  return b as u16;
}

fn reels_hit(slot: u64, boss_id: u8, modn: u16) -> bool {
  let m: u64 = modn as u64;
  let r1 = (slot + (boss_id as u64)) % m;
  let r2 = ((slot / m) + (boss_id as u64) * 3u64) % m;
  let r3 = ((slot / (m*m + 1u64)) + (boss_id as u64) * 5u64) % m;
  return r1 == r2 && r2 == r3;
}

fn maybe_rotate(cfg: ZeroExeCfg @mut, cur_slot: u64) {
  if (cfg.rotate_every_slots == 0u64) { return; }
  if (cfg.last_rotate_slot == 0u64 || cur_slot - cfg.last_rotate_slot >= cfg.rotate_every_slots) {
    let prev = cfg.boss_id;
    cfg.boss_id = cfg.boss_id + 1u8; if (cfg.boss_id > cfg.max_boss) { cfg.boss_id = 0u8; }
    cfg.last_rotate_slot = cur_slot;
    if (cfg.boss_id != prev) { cfg.shadow_pow = 0u8; }
  }
}

// -------------------------------------
// Jackpot engine
// -------------------------------------
fn maybe_jackpot(cfg: ZeroExeCfg @mut,
                 zero_mint: TokenMint @mut,
                 base_vault: TokenAccount @mut,
                 treasury_base: TokenAccount @mut,
                 authority: pubkey @signer,
                 winner_base_dst: TokenAccount @mut,
                 winner_zero_dst: TokenAccount @mut,
                 cur_slot: u64) {
  maybe_rotate(cfg @mut, cur_slot);

  let modn = era_trigger_mod(cfg, cur_slot);
  let hit = reels_hit(cur_slot, cfg.boss_id, modn);

  // Shadow pressure grows on miss
  if (cfg.boss_id == 3u8 && !hit) {
    if (cfg.shadow_pow < 8u8) { cfg.shadow_pow = cfg.shadow_pow + 1u8; }
    return;
  }
  if (!hit) { return; }

  // Nothing to do if pool empty (except Boss 0 ZERO-only can still mint from "energy" 0 → no-op)
  let pool = cfg.jackpot_pool_base;

  // Boss 0: pay ZERO only (no BONK leaves vault). Consume the pool "energy".
  if (cfg.boss_id == 0u8) {
    let unit_price = price_units(cfg, zero_mint);
    if (unit_price == 0u64) { return; }
    // Build "desired_base" similar to BONK payout bosses (liquidity kicker + era bonus),
    // then convert that value into ZERO at current unit price, mint to winner.
    let liq_cut = (base_vault.amount * (cfg.liquidity_pct_bps as u64)) / 10_000u64;
    let bonus_bps = era_bonus_bps(cfg) as u64;

    // Shadow multiplier irrelevant here, boss 0 has none
    let base_boost = pool + (pool * bonus_bps)/10_000u64;
    let desired_base = base_boost + liq_cut;

    let mut mint_tokens = desired_base / unit_price;
    if (mint_tokens == 0u64 && desired_base > 0u64) { mint_tokens = 1u64; } // ensure at least some ZERO if energy existed
    if (mint_tokens > 0u64) { mint_to(zero_mint @mut, winner_zero_dst @mut, authority, mint_tokens); }
    // Reset pool "energy"
    cfg.jackpot_pool_base = 0u64;
    return;
  }

  // Bosses 1..4: BONK payout (with shadow/examiner rules)
  if (pool == 0u64) { return; }

  let mut base_pool = pool;

  // Shadow effective multiplier
  if (cfg.boss_id == 3u8) {
    let mut i: u8 = 0u8; let mut mul: u64 = 1u64;
    while (i < cfg.shadow_pow) { mul = mul * 2u64; i = i + 1u8; }
    base_pool = base_pool * mul;
  }

  let liq_cut = (base_vault.amount * (cfg.liquidity_pct_bps as u64)) / 10_000u64;
  let bonus_bps = era_bonus_bps(cfg) as u64;
  let base_boost = base_pool + (base_pool * bonus_bps) / 10_000u64;
  let desired_base = base_boost + liq_cut;

  // Pay BONK from vault
  let mut pay_base = desired_base;
  if (pay_base > base_vault.amount) { pay_base = base_vault.amount; }
  if (pay_base > 0u64) { transfer(base_vault @mut, winner_base_dst @mut, authority, pay_base); }

  // Examiner: optional extra ZERO mint
  if (cfg.boss_id == 4u8 && cfg.mint_bonus_bps > 0u16) {
    let unit_price = price_units(cfg, zero_mint);
    if (unit_price > 0u64) {
      let raw_tokens = pay_base / unit_price;
      let mint_bonus = (raw_tokens * (cfg.mint_bonus_bps as u64))/10_000u64;
      if (mint_bonus > 0u64) { mint_to(zero_mint @mut, winner_zero_dst @mut, authority, mint_bonus); }
    }
  }

  // Reset pool & pressure after win
  cfg.jackpot_pool_base = 0u64;
  cfg.shadow_pow = 0u8;
}

// -----------------
// Read-only quotes
// -----------------
quote_buy(cfg: ZeroExeCfg, zero_mint: TokenMint, tokens_out_total: u64) -> u64 {
  require(tokens_out_total > 0u64);
  return cost_to_mint_linear(cfg.base_price, cfg.slope, zero_mint.supply, tokens_out_total);
}

quote_sell(cfg: ZeroExeCfg, zero_mint: TokenMint, tokens_in: u64) -> u64 {
  require(tokens_in > 0u64);
  return revenue_from_burn_linear(cfg.base_price, cfg.slope, zero_mint.supply, tokens_in);
}

// --------------
// BUY / SELL
// --------------
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
  require(!cfg.paused);
  require(cfg.token_mint == zero_mint.key);
  require(cfg.base_vault == base_vault.key && cfg.treasury_base == treasury_base.key);
  require(cfg.treasury_token == treasury_token.key);
  require(cfg.authority == authority);
  require(buyer_base_src.owner == buyer && buyer_zero_dst.owner == buyer);
  require(buyer_base_src.mint == cfg.base_mint && buyer_zero_dst.mint == cfg.token_mint);
  require(tokens_out_total > 0u64);

  let cost = cost_to_mint_linear(cfg.base_price, cfg.slope, zero_mint.supply, tokens_out_total);
  require(cost <= max_base_in);

  // BONK split
  let op_base = (cost * (cfg.operator_bps as u64))/10_000u64;
  let to_vault = cost - op_base;

  transfer(buyer_base_src @mut, treasury_base @mut, buyer, op_base);
  transfer(buyer_base_src @mut, base_vault @mut,   buyer, to_vault);

  // ZERO split
  let op_tokens = (tokens_out_total * (cfg.operator_bps as u64))/10_000u64;
  let buyer_tokens = tokens_out_total - op_tokens;
  require(buyer_tokens > 0u64);

  mint_to(zero_mint @mut, buyer_zero_dst @mut, authority, buyer_tokens);
  if (op_tokens > 0u64) { mint_to(zero_mint @mut, treasury_token @mut, authority, op_tokens); }

  // pool grows from this buy
  cfg.jackpot_pool_base += to_vault;

  // telemetry
  cfg.total_attacks += 1u64;

  // jackpot attempt (boss 0 → ZERO only, others → BONK)
  maybe_jackpot(cfg @mut, zero_mint @mut, base_vault @mut, treasury_base @mut, authority,
                buyer_base_src @mut, buyer_zero_dst @mut, cur_slot);
}

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
  require(!cfg.paused);
  require(cfg.token_mint == zero_mint.key);
  require(cfg.base_vault == base_vault.key && cfg.treasury_base == treasury_base.key);
  require(cfg.authority == authority);
  require(seller_zero_src.owner == seller && seller_base_dst.owner == seller);
  require(seller_zero_src.mint == cfg.token_mint && seller_base_dst.mint == cfg.base_mint);
  require(tokens_in > 0u64 && zero_mint.supply >= tokens_in);

  let rev = revenue_from_burn_linear(cfg.base_price, cfg.slope, zero_mint.supply, tokens_in);
  let op_base = (rev * (cfg.operator_bps as u64))/10_000u64;
  let pay_out = rev - op_base;
  require(pay_out >= min_base_out);
  require(base_vault.amount >= (pay_out + op_base));

  // burn ZERO first
  burn(zero_mint @mut, seller_zero_src @mut, seller, tokens_in);

  // pay BONK (operator + seller)
  transfer(base_vault @mut, treasury_base @mut, authority, op_base);
  transfer(base_vault @mut, seller_base_dst @mut,   authority, pay_out);

  // telemetry
  cfg.total_defenses += 1u64;

  // jackpot attempt
  maybe_jackpot(cfg @mut, zero_mint @mut, base_vault @mut, treasury_base @mut, authority,
                seller_base_dst @mut, seller_zero_src @mut, cur_slot);
}

// --------------------------
// Optional: SPIN / OVERLOAD
// --------------------------
// A "spin" is just moving ZERO between two ZERO accounts owned by the same wallet.
// By default (spins_trigger_jp = false), this does NOT call jackpot logic.
// Turn it on later via set_spins_trigger(true) if you want spins to trigger jackpots.
pub spin_overload(cfg: ZeroExeCfg @mut,
                   zero_mint: TokenMint @mut,        // <-- needs @mut
                   base_vault: TokenAccount @mut,
                   treasury_base: TokenAccount @mut,
                   authority: pubkey @signer,
                   player_wallet: pubkey @signer,
                   player_base: TokenAccount @mut,   // BONK dst if spins trigger jackpots
                   z1: TokenAccount @mut,
                   z2: TokenAccount @mut,
                   amount: u64,
                   dir: u8,                           // 0: z1->z2, 1: z2->z1
                   cur_slot: u64) {
  require(!cfg.paused);
  require(z1.mint == zero_mint.key && z2.mint == zero_mint.key);
  require(z1.owner == player_wallet && z2.owner == player_wallet);
  require(player_base.mint == cfg.base_mint && player_base.owner == player_wallet);
  require(amount > 0u64);

  if (dir == 0u8) {
    require(z1.amount >= amount);
    transfer(z1 @mut, z2 @mut, player_wallet, amount);
  } else {
    require(z2.amount >= amount);
    transfer(z2 @mut, z1 @mut, player_wallet, amount);
  }

  cfg.total_spins += 1u64;

  if (cfg.spins_trigger_jp) {
    if (dir == 0u8) {
      // winner_zero_dst is z2
      maybe_jackpot(cfg @mut, zero_mint @mut, base_vault @mut, treasury_base @mut,
                    authority, player_base @mut, z2 @mut, cur_slot);
    } else {
      // winner_zero_dst is z1
      maybe_jackpot(cfg @mut, zero_mint @mut, base_vault @mut, treasury_base @mut,
                    authority, player_base @mut, z1 @mut, cur_slot);
    }
  }
}



// --------------
// Admin setters
// --------------

// Apply a price preset directly to cfg (avoid consts/tuples)
fn apply_preset(cfg: ZeroExeCfg @mut, preset: u8) {
  // 0 = 1k, 1 = 10k, 2 = 20k
  if (preset == 0u8) {
    cfg.base_price = 1_000u64;
    cfg.slope = 5u64;
  } else if (preset == 1u8) {
    cfg.base_price = 10_000u64;
    cfg.slope = 10u64;
  } else {
    cfg.base_price = 20_000u64;
    cfg.slope = 10u64;
  }
}

pub init_zero_preset(
  cfg: ZeroExeCfg @mut,
  zero_mint: TokenMint @mut,
  base_mint: TokenMint,
  base_vault: TokenAccount,
  treasury_base: TokenAccount,
  treasury_token: TokenAccount,
  authority: pubkey @signer,
  token_decimals: u8,        // recommend 5
  price_preset: u8,          // 0=1k, 1=10k, 2=20k
  rotate_every_slots: u64    // e.g., ~650k for ~3 days
) {
  zero_mint.decimals = token_decimals;
  zero_mint.mint_authority = authority;

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

  // curve via preset (no consts/tuples)
  apply_preset(cfg @mut, price_preset);

  cfg.operator_bps = 2000u16;
  cfg.jackpot_pool_base = 0u64;

  cfg.boss_id = 0u8; cfg.max_boss = 4u8;
  cfg.rotate_every_slots = rotate_every_slots; cfg.last_rotate_slot = 0u64;

  // jackpot defaults tuned for multiple/day cadence
  cfg.base_trigger_mod = 200u16;
  cfg.trigger_step = 20u16;
  cfg.base_bonus_bps = 0u16;
  cfg.bonus_step_bps = 0u16;
  cfg.liquidity_pct_bps = 0u16;
  cfg.mint_bonus_bps = 0u16;

  cfg.shadow_pow = 0u8;
  cfg.spins_trigger_jp = false;

  cfg.total_attacks = 0u64;
  cfg.total_defenses = 0u64;
  cfg.total_spins = 0u64;
}

pub set_paused(cfg: ZeroExeCfg @mut, auth: pubkey @signer, v: bool) { require(auth == cfg.authority); cfg.paused = v; }
pub set_curve(cfg: ZeroExeCfg @mut, auth: pubkey @signer, base_price: u64, slope: u64) { require(auth == cfg.authority); cfg.base_price = base_price; cfg.slope = slope; }
pub set_operator(cfg: ZeroExeCfg @mut, auth: pubkey @signer, operator_bps: u16, treasury_base: pubkey, treasury_token: pubkey) {
  require(auth == cfg.authority);
  cfg.operator_bps = operator_bps; cfg.treasury_base = treasury_base; cfg.treasury_token = treasury_token;
}
pub set_rotation(cfg: ZeroExeCfg @mut, auth: pubkey @signer, every_slots: u64) { require(auth == cfg.authority); cfg.rotate_every_slots = every_slots; }
pub set_max_boss(cfg: ZeroExeCfg @mut, auth: pubkey @signer, new_max: u8) { require(auth == cfg.authority); require(new_max <= 10u8); cfg.max_boss = new_max; if (cfg.boss_id > cfg.max_boss) { cfg.boss_id = cfg.max_boss; } }
pub set_mint_bonus(cfg: ZeroExeCfg @mut, auth: pubkey @signer, mint_bonus_bps: u16) { require(auth == cfg.authority); cfg.mint_bonus_bps = mint_bonus_bps; }
pub set_spins_trigger(cfg: ZeroExeCfg @mut, auth: pubkey @signer, v: bool) { require(auth == cfg.authority); cfg.spins_trigger_jp = v; }
pub set_jackpot_tuning(cfg: ZeroExeCfg @mut, auth: pubkey @signer, base_trigger_mod: u16, trigger_step: u16) {
  require(auth == cfg.authority);
  require(base_trigger_mod >= 3u16 && base_trigger_mod <= 5000u16);
  require(trigger_step <= 1000u16);
  cfg.base_trigger_mod = base_trigger_mod; cfg.trigger_step = trigger_step;
}
