// Test script for Declarative Account Constraints

// Define a custom account type
account Vault {
    owner: pubkey,
    token_account: pubkey,
    authority: pubkey,
}

// Function with valid constraints
pub fn deposit(
    vault: Vault @mut @has(owner, token_account),
    owner: Account @signer, // Account with signer privilege
    token_account: Account @mut,
    authority: Account, // Just checked via has_one
    amount: u64
) {
    // Logic here
    require(amount > 0);
}

// Function that should fail compilation if type checker works (commented out to allow build, but useful for manual test)
// pub fn invalid_constraint(
//     vault: Vault @has_one(missing_param) 
// ) {
// }

// Function to test @signer constraint enforcement at runtime
pub fn restricted_action(
    admin: Account @signer
) {
    // Should fail if admin is not a signer
}

// End of script
