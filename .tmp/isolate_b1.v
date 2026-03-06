interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    transfer @discriminator(3) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
}

account Pool {
    reserve_a: u64;
    reserve_b: u64;
    lp_supply: u64;
}

pub init_pool(pool: Pool @mut @init(payer=creator, space=512) @signer, creator: account @mut @signer) {
    pool.reserve_a = 0;
    pool.reserve_b = 0;
    pool.lp_supply = 0;
}

pub add_liquidity(
    pool: Pool @mut,
    user_token_a: account @mut,
    pool_token_a_vault: account @mut,
    user_authority: account @signer,
    amount_a: u64,
    amount_b: u64
) {
    require(amount_a > 0);
    require(amount_b > 0);

    let mut liquidity: u64 = 0;

    if (pool.lp_supply == 0) {
        liquidity = amount_a + amount_b;
    } else {
        require(amount_a * pool.reserve_b == amount_b * pool.reserve_a);
        liquidity = (amount_a * pool.lp_supply) / pool.reserve_a;
    }

    require(liquidity > 0);

    pool.reserve_a = pool.reserve_a + amount_a;
    pool.reserve_b = pool.reserve_b + amount_b;
    pool.lp_supply = pool.lp_supply + liquidity;
}
