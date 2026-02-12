import types::lending_types;
import lending::lending_core;
import lending::lending_deposit;
import lending::lending_borrow;
import lending::lending_liquidation;

use "11111111111111111111111111111111"::{transfer};

pub fn init_lending_market(
    market: lending_types::LendingMarket @mut @init,
    authority: account @signer,
    quote_mint: pubkey,
    name: string
) -> pubkey {
    return lending_core::init_lending_market(market, authority, quote_mint, name);
}

pub fn init_reserve(
    reserve: lending_types::Reserve @mut @init,
    authority: account @signer,
    market: lending_types::LendingMarket,
    liquidity_mint: pubkey,
    collateral_mint: pubkey,
    borrow_rate_bps: u64,
    collateral_factor_bps: u64,
    liquidation_threshold_bps: u64,
    name: string
) -> pubkey {
    return lending_core::init_reserve(
        reserve,
        authority,
        market,
        liquidity_mint,
        collateral_mint,
        borrow_rate_bps,
        collateral_factor_bps,
        liquidation_threshold_bps,
        name
    );
}

pub fn init_obligation(
    obligation: lending_types::Obligation @mut @init,
    owner: account @signer,
    reserve: pubkey
) -> pubkey {
    return lending_deposit::init_obligation(obligation, owner, reserve);
}

pub fn deposit_collateral(
    reserve: lending_types::Reserve @mut,
    obligation: lending_types::Obligation @mut,
    owner: account @signer,
    owner_collateral_token: account @mut,
    reserve_collateral_token: account @mut,
    amount: u64,
    token_bytecode: account
) {
    transfer(owner_collateral_token, reserve_collateral_token, owner, amount);
    lending_deposit::deposit_collateral(reserve, obligation, owner, amount);
}

pub fn borrow(
    reserve: lending_types::Reserve @mut,
    obligation: lending_types::Obligation @mut,
    borrower: account @signer,
    reserve_authority: account @signer,
    reserve_liquidity_token: account @mut,
    borrower_liquidity_token: account @mut,
    amount: u64,
    token_bytecode: account
) {
    lending_borrow::borrow(reserve, obligation, borrower, amount);
    transfer(reserve_liquidity_token, borrower_liquidity_token, reserve_authority, amount);
}

pub fn repay(
    reserve: lending_types::Reserve @mut,
    obligation: lending_types::Obligation @mut,
    payer: account @signer,
    payer_liquidity_token: account @mut,
    reserve_liquidity_token: account @mut,
    amount: u64,
    token_bytecode: account
) {
    transfer(payer_liquidity_token, reserve_liquidity_token, payer, amount);
    lending_borrow::repay(reserve, obligation, payer, amount);
}

pub fn liquidate(
    reserve: lending_types::Reserve @mut,
    obligation: lending_types::Obligation @mut,
    liquidator: account @signer,
    reserve_authority: account @signer,
    liquidator_repay_token: account @mut,
    reserve_liquidity_token: account @mut,
    reserve_collateral_token: account @mut,
    liquidator_collateral_token: account @mut,
    repay_amount: u64,
    token_bytecode: account
) {
    let seized_collateral: u64 = (repay_amount * 10500) / 10000;

    transfer(liquidator_repay_token, reserve_liquidity_token, liquidator, repay_amount);
    lending_liquidation::liquidate(reserve, obligation, liquidator, repay_amount);
    transfer(
        reserve_collateral_token,
        liquidator_collateral_token,
        reserve_authority,
        seized_collateral
    );
}
