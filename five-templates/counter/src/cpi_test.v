// Minimal CPI test - PDA Initialization
// This tests the failing @init logic

account Dummy {
    val: u64;
}

// Function that initializes a PDA using the @init logic
// This effectively calls the VM's CREATE_ACCOUNT instruction with init=true
// which triggers the failing create_pda_account code
pub init_pda(
    new_pda: Dummy @init(payer=user, space=8, seeds=["dummy_v3"]),
    user: account @signer
) {
    // Just set a value to verify initialization
    new_pda.val = 123;
}
