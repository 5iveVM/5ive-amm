
pub fn launch_token(
    curve: BondingCurve @mut @init,
    mint: Mint @mut @init,
    curve_token_account: TokenAccount @mut @init,
    creator: account @signer,
    name: string,
    symbol: string,
    uri: string
) {
    core_launch_token(curve, mint, curve_token_account, creator, name, symbol, uri);
}

pub fn buy_token(
    curve: BondingCurve @mut,
    curve_token_account: TokenAccount @mut,
    buyer_token_account: TokenAccount @mut,
    buyer: account @signer,
    mint: Mint, 
    amount_sol_in: u64
) {
    core_buy_token(curve, curve_token_account, buyer_token_account, buyer, mint, amount_sol_in);
}

pub fn sell_token(
    curve: BondingCurve @mut,
    curve_token_account: TokenAccount @mut,
    seller_token_account: TokenAccount @mut,
    seller: account @signer,
    mint: Mint,
    amount_tokens_in: u64
) {
    core_sell_token(curve, curve_token_account, seller_token_account, seller, mint, amount_tokens_in);
}

// Helper needed for tests initiating accounts
pub fn init_token_account(
    token_account: TokenAccount @mut @init,
    owner: account @signer,
    mint: pubkey
) {
    init_token_account(token_account, owner, mint.key);
}
