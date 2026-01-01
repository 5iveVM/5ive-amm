account TokenMint {
    supply: u64;
    decimals: u8;
    is_initialized: bool;
    freeze_authority: pubkey;
    mint_authority: pubkey;
}

account TokenAccount {
    mint: pubkey;
    owner: pubkey;
    amount: u64;
    delegate: pubkey;
    state: u8;
    is_native: bool;
    delegated_amount: u64;
}

pub initialize_mint(mint: TokenMint @mut, mint_authority: pubkey @signer, decimals: u8, freeze_authority: pubkey) {
    require(mint.is_initialized == false); // Ensure mint is not already initialized

    mint.supply = 0;
    mint.decimals = decimals;
    mint.is_initialized = true;
    mint.freeze_authority = freeze_authority;
    mint.mint_authority = mint_authority;
}

pub initialize_account(token_account: TokenAccount @mut, mint: TokenMint, owner: pubkey @signer) {
    require(token_account.amount == 0); // Ensure token account is not already initialized with a balance
    require(mint.is_initialized == true); // Ensure mint is initialized

    token_account.mint = mint;
    token_account.owner = owner;
    token_account.amount = 0;
    token_account.delegate = owner;
    token_account.state = 1;
    token_account.is_native = false;
    token_account.delegated_amount = 0;
}

pub mint_to(mint: TokenMint @mut, destination: TokenAccount @mut, mint_authority: pubkey @signer, amount: u64) {
    require(mint.is_initialized == true); // Ensure mint is initialized
    require(amount > 0);                   // Ensure amount is positive
    require(mint.mint_authority == mint_authority); // Ensure passed mint_authority matches mint's authority
    require(destination.mint == mint.key); // Ensure destination account is for the correct mint (requires mint.key to be accessible)

    mint.supply = mint.supply + amount;
    destination.amount = destination.amount + amount;
}

pub transfer(source: TokenAccount @mut, destination: TokenAccount @mut, authority: pubkey @signer, amount: u64) {
    require(source.owner == authority);       // Ensure authority is the owner of the source account
    require(source.amount >= amount);         // Ensure source account has sufficient balance
    require(source.state == 1);               // Ensure source account is not frozen (state 1 = initialized)
    require(destination.mint == source.mint); // Ensure source and destination accounts are for the same mint
    require(amount > 0);                      // Ensure amount is positive

    source.amount = source.amount - amount;
    destination.amount = destination.amount + amount;
}

pub burn(mint: TokenMint @mut, token_account: TokenAccount @mut, owner: pubkey @signer, amount: u64) {
    require(token_account.owner == owner);       // Ensure owner is the owner of the token account
    require(token_account.amount >= amount);     // Ensure token account has sufficient balance
    require(amount > 0);                         // Ensure amount is positive

    mint.supply = mint.supply - amount;
    token_account.amount = token_account.amount - amount;
}

pub approve(source: TokenAccount @mut, delegate: pubkey, owner: pubkey @signer, amount: u64) {
    require(source.owner == owner); // Ensure owner is the owner of the source account
    require(amount >= 0);           // Ensure amount is non-negative

    source.delegate = delegate;
    source.delegated_amount = amount;
}

pub revoke(source: TokenAccount @mut, owner: pubkey @signer) {
    require(source.owner == owner); // Ensure owner is the owner of the source account

    source.delegate = owner;
    source.delegated_amount = 0;
}

pub freeze_account(mint: TokenMint @mut, token_account: TokenAccount @mut, freeze_authority: pubkey @signer) {
    require(mint.freeze_authority == freeze_authority); // Ensure freeze_authority is the correct authority for the mint
    require(token_account.state != 2);                 // Ensure the account is not already frozen

    token_account.state = 2;
}

pub thaw_account(mint: TokenMint @mut, token_account: TokenAccount @mut, freeze_authority: pubkey @signer) {
    require(mint.freeze_authority == freeze_authority); // Ensure freeze_authority is the correct authority for the mint
    require(token_account.state == 2);                 // Ensure the account is currently frozen

    token_account.state = 1;
}

pub close_account(token_account: TokenAccount @mut, destination: pubkey, owner: pubkey @signer) {
    require(token_account.owner == owner);   // Ensure owner is the owner of the token account
    require(token_account.amount == 0);      // Ensure the account has zero balance before closing

    token_account.amount = 0;
    token_account.delegate = owner;
    token_account.state = 0;
    token_account.delegated_amount = 0;
}

pub set_mint_authority(mint: TokenMint @mut, new_authority: pubkey, current_authority: pubkey @signer) {
    require(mint.mint_authority == current_authority); // Ensure current_authority is the existing mint authority

    mint.mint_authority = new_authority;
}

pub set_freeze_authority(mint: TokenMint @mut, new_authority: pubkey, current_authority: pubkey @signer) {
    require(mint.freeze_authority == current_authority); // Ensure current_authority is the existing freeze authority

    mint.freeze_authority = new_authority;
}

pub get_balance(token_account: TokenAccount) -> u64 {
    return token_account.amount;
}

pub get_supply(mint: TokenMint) -> u64 {
    return mint.supply;
}

pub is_frozen(token_account: TokenAccount) -> bool {
    return token_account.state == 2;
}

pub get_delegated_amount(token_account: TokenAccount) -> u64 {
    return token_account.delegated_amount;
}

pub transfer_checked(source: TokenAccount @mut, destination: TokenAccount @mut, delegate: pubkey @signer, amount: u64) {
    require(source.owner == delegate);           // Ensure delegate is the owner of the source account
    require(source.delegated_amount >= amount);  // Ensure the delegated amount is sufficient
    require(source.amount >= amount);            // Ensure the source account has sufficient balance
    require(source.state == 1);                  // Ensure the source account is not frozen
    require(destination.mint == source.mint);    // Ensure source and destination accounts are for the same mint
    require(amount > 0);                         // Ensure amount is positive

    source.amount = source.amount - amount;
    destination.amount = destination.amount + amount;
    source.delegated_amount = source.delegated_amount - amount;
}
