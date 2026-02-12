account PerpMarket {
    authority: pubkey;
    base_mint: pubkey;
    quote_mint: pubkey;
    maintenance_margin_bps: u64;
    liquidation_fee_bps: u64;
    is_paused: bool;
    name: string<32>;
}

account Position {
    owner: pubkey;
    market: pubkey;
    size: u64;
    entry_price: u64;
    collateral: u64;
}
