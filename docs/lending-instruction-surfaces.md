# Lending Instruction Surfaces (Compatibility Freeze)

## Canonical (`5ive-lending-2/src/main.v`)
- `init_market`
- `set_market_pause`
- `init_reserve`
- `init_obligation`
- `calculate_utilization`
- `calculate_borrow_rate`
- `refresh_reserve`
- `refresh_obligation`
- `refresh_obligation_with_oracle`
- `deposit_reserve_liquidity`
- `withdraw_reserve_liquidity`
- `borrow_obligation_liquidity`
- `repay_obligation_liquidity`
- `liquidate_obligation`

## Legacy (`5ive-lending/src/main.v`, before deprecation)
- `init_market`
- `init_reserve`
- `init_obligation`
- `refresh_reserve`
- `deposit`
- `withdraw`
- `borrow`
- `repay`

## Legacy (`5ive-lending-4/src/main.v`, before deprecation)
- `init_market`
- `init_reserve`
- `init_obligation`
- `deposit_collateral`
- `borrow`
- `repay`
- `liquidate`

## Notes
- Canonical ABI keeps the `5ive-lending-2` naming model to minimize breakage.
- Risk/interest behavior incorporates features from both legacy variants.
