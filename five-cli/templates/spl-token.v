// SPL Token CPI template using the bundled stdlib interface.
// This template is intentionally focused on the transfer/mint CPI flows that are
// currently exposed by `std::interfaces::spl_token`.

use std::interfaces::spl_token;

// Mint tokens to a destination token account via CPI.
pub mint_tokens(
    mint: account @mut,
    destination: account @mut,
    authority: account @signer,
    amount: u64
) {
    require(amount > 0);
    spl_token::mint_to(mint, destination, authority, amount);
}

// Transfer tokens via CPI using the source account owner as authority signer.
pub transfer_tokens(
    source: account @mut,
    destination: account @mut,
    authority: account @signer,
    amount: u64
) {
    require(amount > 0);
    spl_token::transfer(source, destination, authority, amount);
}
