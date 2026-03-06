use std::interfaces::spl_token;

account Pool {
    token_a_mint: pubkey;
    token_b_mint: pubkey;
    token_a_vault: pubkey;
    token_b_vault: pubkey;
    lp_mint: pubkey;
    reserve_a: u64;
    reserve_b: u64;
    lp_supply: u64;
    fee_numerator: u64;
    fee_denominator: u64;
    protocol_fee_numerator: u64;
    protocol_fees_a: u64;
    protocol_fees_b: u64;
    authority: pubkey;
    is_paused: bool;
}

pub init_pool(
    pool: Pool @mut @init(payer=creator, space=512) @signer,
    creator: account @mut @signer,
    token_a_mint: pubkey,
    token_b_mint: pubkey,
    token_a_vault: pubkey,
    token_b_vault: pubkey,
    lp_mint: pubkey,
    fee_numerator: u64,
    fee_denominator: u64,
    protocol_fee_numerator: u64
) {
    require(fee_denominator > 0);
    require(fee_numerator < fee_denominator);
    require(protocol_fee_numerator <= fee_numerator);

    pool.token_a_mint = token_a_mint;
    pool.token_b_mint = token_b_mint;
    pool.token_a_vault = token_a_vault;
    pool.token_b_vault = token_b_vault;
    pool.lp_mint = lp_mint;
    pool.reserve_a = 0;
    pool.reserve_b = 0;
    pool.lp_supply = 0;
    pool.fee_numerator = fee_numerator;
    pool.fee_denominator = fee_denominator;
    pool.protocol_fee_numerator = protocol_fee_numerator;
    pool.protocol_fees_a = 0;
    pool.protocol_fees_b = 0;
    pool.authority = creator.ctx.key;
    pool.is_paused = false;
}

pub bootstrap_liquidity(
    pool: Pool @mut @signer,
    user_token_a: account @mut,
    user_token_b: account @mut,
    pool_token_a_vault: account @mut,
    pool_token_b_vault: account @mut,
    lp_mint: account @mut,
    user_lp_account: account @mut,
    user_authority: account @signer,
    token_program: account,
    amount_a: u64,
    amount_b: u64,
    min_liquidity: u64
) {
    require(!pool.is_paused);
    require(amount_a > 0);
    require(amount_b > 0);
    require(pool.reserve_a == 0);
    require(pool.reserve_b == 0);
    require(pool.lp_supply == 0);
    require(pool_token_a_vault.ctx.key == pool.token_a_vault);
    require(pool_token_b_vault.ctx.key == pool.token_b_vault);
    require(lp_mint.ctx.key == pool.lp_mint);

    let initial_liquidity: u64 = amount_a + amount_b;
    require(initial_liquidity >= min_liquidity);

    spl_token::SPLToken::transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
    spl_token::SPLToken::transfer(user_token_b, pool_token_b_vault, user_authority, amount_b);
    spl_token::SPLToken::mint_to(lp_mint, user_lp_account, pool, initial_liquidity);

    pool.reserve_a = amount_a;
    pool.reserve_b = amount_b;
    pool.lp_supply = initial_liquidity;
}

pub add_liquidity(
    pool: Pool @mut @signer,
    user_token_a: account @mut,
    user_token_b: account @mut,
    pool_token_a_vault: account @mut,
    pool_token_b_vault: account @mut,
    lp_mint: account @mut,
    user_lp_account: account @mut,
    user_authority: account @signer,
    token_program: account,
    amount_a: u64,
    amount_b: u64,
    min_liquidity: u64
) {
    require(!pool.is_paused);
    require(amount_a > 0);
    require(amount_b > 0);
    require(pool.reserve_a > 0);
    require(pool.reserve_b > 0);
    require(pool.lp_supply > 0);
    require(pool_token_a_vault.ctx.key == pool.token_a_vault);
    require(pool_token_b_vault.ctx.key == pool.token_b_vault);
    require(lp_mint.ctx.key == pool.lp_mint);

    require(amount_a * pool.reserve_b == amount_b * pool.reserve_a);
    let proportional_liquidity: u64 = (amount_a * pool.lp_supply) / pool.reserve_a;
    require(proportional_liquidity > 0);
    require(proportional_liquidity >= min_liquidity);

    spl_token::SPLToken::transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
    spl_token::SPLToken::transfer(user_token_b, pool_token_b_vault, user_authority, amount_b);
    spl_token::SPLToken::mint_to(lp_mint, user_lp_account, pool, proportional_liquidity);

    pool.reserve_a = pool.reserve_a + amount_a;
    pool.reserve_b = pool.reserve_b + amount_b;
    pool.lp_supply = pool.lp_supply + proportional_liquidity;
}

pub swap(
    pool: Pool @mut @signer,
    user_source: account @mut,
    user_destination: account @mut,
    pool_source_vault: account @mut,
    pool_destination_vault: account @mut,
    user_authority: account @signer,
    token_program: account,
    amount_in: u64,
    min_amount_out: u64,
    is_a_to_b: bool
) {
    require(!pool.is_paused);
    require(amount_in > 0);
    require(pool.reserve_a > 0);
    require(pool.reserve_b > 0);

    let mut x: u64 = 0;
    let mut y: u64 = 0;
    if (is_a_to_b) {
        require(pool_source_vault.ctx.key == pool.token_a_vault);
        require(pool_destination_vault.ctx.key == pool.token_b_vault);
        x = pool.reserve_a;
        y = pool.reserve_b;
    } else {
        require(pool_source_vault.ctx.key == pool.token_b_vault);
        require(pool_destination_vault.ctx.key == pool.token_a_vault);
        x = pool.reserve_b;
        y = pool.reserve_a;
    }

    let protocol_fee: u64 = (amount_in * pool.protocol_fee_numerator) / pool.fee_denominator;
    let lp_fee: u64 = (amount_in * (pool.fee_numerator - pool.protocol_fee_numerator)) / pool.fee_denominator;
    let dx_after_fee: u64 = amount_in - protocol_fee - lp_fee;
    let amount_out: u64 = (y * dx_after_fee) / (x + dx_after_fee);

    require(amount_out > 0);
    require(amount_out < y);
    require(amount_out >= min_amount_out);

    spl_token::SPLToken::transfer(user_source, pool_source_vault, user_authority, amount_in);
    spl_token::SPLToken::transfer(pool_destination_vault, user_destination, pool, amount_out);

    if (is_a_to_b) {
        pool.reserve_a = pool.reserve_a + amount_in - protocol_fee;
        pool.reserve_b = pool.reserve_b - amount_out;
        pool.protocol_fees_a = pool.protocol_fees_a + protocol_fee;
    } else {
        pool.reserve_b = pool.reserve_b + amount_in - protocol_fee;
        pool.reserve_a = pool.reserve_a - amount_out;
        pool.protocol_fees_b = pool.protocol_fees_b + protocol_fee;
    }
}

pub remove_liquidity(
    pool: Pool @mut @signer,
    user_lp_account: account @mut,
    user_token_a: account @mut,
    user_token_b: account @mut,
    pool_token_a_vault: account @mut,
    pool_token_b_vault: account @mut,
    lp_mint: account @mut,
    user_authority: account @signer,
    token_program: account,
    lp_amount: u64,
    min_amount_a: u64,
    min_amount_b: u64
) {
    require(!pool.is_paused);
    require(lp_amount > 0);
    require(lp_amount <= pool.lp_supply);
    require(pool_token_a_vault.ctx.key == pool.token_a_vault);
    require(pool_token_b_vault.ctx.key == pool.token_b_vault);
    require(lp_mint.ctx.key == pool.lp_mint);

    let amount_a: u64 = (lp_amount * pool.reserve_a) / pool.lp_supply;
    let amount_b: u64 = (lp_amount * pool.reserve_b) / pool.lp_supply;
    require(amount_a > 0);
    require(amount_b > 0);
    require(amount_a >= min_amount_a);
    require(amount_b >= min_amount_b);

    spl_token::SPLToken::burn(user_lp_account, lp_mint, user_authority, lp_amount);
    spl_token::SPLToken::transfer(pool_token_a_vault, user_token_a, pool, amount_a);
    spl_token::SPLToken::transfer(pool_token_b_vault, user_token_b, pool, amount_b);

    pool.reserve_a = pool.reserve_a - amount_a;
    pool.reserve_b = pool.reserve_b - amount_b;
    pool.lp_supply = pool.lp_supply - lp_amount;
}

pub collect_protocol_fees(
    pool: Pool @mut @signer,
    pool_token_a_vault: account @mut,
    pool_token_b_vault: account @mut,
    recipient_a: account @mut,
    recipient_b: account @mut,
    authority: account @signer,
    token_program: account,
    amount_a: u64,
    amount_b: u64
) {
    require(pool.authority == authority.ctx.key);
    require(pool_token_a_vault.ctx.key == pool.token_a_vault);
    require(pool_token_b_vault.ctx.key == pool.token_b_vault);
    require(amount_a <= pool.protocol_fees_a);
    require(amount_b <= pool.protocol_fees_b);

    spl_token::SPLToken::transfer(pool_token_a_vault, recipient_a, pool, amount_a);
    spl_token::SPLToken::transfer(pool_token_b_vault, recipient_b, pool, amount_b);
    pool.reserve_a = pool.reserve_a - amount_a;
    pool.reserve_b = pool.reserve_b - amount_b;
    pool.protocol_fees_a = pool.protocol_fees_a - amount_a;
    pool.protocol_fees_b = pool.protocol_fees_b - amount_b;
}

pub update_fees(
    pool: Pool @mut,
    authority: account @signer,
    new_fee_numerator: u64,
    new_protocol_fee_numerator: u64
) {
    require(pool.authority == authority.ctx.key);
    require(new_fee_numerator < pool.fee_denominator);
    require(new_protocol_fee_numerator <= new_fee_numerator);
    pool.fee_numerator = new_fee_numerator;
    pool.protocol_fee_numerator = new_protocol_fee_numerator;
}

pub set_authority(
    pool: Pool @mut,
    authority: account @signer,
    new_authority: pubkey
) {
    require(pool.authority == authority.ctx.key);
    pool.authority = new_authority;
}

pub set_paused(
    pool: Pool @mut,
    authority: account @signer,
    paused: bool
) {
    require(pool.authority == authority.ctx.key);
    pool.is_paused = paused;
}

pub get_reserves(pool: Pool) -> u64 {
    return pool.reserve_a;
}

pub get_reserve_b(pool: Pool) -> u64 {
    return pool.reserve_b;
}

pub get_lp_supply(pool: Pool) -> u64 {
    return pool.lp_supply;
}

pub get_protocol_fees_a(pool: Pool) -> u64 {
    return pool.protocol_fees_a;
}
