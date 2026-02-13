// 5IVE DSL - AMM Swap between 5IVE Native Token and SPL Token
// Modular Refactored Version

interface SPLToken {
    initialize_mint @discriminator(0) (mint: pubkey, decimals: u8, authority: pubkey, freeze_authority: pubkey);
    mint_to @discriminator(7) (mint: pubkey, to: pubkey, authority: pubkey, amount: u64);
    transfer @discriminator(3) (source: pubkey, destination: pubkey, authority: pubkey, amount: u64);
    burn @discriminator(8) (mint: pubkey, target_account: pubkey, authority: pubkey, amount: u64);
}

account AMMPool {
    token_a_reserve: u64;
    token_b_reserve: u64;
    total_lp_shares: u64;
    fee_bps: u64;
    token_a_mint: pubkey;
    token_b_mint: pubkey;
    pool_authority: pubkey;
    lp_token_mint: pubkey;
    last_k: u64;
}

account LPAccount {
    owner: pubkey;
    pool: pubkey;
    lp_shares: u64;
}

import math;

pub init_pool(pool: AMMPool @mut, authority: account @signer, token_a_mint: pubkey, token_b_mint: pubkey, lp_token_mint: pubkey, fee_bps: u64) -> pubkey {
    require(fee_bps < 10000);
    require(token_a_mint != token_b_mint);
    pool.token_a_reserve = 0;
    pool.token_b_reserve = 0;
    pool.total_lp_shares = 0;
    pool.fee_bps = fee_bps;
    pool.token_a_mint = token_a_mint;
    pool.token_b_mint = token_b_mint;
    pool.pool_authority = authority.key;
    pool.lp_token_mint = lp_token_mint;
    pool.last_k = 0;
    return pool.key;
}
