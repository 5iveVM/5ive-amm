// bonding_curve_token.v — Five DSL single-file
// Fully functional token mint with a built-in **linear bonding curve** priced in another (base) token.
// Supports: buy (mint vs base), sell (burn for base), quotes, fees to treasury, pause, and param updates.
// Single authority owns the curve vault and acts as mint_authority.

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
// Curve config (linear: p=a+b*s)
// ----------------------------
account CurveCfg {
  // wiring
  token_mint: pubkey;      // the token we mint/burn
  base_mint: pubkey;       // base token used to buy/sell (e.g., USDC)
  base_vault: pubkey;      // base TokenAccount owned by `authority`
  treasury_base: pubkey;   // base fees go here
  authority: pubkey;       // curve authority (vault owner & mint_authority)
  paused: bool;

  // params (units are in base token smallest units per 1 token)
  base_price: u64;         // a
  slope: u64;              // b (price increases by `slope` per token of supply)

  // fees (basis points)
  fee_bps_buy: u16;        // fee applied on base paid for buys
  fee_bps_sell: u16;       // fee applied on base out for sells
}

// ----------------------------
// Math helpers (u128 safe math)
// ----------------------------
fn cost_to_mint_linear(a: u64, b: u64, s: u64, n: u64) -> u128 {
  // Sum_{i=0..n-1} (a + b*(s+i)) = n*a + b*(n*s + n*(n-1)/2)
  let n128 = n as u128; let s128 = s as u128; let a128 = a as u128; let b128 = b as u128;
  let term1 = n128 * a128;
  let term2 = b128 * (n128 * s128 + (n128 * (n128 - 1u128)) / 2u128);
  return term1 + term2;
}

fn revenue_from_burn_linear(a: u64, b: u64, s: u64, n: u64) -> u128 {
  // Sum_{i=1..n} (a + b*(s-i)) = n*a + b*(n*s - n*(n+1)/2)
  let n128 = n as u128; let s128 = s as u128; let a128 = a as u128; let b128 = b as u128;
  let term1 = n128 * a128;
  let term2 = b128 * (n128 * s128 - (n128 * (n128 + 1u128)) / 2u128);
  return term1 + term2;
}

fn min_u128(a: u128, b: u128) -> u128 { if (a < b) { return a; } else { return b; } }

// --------------
// Quote helpers
// --------------
// Quotes return base amount (in base token units). No fees included unless noted.
pub quote_buy(curve: CurveCfg, token_mint: TokenMint, tokens_out: u64) -> u128 {
  require(token_mint.key == curve.token_mint);
  require(tokens_out > 0);
  return cost_to_mint_linear(curve.base_price, curve.slope, token_mint.supply, tokens_out);
}

pub quote_sell(curve: CurveCfg, token_mint: TokenMint, tokens_in: u64) -> u128 {
  require(token_mint.key == curve.token_mint);
  require(tokens_in > 0 && token_mint.supply >= tokens_in);
  return revenue_from_burn_linear(curve.base_price, curve.slope, token_mint.supply, tokens_in);
}

// --------------
// Buy / Sell
// --------------
// BUY EXACT TOKENS: user wants `tokens_out`, pays base <= max_base_in.
pub buy_exact_tokens(curve: CurveCfg @mut,
                     token_mint: TokenMint @mut,
                     base_vault: TokenAccount @mut,
                     treasury_base: TokenAccount @mut,
                     buyer_base_src: TokenAccount @mut,
                     buyer_token_dst: TokenAccount @mut,
                     authority: pubkey @signer,
                     buyer: pubkey @signer,
                     tokens_out: u64,
                     max_base_in: u64) {
  // wiring checks
  require(curve.token_mint == token_mint.key);
  require(curve.base_vault == base_vault.key);
  require(curve.treasury_base == treasury_base.key);
  require(curve.authority == authority);
  require(buyer_base_src.owner == buyer && buyer_token_dst.owner == buyer);
  require(buyer_base_src.mint == curve.base_mint);
  require(buyer_token_dst.mint == curve.token_mint);
  require(!curve.paused && tokens_out > 0);

  // quote cost and fees
  let cost_u128 = cost_to_mint_linear(curve.base_price, curve.slope, token_mint.supply, tokens_out);
  let mut cost: u64 = cost_u128 as u64; // assume fits (choose params accordingly)
  let fee = (cost * (curve.fee_bps_buy as u64)) / 10_000u64;
  let total = cost + fee;
  require(total <= max_base_in);

  // collect base: cost -> vault; fee -> treasury
  transfer(buyer_base_src, base_vault, buyer, cost);
  if (fee > 0) { transfer(buyer_base_src @mut, treasury_base @mut, buyer, fee); }

  // mint tokens to buyer (authority mints)
  mint_to(token_mint, buyer_token_dst, authority, tokens_out);
}

// SELL EXACT TOKENS: user provides `tokens_in`, receives base >= min_base_out.
pub sell_exact_tokens(curve: CurveCfg @mut,
                      token_mint: TokenMint @mut,
                      base_vault: TokenAccount @mut,
                      treasury_base: TokenAccount @mut,
                      seller_token_src: TokenAccount @mut,
                      seller_base_dst: TokenAccount @mut,
                      authority: pubkey @signer,
                      seller: pubkey @signer,
                      tokens_in: u64,
                      min_base_out: u64) {
  // wiring checks
  require(curve.token_mint == token_mint.key);
  require(curve.base_vault == base_vault.key);
  require(curve.treasury_base == treasury_base.key);
  require(curve.authority == authority);
  require(seller_token_src.owner == seller && seller_base_dst.owner == seller);
  require(seller_token_src.mint == curve.token_mint);
  require(seller_base_dst.mint == curve.base_mint);
  require(!curve.paused && tokens_in > 0 && token_mint.supply >= tokens_in);

  // compute revenue and fee
  let rev_u128 = revenue_from_burn_linear(curve.base_price, curve.slope, token_mint.supply, tokens_in);
  let mut rev: u64 = rev_u128 as u64; // assume fits
  let fee = (rev * (curve.fee_bps_sell as u64)) / 10_000u64;
  let pay_out = rev - fee;
  require(pay_out >= min_base_out);
  require(base_vault.amount >= (pay_out + fee));

  // burn seller tokens first
  burn(token_mint, seller_token_src, seller, tokens_in);

  // pay base to seller, fee to treasury (authority moves from vault)
  transfer(base_vault, seller_base_dst, authority, pay_out);
  if (fee > 0) { transfer(base_vault @mut, treasury_base @mut, authority, fee); }
}

// --------------
// Admin
// --------------
pub init_curve(curve: CurveCfg @mut,
               token_mint: TokenMint @mut,
               base_mint: TokenMint,
               base_vault: TokenAccount,
               treasury_base: TokenAccount,
               authority: pubkey @signer,
               decimals: u8,
               initial_supply_to_treasury: u64,
               base_price: u64,
               slope: u64,
               fee_bps_buy: u16,
               fee_bps_sell: u16) {
  // setup token mint
  token_mint.decimals = decimals;
  token_mint.mint_authority = authority;

  // wire curve
  require(base_vault.mint == base_mint.key);
  require(treasury_base.mint == base_mint.key);
  curve.token_mint = token_mint.key;
  curve.base_mint = base_mint.key;
  curve.base_vault = base_vault.key;
  curve.treasury_base = treasury_base.key;
  curve.authority = authority;
  curve.paused = false;
  curve.base_price = base_price;
  curve.slope = slope;
  curve.fee_bps_buy = fee_bps_buy;
  curve.fee_bps_sell = fee_bps_sell;

  // optional seed liquidity (mint to treasury destination)
  if (initial_supply_to_treasury > 0) {
    mint_to(token_mint, /*dest*/ TokenAccount{key: curve.token_mint} @mut, authority, initial_supply_to_treasury);
  }
}

pub set_paused(curve: CurveCfg @mut, authority: pubkey @signer, v: bool) { require(authority == curve.authority); curve.paused = v; }

pub set_params(curve: CurveCfg @mut, authority: pubkey @signer, base_price: u64, slope: u64, fee_bps_buy: u16, fee_bps_sell: u16) {
  require(authority == curve.authority);
  curve.base_price = base_price; curve.slope = slope;
  curve.fee_bps_buy = fee_bps_buy; curve.fee_bps_sell = fee_bps_sell;
}
