use std::interfaces::spl_token;

pub assert_spl_state(
    mint: spl_token::Mint @serializer("raw"),
    token: spl_token::TokenAccount @serializer("raw")
) {
    // Exercise typed field decode paths without cross-type literal comparisons.
    require(mint.supply == mint.supply);
    require(mint.decimals == mint.decimals);
    require(mint.is_initialized == mint.is_initialized);
    require(token.amount == token.amount);
    require(token.state == token.state);
}
