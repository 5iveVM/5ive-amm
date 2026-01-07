// Counter Program - Five DSL E2E Test
// Demonstrates state persistence with account-based storage
// Uses @init constraint for CPI-based account creation

// ============================================================================
// ACCOUNT DEFINITIONS
// ============================================================================

// Counter account holds the state
account Counter {
    authority: pubkey;       // Owner of this counter
    count: u64;          // Current count value
    initialized: u64;    // Whether the counter is initialized (1 = true, 0 = false)
}

// ============================================================================
// COUNTER OPERATIONS
// ============================================================================

// Initialize a new counter account
// Uses @init constraint to create account via CPI
// Counter is a PDA derived from ["counter", owner.key]
pub initialize(
    counter: Counter @mut @init(payer=owner, space=56, seeds=["counter", owner.key]),
    owner: account @signer
) -> pubkey {
    counter.authority = owner.key;
    counter.count = 0;
    counter.initialized = 1;

    return counter.key;
}

// Increment the counter by 1
pub increment(
    counter: Counter @mut,
    owner: account @signer
) {
    // Verify ownership
    require(counter.authority == owner.key);
    require(counter.initialized);

    // Increment the counter
    counter.count = counter.count + 1;
}

// Decrement the counter by 1 (with underflow protection)
pub decrement(
    counter: Counter @mut,
    owner: account @signer
) {
    // Verify ownership
    require(counter.authority == owner.key);
    require(counter.initialized);

    // Decrement the counter (with underflow protection)
    if (counter.count > 0) {
        counter.count = counter.count - 1;
    }
}

// Add a specific amount to the counter
pub add_amount(
    counter: Counter @mut,
    owner: account @signer,
    amount: u64
) {
    // Verify ownership
    require(counter.authority == owner.key);
    require(counter.initialized);

    // Add amount to counter
    counter.count = counter.count + amount;
}

// Get the current count value
pub get_count(counter: Counter) -> u64 {
    return counter.count;
}

// Reset counter to zero
pub reset(
    counter: Counter @mut,
    owner: account @signer
) {
    // Verify ownership
    require(counter.authority == owner.key);
    require(counter.initialized);

    // Reset to zero
    counter.count = 0;
}
