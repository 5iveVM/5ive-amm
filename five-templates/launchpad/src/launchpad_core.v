import launchpad_types;
import bonding_curve;
import token_mint;
import token_transfer;

// Launch a new token
pub fn core_launch_token(
    curve: launchpad_types::BondingCurve @mut @init,
    mint: launchpad_types::Mint @mut @init,
    curve_token_account: launchpad_types::TokenAccount @mut @init,
    creator: account @signer,
    name: string,
    symbol: string,
    uri: string
) {
    // 1. Init Mint
    token_mint::init_mint(mint, creator, 0, 6, name, symbol, uri); // No freeze authority

    // 2. Init Curve
    bonding_curve::init_curve(curve);

    // 3. Init Curve Token Account (Owner is Curve PDA)
    token_transfer::init_token_account_pda(curve_token_account, curve.key, mint.key);

    // 4. Mint Initial Supply to Curve
    let amount = curve.real_token_reserves;
    token_mint::mint_to(mint, curve_token_account, creator, amount);
}

// Buy Action
pub fn core_buy_token(
    curve: launchpad_types::BondingCurve @mut,
    curve_token_account: launchpad_types::TokenAccount @mut,
    buyer_token_account: launchpad_types::TokenAccount @mut,
    buyer: account @signer,
    mint: launchpad_types::Mint,
    amount_sol_in: u64
) {
    // 1. Calculate tokens out
    let tokens_out = bonding_curve::buy(curve, amount_sol_in, 0);

    // 2. Transfer Tokens: Curve -> Buyer
    require(curve_token_account.balance >= tokens_out);

    // "Virtual" transfer for PDA-owned account (simulating trusted transfer)
    curve_token_account.balance = curve_token_account.balance - tokens_out;
    buyer_token_account.balance = buyer_token_account.balance + tokens_out;
}

// Sell Action
pub fn core_sell_token(
    curve: launchpad_types::BondingCurve @mut,
    curve_token_account: launchpad_types::TokenAccount @mut,
    seller_token_account: launchpad_types::TokenAccount @mut,
    seller: account @signer,
    mint: launchpad_types::Mint,
    amount_tokens_in: u64
) {
    // 1. Calculate SOL out
    let sol_out = bonding_curve::sell(curve, amount_tokens_in, 0);

    // 2. Transfer tokens: Seller -> Curve
    token_transfer::transfer(seller_token_account, curve_token_account, seller, mint, amount_tokens_in);
}
