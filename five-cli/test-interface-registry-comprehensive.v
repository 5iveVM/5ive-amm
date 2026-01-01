// Comprehensive Interface Registry Test Suite
// Tests all aspects of the interface system: preprocessing, validation, method calls, etc.

// Test 1: Multiple interface definitions
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    initialize_mint @discriminator(0) (mint: pubkey, decimals: u8, authority: pubkey, freeze_authority: pubkey);
    mint_to @discriminator(7) (mint: pubkey, to: pubkey, authority: pubkey, amount: u64);
    transfer @discriminator(3) (source: pubkey, destination: pubkey, authority: pubkey, amount: u64);
    burn @discriminator(8) (account: pubkey, mint: pubkey, authority: pubkey, amount: u64);
}

interface SystemProgram @program("11111111111111111111111111111112") {
    create_account @discriminator(0) (from: pubkey, to: pubkey, lamports: u64, space: u64, owner: pubkey);
    assign @discriminator(1) (account: pubkey, owner: pubkey);
    transfer_lamports @discriminator(2) (from: pubkey, to: pubkey, lamports: u64);
}

interface AssociatedTokenProgram @program("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL") {
    create @discriminator(0) (payer: pubkey, associated_token: pubkey, wallet: pubkey, mint: pubkey);
}

// Test 2: Type validation and coercion
test_type_validation(mint: account @mut, dest: account @mut, amount: u64, small_amount: u8) {
    // Should work: u8 coerces to u64
    SPLToken.mint_to(mint, dest, mint, small_amount);
    
    // Should work: exact type match
    SPLToken.mint_to(mint, dest, mint, amount);
}

// Test 3: Account constraint validation
test_account_constraints(
    payer: account @signer,
    mint: account @init,
    token_account: account @mut,
    amount: u64
) {
    // SystemProgram with signer constraint
    SystemProgram.create_account(payer, mint, 2039280, 82, payer);
    
    // SPL Token with mutable account constraint
    SPLToken.initialize_mint(mint, 6, payer, payer);
    SPLToken.mint_to(mint, token_account, payer, amount);
}

// Test 4: Cross-interface composition
test_cross_interface_composition(
    payer: account @signer,
    mint: account @init,
    wallet: pubkey,
    amount: u64
) -> bool {
    // Step 1: Create and initialize mint account
    SystemProgram.create_account(payer, mint, 2039280, 82, payer);
    SPLToken.initialize_mint(mint, 6, payer, payer);
    
    // Step 2: Create associated token account
    AssociatedTokenProgram.create(payer, mint, wallet, mint);
    
    // Step 3: Mint tokens
    SPLToken.mint_to(mint, mint, payer, amount);
    
    return true;
}

// Test 5: Complex parameter types (using different type combinations)
test_complex_parameters(
    source: account @mut,
    dest: account @mut, 
    authority: account @signer,
    transfer_amount: u64
) {
    // Test pubkey parameter compatibility
    SPLToken.transfer(source, dest, authority, transfer_amount);
    
    // Test with different account types
    SPLToken.burn(source, dest, authority, transfer_amount);
}

// Test 6: Interface method return type handling
test_return_types(mint: account @init, payer: account @signer) -> pubkey {
    // This should return the mint account pubkey after initialization
    SystemProgram.create_account(payer, mint, 2039280, 82, payer);
    SPLToken.initialize_mint(mint, 6, payer, payer);
    
    // Return the mint pubkey
    return mint;
}

// Test 7: Error case handling (these should be caught by type checker)
// Commented out as they should cause compilation errors:
// test_invalid_types(mint: account, amount: string) {
//     SPLToken.mint_to(mint, mint, mint, amount); // Error: string not compatible with u64
// }

// test_missing_constraints(mint: account, payer: account) {
//     SystemProgram.create_account(payer, mint, 1000000, 82, payer); // Error: payer should be @signer
// }

// Test 8: Multiple discriminators validation
test_discriminator_uniqueness(mint: account @mut, dest: account @mut, amount: u64) {
    // These use different discriminators and should all work
    SPLToken.initialize_mint(mint, 6, mint, mint);  // discriminator 0
    SPLToken.transfer(mint, dest, mint, amount);     // discriminator 3  
    SPLToken.mint_to(mint, dest, mint, amount);      // discriminator 7
    SPLToken.burn(dest, mint, mint, amount);         // discriminator 8
}