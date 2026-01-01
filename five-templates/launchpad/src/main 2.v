import launchpad_types;
import bonding_curve;
import token_mint;
import token_transfer;
import launchpad_core;

pub fn launch_token(
    curve: BondingCurve @mut @init,
    mint: Mint @mut @init,
    curve_token_account: TokenAccount @mut @init,
    creator: account @signer,
    name: string,
    symbol: string,
    uri: string
) {
    launchpad_core::launch_token(curve, mint, curve_token_account, creator, name, symbol, uri);
}

pub fn buy_token(
    curve: BondingCurve @mut,
    curve_token_account: TokenAccount @mut,
    buyer_token_account: TokenAccount @mut,
    buyer: account @signer,
    mint: Mint, 
    amount_sol_in: u64
) {
    launchpad_core::buy_token(curve, curve_token_account, buyer_token_account, buyer, mint, amount_sol_in);
}

pub fn sell_token(
    curve: BondingCurve @mut,
    curve_token_account: TokenAccount @mut,
    seller_token_account: TokenAccount @mut,
    seller: account @signer,
    mint: Mint,
    amount_tokens_in: u64
) {
    launchpad_core::sell_token(curve, curve_token_account, seller_token_account, seller, mint, amount_tokens_in);
}

// Helper needed for tests initiating accounts
pub fn init_token_account(
    token_account: TokenAccount @mut @init,
    owner: account @signer,
    mint: pubkey
) {
    // We can call token_transfer::init_token_account but remember to remove prefix in internal call if multi_file_mode
    // Wait, main.v calls token_transfer::init_token_account. Does token_transfer have implicit imports removed? YES.
    // Does main.v have removed prefixes? 
    // In multi_file_mode, if we import modules in main.v, we should use prefixes in main.v if main.v is the "hub".
    // Alternatively, eliminate prefixes everywhere. 
    // In my Launchpad fix, I removed prefixes in `launchpad_core.v`. 
    // In `main.v`, I should also verify.
    
    // Actually, distinct modules SHOULD use prefixes cleanly IF they are imported. 
    // If I use `multi_file_mode=true` and `import launchpad_core`, I call `launchpad_core::launch_token`.
    // BUT inside `launchpad_core.v`, it used functions from other modules by name directly (init_mint).
    
    // Let's stick to consistent pattern:
    // main.v calls implementation modules with prefix. 
    // Implementation modules call each other without prefix (as they are all "modules" inside the same crate roughly).
    
    token_transfer::init_token_account(token_account, owner, mint.key);
}
