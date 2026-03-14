use std::interfaces::spl_token;

pub assert_spl_state(
    mint: spl_token::Mint @serializer("raw"),
    token: spl_token::TokenAccount @serializer("raw"),
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
    let mut ok = mint.mint_authority_option == expected_mint_authority_option;
    require(ok);
    ok = mint.mint_authority == expected_mint_authority;
    require(ok);
    ok = mint.decimals == expected_decimals;
    require(ok);
    ok = mint.is_initialized == expected_is_initialized;
    require(ok);
    ok = mint.freeze_authority_option == expected_freeze_authority_option;
    require(ok);
    ok = mint.freeze_authority == expected_freeze_authority;
    require(ok);

    ok = token.amount == expected_amount;
    require(ok);
    ok = mint.supply == expected_supply;
    require(ok);
    ok = token.mint == expected_token_mint;
    require(ok);
    ok = token.owner == expected_token_owner;
    require(ok);
    ok = token.delegate_option == expected_delegate_option;
    require(ok);
    ok = token.delegate == expected_delegate;
    require(ok);
    ok = token.state == expected_state;
    require(ok);
    ok = token.is_native_option == expected_is_native_option;
    require(ok);
    ok = token.is_native == expected_is_native;
    require(ok);
    ok = token.delegated_amount == expected_delegated_amount;
    require(ok);
    ok = token.close_authority_option == expected_close_authority_option;
    require(ok);
    ok = token.close_authority == expected_close_authority;
    require(ok);
}
