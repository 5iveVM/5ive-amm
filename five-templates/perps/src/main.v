import types::perps_types;
import perps::perps_core;
import perps::perps_liquidation;

use "11111111111111111111111111111111"::{transfer};

pub fn init_market(
    market: perps_types::PerpMarket @mut @init,
    authority: account @signer,
    base_mint: pubkey,
    quote_mint: pubkey,
    maintenance_margin_bps: u64,
    liquidation_fee_bps: u64,
    name: string
) -> pubkey {
    return perps_core::init_market(
        market,
        authority,
        base_mint,
        quote_mint,
        maintenance_margin_bps,
        liquidation_fee_bps,
        name
    );
}

pub fn init_position(
    position: perps_types::Position @mut @init,
    owner: account @signer,
    market: pubkey
) -> pubkey {
    return perps_core::init_position(position, owner, market);
}

pub fn open_position(
    market: perps_types::PerpMarket,
    position: perps_types::Position @mut,
    owner: account @signer,
    owner_collateral_token: account @mut,
    market_collateral_vault: account @mut,
    size: u64,
    entry_price: u64,
    collateral: u64,
    token_bytecode: account
) {
    transfer(owner_collateral_token, market_collateral_vault, owner, collateral);
    perps_core::open_position(market, position, owner, size, entry_price, collateral);
}

pub fn close_position(
    position: perps_types::Position @mut,
    owner: account @signer,
    market_authority: account @signer,
    market_collateral_vault: account @mut,
    owner_collateral_token: account @mut,
    token_bytecode: account
) {
    let collateral_out: u64 = position.collateral;

    perps_core::close_position(position, owner);
    transfer(market_collateral_vault, owner_collateral_token, market_authority, collateral_out);
}

pub fn add_collateral(
    position: perps_types::Position @mut,
    owner: account @signer,
    owner_collateral_token: account @mut,
    market_collateral_vault: account @mut,
    amount: u64,
    token_bytecode: account
) {
    let new_collateral: u64 = position.collateral + amount;
    transfer(owner_collateral_token, market_collateral_vault, owner, amount);
    perps_core::update_collateral(position, owner, new_collateral);
}

pub fn withdraw_collateral(
    position: perps_types::Position @mut,
    owner: account @signer,
    market_authority: account @signer,
    market_collateral_vault: account @mut,
    owner_collateral_token: account @mut,
    amount: u64,
    token_bytecode: account
) {
    require(position.collateral >= amount);

    let new_collateral: u64 = position.collateral - amount;
    perps_core::update_collateral(position, owner, new_collateral);
    transfer(market_collateral_vault, owner_collateral_token, market_authority, amount);
}

pub fn liquidate_position(
    market: perps_types::PerpMarket,
    position: perps_types::Position @mut,
    liquidator: account @signer,
    market_authority: account @signer,
    market_collateral_vault: account @mut,
    liquidator_collateral_token: account @mut,
    token_bytecode: account
) {
    let seized_collateral: u64 = position.collateral;

    perps_liquidation::liquidate_position(market, position, liquidator);
    transfer(
        market_collateral_vault,
        liquidator_collateral_token,
        market_authority,
        seized_collateral
    );
}
