// Five DSL Constraint Annotation Completion Demo
//
// This file demonstrates constraint annotation autocomplete for:
// - @signer - Requires account to sign the transaction
// - @mut - Marks account as mutable/writable
// - @init - Initializes a new account
// - @writable - Alias for @mut
//
// Try typing '@' after the account type in function parameters:
//
// Example 1: Transfer function with constraints
pub transfer(
    from: account @signer @mut,
    to: account @mut,
    amount: u64
) {
    // Transfer logic here
}

// Example 2: Initialize function with @init constraint
pub initialize(
    new_account: account @init(payer=payer, space=100),
    payer: account @signer @mut
) {
    // Initialize account
}

// Example 3: Multiple constraints per parameter
pub complex_operation(
    authority: account @signer,
    source: account @mut,
    destination: account @writable,
    fee_account: account @
    // ^ Type '@' here to see all constraint suggestions
) {
    // Complex operation logic
}

// Example 4: Basic account without constraints
pub read_only(
    viewer: account
) {
    // Read-only operation
}
