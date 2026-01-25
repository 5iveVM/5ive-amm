// INVOKE_SIGNED with PDA Authority Example
//
// This contract demonstrates using INVOKE_SIGNED to call an external program
// with authority from a Program Derived Address (PDA) that this contract controls.
//
// Use case: Burning tokens from a treasury account where the treasury is a PDA
// controlled by this contract.
//
// Interface: Calls SPL Token's burn instruction
// Authority: A PDA derived from contract state
// Serializer: Borsh (standard for SPL Token)
// Data Args: amount (u64 literal)
// Account Args: token_account, mint, authority (PDA)

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    burn @discriminator(8) (
        token_account: pubkey,
        mint: pubkey,
        authority: pubkey,
        amount: u64
    );
}

// Global state tracking treasury balance
mut treasury_balance: u64;

pub init_treasury() {
    treasury_balance = 0;
}

pub burn_from_treasury(
    token_account: account @mut,
    mint: account @mut,
    treasury_pda: account,        // PDA authority (not a signer)
    amount: u64
) {
    // Call SPL Token's burn instruction with PDA authority
    // - token_account: the token account to burn from
    // - mint: the mint being burned
    // - treasury_pda: the PDA authority (controlled by this contract)
    // - amount: number of tokens to burn (as u64 literal)
    //
    // The VM uses INVOKE_SIGNED internally with the PDA's derivation seeds.
    // This allows the contract to burn tokens without direct signer authority.
    SPLToken.burn(token_account, mint, treasury_pda, 1000);
}

// Helper to get the treasury PDA
pub get_treasury() -> pubkey {
    // In a real implementation, derive the PDA from "treasury" seed
    // For now, this is a placeholder
    treasury_pda
}
