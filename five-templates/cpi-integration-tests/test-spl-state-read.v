use std::interfaces::spl_token;

pub assert_spl_state(
    mint: spl_token::Mint @serializer("raw"),
    token: spl_token::TokenAccount @serializer("raw"),
    expected_supply: u64,
    expected_amount: u64
) {
    require(mint.supply == expected_supply);
    require(token.amount == expected_amount);
}
