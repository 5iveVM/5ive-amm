account Mint @serializer("anchor") {
    mint_authority_option: u32;
    mint_authority: pubkey;
    supply: u64;
    decimals: u8;
    is_initialized: bool;
    freeze_authority_option: u32;
    freeze_authority: pubkey;
}

account TokenAccount @serializer("anchor") {
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

pub assert_spl_state(
    mint: Mint @serializer("raw"),
    token: TokenAccount @serializer("raw"),
    expected_mint_authority_option: u32,
    expected_mint_authority: pubkey,
    expected_decimals: u8,
    expected_is_initialized: bool,
    expected_freeze_authority_option: u32,
    expected_freeze_authority: pubkey,
    expected_amount: u64,
    expected_supply: u64,
    expected_token_mint: pubkey,
    expected_token_owner: pubkey,
    expected_delegate_option: u32,
    expected_delegate: pubkey,
    expected_state: u8,
    expected_is_native_option: u32,
    expected_is_native: u64,
    expected_delegated_amount: u64,
    expected_close_authority_option: u32,
    expected_close_authority: pubkey
) {
    require(mint.mint_authority_option == expected_mint_authority_option);
    require(mint.mint_authority == expected_mint_authority);
    require(mint.decimals == expected_decimals);
    require(mint.is_initialized == expected_is_initialized);
    require(mint.freeze_authority_option == expected_freeze_authority_option);
    require(mint.freeze_authority == expected_freeze_authority);

    require(token.amount == expected_amount);
    require(mint.supply == expected_supply);
    require(token.mint == expected_token_mint);
    require(token.owner == expected_token_owner);
    require(token.delegate_option == expected_delegate_option);
    require(token.delegate == expected_delegate);
    require(token.state == expected_state);
    require(token.is_native_option == expected_is_native_option);
    require(token.is_native == expected_is_native);
    require(token.delegated_amount == expected_delegated_amount);
    require(token.close_authority_option == expected_close_authority_option);
    require(token.close_authority == expected_close_authority);
}
