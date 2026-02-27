// std::interfaces::spl_token
// SPL Token Program interface (legacy token program)

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") @serializer(raw) {
    transfer @discriminator(3) (
        source: account,
        destination: account,
        authority: account,
        amount: u64
    );

    approve @discriminator(4) (
        source: account,
        delegate: account,
        authority: account,
        amount: u64
    );

    revoke @discriminator(5) (
        source: account,
        authority: account
    );

    mint_to @discriminator(7) (
        mint: account,
        destination: account,
        authority: account,
        amount: u64
    );

    burn @discriminator(8) (
        source: account,
        mint: account,
        authority: account,
        amount: u64
    );

    close_account @discriminator(9) (
        token_account: account,
        destination: account,
        authority: account
    );

    transfer_checked @discriminator(12) (
        source: account,
        mint: account,
        destination: account,
        authority: account,
        amount: u64,
        decimals: u8
    );

    approve_checked @discriminator(13) (
        source: account,
        mint: account,
        delegate: account,
        authority: account,
        amount: u64,
        decimals: u8
    );

    mint_to_checked @discriminator(14) (
        mint: account,
        destination: account,
        authority: account,
        amount: u64,
        decimals: u8
    );

    burn_checked @discriminator(15) (
        source: account,
        mint: account,
        authority: account,
        amount: u64,
        decimals: u8
    );
}
