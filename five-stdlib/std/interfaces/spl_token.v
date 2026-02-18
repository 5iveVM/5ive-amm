// std::interfaces::spl_token
// SPL Token Program interface (legacy token program)

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") @serializer("raw") {
    transfer @discriminator(3) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );

    mint_to @discriminator(7) (
        mint: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
}
