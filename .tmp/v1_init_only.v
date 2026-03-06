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
