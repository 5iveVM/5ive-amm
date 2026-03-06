// 5IVE AMM: Constant Product Market Maker (x * y = k)
// Informed by AGENTS.md technical specifications.

// --- Interfaces ---

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    transfer @discriminator(3) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );

    mint_to @discriminator(7) (
        mint: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );

    burn @discriminator(8) (
        source: Account,
        mint: Account,
        authority: Account,
        amount: u64
    );
}

// --- State Definitions ---

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
    authority: pubkey;
}

// --- Instructions ---

pub init_pool(
    pool: Pool @mut @init(payer=creator, space=512) @signer,
    creator: account @mut @signer,
    token_a_mint: pubkey,
    token_b_mint: pubkey,
    token_a_vault: pubkey,
    token_b_vault: pubkey,
    lp_mint: pubkey,
    fee_numerator: u64,
    fee_denominator: u64
) -> pubkey {
    require(fee_denominator > 0);
    require(fee_numerator < fee_denominator);

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
    pool.authority = creator.key;

    return pool.key;
}

pub add_liquidity(
    pool: Pool @mut,
    user_token_a: account @mut,
    user_token_b: account @mut,
    pool_token_a_vault: account @mut,
    pool_token_b_vault: account @mut,
    lp_mint: account @mut,
    user_lp_account: account @mut,
    user_authority: account @signer,
    amount_a: u64,
    amount_b: u64
) {
    require(amount_a > 0);
    require(amount_b > 0);

    let mut liquidity: u64 = 0;

    if (pool.lp_supply == 0) {
        // Initial liquidity: simple sum for first deposit
        liquidity = amount_a + amount_b;
    } else {
        // Enforce ratio: amount_a / reserve_a == amount_b / reserve_b
        // Using cross-multiplication to avoid decimals: amount_a * reserve_b == amount_b * reserve_a
        require(amount_a * pool.reserve_b == amount_b * pool.reserve_a);
        
        // liquidity = (amount_a / reserve_a) * lp_supply
        liquidity = (amount_a * pool.lp_supply) / pool.reserve_a;
    }

    require(liquidity > 0);

    // Transfer assets to pool vaults
    SPLToken.transfer(user_token_a, pool_token_a_vault, user_authority, amount_a);
    SPLToken.transfer(user_token_b, pool_token_b_vault, user_authority, amount_b);

    // Mint LP tokens to user
    // Note: In production, lp_mint authority would be a PDA
    SPLToken.mint_to(lp_mint, user_lp_account, user_authority, liquidity);

    // Update pool state
    pool.reserve_a = pool.reserve_a + amount_a;
    pool.reserve_b = pool.reserve_b + amount_b;
    pool.lp_supply = pool.lp_supply + liquidity;
}

pub swap(
    pool: Pool @mut,
    user_source: account @mut,
    user_destination: account @mut,
    pool_source_vault: account @mut,
    pool_destination_vault: account @mut,
    user_authority: account @signer,
    amount_in: u64,
    is_a_to_b: bool
) {
    require(amount_in > 0);
    require(pool.reserve_a > 0);
    require(pool.reserve_b > 0);

    let mut x: u64 = 0;
    let mut y: u64 = 0;

    if (is_a_to_b) {
        x = pool.reserve_a;
        y = pool.reserve_b;
    } else {
        x = pool.reserve_b;
        y = pool.reserve_a;
    }

    // dy = (y * dx_after_fee) / (x + dx_after_fee)
    let dx_after_fee: u64 = (amount_in * (pool.fee_denominator - pool.fee_numerator)) / pool.fee_denominator;
    let amount_out: u64 = (y * dx_after_fee) / (x + dx_after_fee);

    require(amount_out > 0);
    require(amount_out < y);

    // Execute transfers
    SPLToken.transfer(user_source, pool_source_vault, user_authority, amount_in);
    SPLToken.transfer(pool_destination_vault, user_destination, user_authority, amount_out);

    // Update reserves
    if (is_a_to_b) {
        pool.reserve_a = pool.reserve_a + amount_in;
        pool.reserve_b = pool.reserve_b - amount_out;
    } else {
        pool.reserve_b = pool.reserve_b + amount_in;
        pool.reserve_a = pool.reserve_a - amount_out;
    }
}
