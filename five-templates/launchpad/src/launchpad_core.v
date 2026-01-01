// Launch a new token
pub fn core_launch_token(
    curve: BondingCurve @mut @init,
    mint: Mint @mut @init,
    curve_token_account: TokenAccount @mut @init,
    creator: account @signer,
    name: string,
    symbol: string,
    uri: string
) {
    // 1. Init Mint
    init_mint(mint, creator, 0, 6, name, symbol, uri); // No freeze authority

    // 2. Init Curve
    init_curve(curve);

    // 3. Init Curve Token Account (Owner is Curve PDA)
    init_token_account_pda(curve_token_account, curve.key, mint.key);

    // 4. Mint Initial Supply to Curve
    let amount = curve.real_token_reserves;
    mint_to(mint, curve_token_account, creator, amount);
}

// Buy Action
pub fn core_buy_token(
    curve: BondingCurve @mut,
    curve_token_account: TokenAccount @mut,
    buyer_token_account: TokenAccount @mut,
    buyer: account @signer,
    mint: Mint, 
    amount_sol_in: u64
) {
    // 1. Calculate tokens out
    let tokens_out = buy(curve, amount_sol_in, 0);

    // 2. Transfer Tokens: Curve -> Buyer
    require(curve_token_account.balance >= tokens_out);

    // "Virtual" transfer for PDA-owned account (simulating trusted transfer)
    curve_token_account.balance = curve_token_account.balance - tokens_out;
    buyer_token_account.balance = buyer_token_account.balance + tokens_out;
}

// Sell Action
pub fn core_sell_token(
    curve: BondingCurve @mut,
    curve_token_account: TokenAccount @mut,
    seller_token_account: TokenAccount @mut,
    seller: account @signer,
    mint: Mint,
    amount_tokens_in: u64
) {
    // 1. Calculate SOL out
    let sol_out = sell(curve, amount_tokens_in, 0);

    // 2. Transfer tokens: Seller -> Curve
    transfer(seller_token_account, curve_token_account, seller, mint, amount_tokens_in);
}
