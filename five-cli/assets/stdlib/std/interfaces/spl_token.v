// std::interfaces::spl_token
// SPL Token Program interface (legacy token program)

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") @serializer("raw") {
    transfer @discriminator(3) (
        source: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );

    approve @discriminator(4) (
        source: Account,
        delegate: Account,
        authority: Account,
        amount: u64
    );

    revoke @discriminator(5) (
        source: Account,
        authority: Account
    );

    mint_to @discriminator(7) (
        mint: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );

    burn @discriminator(8) (
        source: Account,
        mint: Account,
        authority: Account,
        amount: u64
    );

    close_account @discriminator(9) (
        token_account: Account,
        destination: Account,
        authority: Account
    );

    transfer_checked @discriminator(12) (
        source: Account,
        mint: Account,
        destination: Account,
        authority: Account,
        amount: u64,
        decimals: u8
    );

    approve_checked @discriminator(13) (
        source: Account,
        mint: Account,
        delegate: Account,
        authority: Account,
        amount: u64,
        decimals: u8
    );

    mint_to_checked @discriminator(14) (
        mint: Account,
        destination: Account,
        authority: Account,
        amount: u64,
        decimals: u8
    );

    burn_checked @discriminator(15) (
        source: Account,
        mint: Account,
        authority: Account,
        amount: u64,
        decimals: u8
    );
}
