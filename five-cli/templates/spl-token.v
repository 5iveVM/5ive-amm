// SPL Token CPI template using a local interface declaration.
// Local interfaces use dot-call syntax in the current compiler path.

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") @serializer(raw) {
    transfer @discriminator(3) (
        source: account,
        destination: account,
        authority: account,
        amount: u64
    );

    mint_to @discriminator(7) (
        mint: account,
        destination: account,
        authority: account,
        amount: u64
    );
}

// Mint tokens to a destination token account via CPI.
pub mint_tokens(
    mint: account @mut,
    destination: account @mut,
    authority: account @signer,
    amount: u64
) {
    require(amount > 0);
    SPLToken.mint_to(mint, destination, authority, amount);
}

// Transfer tokens via CPI using the source account owner as authority signer.
pub transfer_tokens(
    source: account @mut,
    destination: account @mut,
    authority: account @signer,
    amount: u64
) {
    require(amount > 0);
    SPLToken.transfer(source, destination, authority, amount);
}
