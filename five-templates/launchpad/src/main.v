import launchpad_types;
import launchpad_core;
import bonding_curve;
import token_mint;
import token_transfer;

pub fn launch_token(
    curve: launchpad_types::BondingCurve @mut @init,
    mint: launchpad_types::Mint @mut @init,
    curve_token_account: launchpad_types::TokenAccount @mut @init,
    creator: account @signer,
    name: string,
    symbol: string,
    uri: string
) {
    launchpad_core::core_launch_token(curve, mint, curve_token_account, creator, name, symbol, uri);
}

pub fn buy_token(
    curve: launchpad_types::BondingCurve @mut,
    curve_token_account: launchpad_types::TokenAccount @mut,
    buyer_token_account: launchpad_types::TokenAccount @mut,
    buyer: account @signer,
    mint: launchpad_types::Mint,
    amount_sol_in: u64
) {
    launchpad_core::core_buy_token(curve, curve_token_account, buyer_token_account, buyer, mint, amount_sol_in);
}

pub fn sell_token(
    curve: launchpad_types::BondingCurve @mut,
    curve_token_account: launchpad_types::TokenAccount @mut,
    seller_token_account: launchpad_types::TokenAccount @mut,
    seller: account @signer,
    mint: launchpad_types::Mint,
    amount_tokens_in: u64
) {
    launchpad_core::core_sell_token(curve, curve_token_account, seller_token_account, seller, mint, amount_tokens_in);
}

// Helper needed for tests initiating accounts
pub fn init_token_account(
    token_account: launchpad_types::TokenAccount @mut @init,
    owner: account @signer,
    mint: pubkey
) {
    token_transfer::init_token_account(token_account, owner, mint);
}
