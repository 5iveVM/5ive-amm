account PriceOracle {
    price: u64;
    decimals: u8;
    last_update: u64;
}

pub init_oracle(
    oracle: PriceOracle @mut @init(payer=authority, space=128),
    authority: account @signer,
    price: u64,
    decimals: u8
) {
    require(price > 0);
    oracle.price = price;
    oracle.decimals = decimals;
    oracle.last_update = get_clock();
}

pub set_oracle(
    oracle: PriceOracle @mut,
    authority: account @signer,
    price: u64,
    decimals: u8,
    last_update: u64
) {
    require(price > 0);
    oracle.price = price;
    oracle.decimals = decimals;
    oracle.last_update = last_update;
}
