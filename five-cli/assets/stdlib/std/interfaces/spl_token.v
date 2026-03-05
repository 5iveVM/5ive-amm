// std::interfaces::spl_token
// SPL Token Program interface (legacy token program)

account Mint @serializer("raw") {
    mint_authority_option: u32;
    mint_authority: pubkey;
    supply: u64;
    decimals: u8;
    is_initialized: bool;
    freeze_authority_option: u32;
    freeze_authority: pubkey;
}

account TokenAccount @serializer("raw") {
    mint: pubkey;
    owner: pubkey;
    amount: u64;
    delegate_option: u32;
    delegate: pubkey;
    state: u8;
    is_native_option: u32;
    is_native: u64;
    delegated_amount: u64;
    close_authority_option: u32;
    close_authority: pubkey;
}

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") @serializer("raw") {
    transfer @discriminator(3) (
        source: TokenAccount,
        destination: TokenAccount,
        authority: Account @authority,
        amount: u64
    );

    approve @discriminator(4) (
        source: TokenAccount,
        delegate: Account,
        authority: Account @authority,
        amount: u64
    );

    revoke @discriminator(5) (
        source: TokenAccount,
        authority: Account @authority
    );

    mint_to @discriminator(7) (
        mint: Mint,
        destination: TokenAccount,
        authority: Account @authority,
        amount: u64
    );

    burn @discriminator(8) (
        source: TokenAccount,
        mint: Mint,
        authority: Account @authority,
        amount: u64
    );

    close_account @discriminator(9) (
        token_account: TokenAccount,
        destination: Account,
        authority: Account @authority
    );

    transfer_checked @discriminator(12) (
        source: TokenAccount,
        mint: Mint,
        destination: TokenAccount,
        authority: Account @authority,
        amount: u64,
        decimals: u8
    );

    approve_checked @discriminator(13) (
        source: TokenAccount,
        mint: Mint,
        delegate: Account,
        authority: Account @authority,
        amount: u64,
        decimals: u8
    );

    mint_to_checked @discriminator(14) (
        mint: Mint,
        destination: TokenAccount,
        authority: Account @authority,
        amount: u64,
        decimals: u8
    );

    burn_checked @discriminator(15) (
        source: TokenAccount,
        mint: Mint,
        authority: Account @authority,
        amount: u64,
        decimals: u8
    );
}
