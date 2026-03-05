# 5ive Projects Test Gap Matrix

This matrix covers the canonical projects:
- `5ive-token`
- `5ive-amm`
- `5ive-cfd`
- `5ive-esccrow`
- `5ive-lending-2`

Current state: most tests are model/math assertions and do not execute instruction-level account transitions for each public function.

## Coverage Legend
- `Instruction-tested`: test invokes the public function with account state transitions and asserts post-state.
- `Proxy-tested`: test checks equivalent math/logic only.
- `Missing`: no meaningful test for function behavior.

## 5ive-token

### Public functions in source
- `init_mint`: Missing
- `init_token_account`: Missing
- `mint_to`: Proxy-tested
- `transfer`: Proxy-tested
- `burn`: Proxy-tested
- `close_account`: Proxy-tested
- `approve`: Proxy-tested
- `revoke`: Missing
- `freeze_account`: Proxy-tested
- `thaw_account`: Proxy-tested
- `set_mint_authority`: Missing
- `set_freeze_authority`: Missing
- `set_max_supply`: Proxy-tested
- `get_supply`: Missing
- `get_balance`: Missing
- `init_counter`/`increment`/`get_value` (legacy counter in same project): toy-only tests, not instruction-state tests

### Minimum true tests to add
- `init_mint`: success, invalid authority signer, duplicate init fails.
- `init_token_account`: success, wrong owner authority, duplicate init fails.
- `mint_to`: success, exceeds cap fails, wrong mint authority fails.
- `transfer`: owner transfer success, delegate transfer success, insufficient balance fails, frozen account fails.
- `burn`: success, insufficient balance fails, wrong authority fails.
- `close_account`: zero-balance success, nonzero fails, wrong close authority fails.
- `approve`/`revoke`: set delegate + allowance, delegated transfer consumes allowance, revoke clears delegate.
- `freeze_account`/`thaw_account`: freeze blocks transfer, thaw restores transfer.
- authority updates: old authority fails after rotation, new authority succeeds.
- getters: read expected live state after mutation sequence.

## 5ive-amm

### Public functions in source
- `init_pool`: Missing
- `add_liquidity`: Proxy-tested
- `swap`: Proxy-tested
- `remove_liquidity`: Proxy-tested
- `collect_protocol_fees`: Proxy-tested (partial)
- `update_fees`: Missing
- `set_authority`: Missing
- `set_paused`: Missing
- `bootstrap_liquidity`: Missing
- `get_reserves`/`get_reserve_b`/`get_lp_supply`/`get_protocol_fees_a`: Missing

### Minimum true tests to add
- `init_pool`: success, invalid fee config fails, wrong authority signer fails.
- `bootstrap_liquidity`: first-liquidity success, min-LP guard fail path.
- `add_liquidity`: proportional add success, ratio mismatch fails, slippage/min-liquidity fail.
- `swap`: success updates reserves and fees, slippage fail, paused pool fail.
- `remove_liquidity`: success proportional withdrawal, exceeds LP balance fails.
- `collect_protocol_fees`: authority-only success, unauthorized fails.
- `update_fees`/`set_authority`/`set_paused`: admin-only checks and state transitions.
- getters: return values matching pool state after operations.

## 5ive-cfd

### Public functions in source
- `init_contract`: Missing
- `open_position`: Proxy-tested
- `accrue_funding`: Proxy-tested
- `close_position`: Proxy-tested
- `liquidate_position`: Proxy-tested
- `calc_unrealized_pnl`: Proxy-tested
- `check_liquidation`/`check_stop_loss`/`check_take_profit`: Proxy-tested
- `update_fee`/`update_leverage`/`update_margins`/`update_funding_rate`: Missing
- `pause_cfd`/`unpause_cfd`: Missing
- `set_position_cap`: Missing
- `get_positions`/`get_volume`/`get_pnl`/`get_collateral`/`get_open_interest`/`get_insurance_fund`: Missing

### Minimum true tests to add
- `init_contract`: success + invalid parameter guards.
- `open_position`: long/short success, leverage cap fail, paused fail.
- `accrue_funding`: funding updates state and impacts position accounting.
- `close_position`: profitable + loss cases, fee accounting and insurance fund effects.
- `liquidate_position`: eligible liquidation success, ineligible fails, liquidator reward/penalty checks.
- admin functions: auth checks + state mutation assertions.
- getters: reflect current contract/position state after lifecycle.

## 5ive-esccrow

### Public functions in source
- `initialize_escrow`: Proxy-tested
- `deposit_to_escrow`: Proxy-tested
- `release_funds`: Proxy-tested
- `refund_to_buyer`: Proxy-tested
- `partial_release`: Proxy-tested
- `seller_timeout_release`: Proxy-tested
- `get_balance`/`get_status`/`is_deposited`: Proxy-tested

### Minimum true tests to add
- `initialize_escrow`: success, zero amount fails, buyer==seller fails.
- `deposit_to_escrow`: exact amount success, mismatch fails, double-deposit fails.
- `release_funds`: seller-only success in valid state, invalid state/actor fails.
- `refund_to_buyer`: arbiter-only within window success, outside window fails.
- `partial_release`: valid split success, invalid split fails, status and balances update.
- `seller_timeout_release`: only after timeout success, before timeout fails.
- getters: assert values from real escrow account state transitions.

## 5ive-lending-2

### Public functions in source
- `init_market`: Missing
- `set_market_pause`: Missing
- `transfer_market_admin`: Missing
- `init_reserve`: Missing
- `set_reserve_config`: Missing
- `init_obligation`: Missing
- `init_oracle`: Missing
- `set_oracle`: Missing
- `refresh_reserve`: Missing
- `refresh_obligation`: Missing
- `refresh_obligation_with_oracle`: Missing
- `deposit_reserve_liquidity`: Proxy-tested
- `withdraw_reserve_liquidity`: Proxy-tested
- `borrow_obligation_liquidity`: Proxy-tested
- `repay_obligation_liquidity`: Proxy-tested
- `liquidate_obligation`: Proxy-tested
- `collect_protocol_fees`: Proxy-tested (partial)
- `get_utilization`: Proxy-tested
- `get_borrow_rate`: Proxy-tested

### Minimum true tests to add
- market/oracle/reserve init functions: happy path + invalid config/auth checks.
- admin mutation functions: admin-only checks and transition assertions.
- `deposit_reserve_liquidity`: balance + collateral mint changes; supply cap fail path.
- `withdraw_reserve_liquidity`: health-check gate, liquidity availability gate.
- `borrow_obligation_liquidity`: LTV gate, oracle freshness gate, reserve liquidity gate.
- `repay_obligation_liquidity`: partial/full repay effects on borrow state.
- `liquidate_obligation`: only unhealthy positions; liquidation math and state updates.
- refresh functions: accrue interest/update indexes and oracle-dependent risk numbers.
- getters: reflect live reserve/market state after operations.

## Recommended rollout
1. Implement instruction-level tests for `5ive-lending-2` first (largest risk surface).
2. Implement instruction-level tests for `5ive-token` next (used in user journeys).
3. Implement AMM and CFD lifecycle suites.
4. Convert escrow proxy tests to full state-transition tests.
5. Keep proxy/math tests as fast unit checks, but mark them as secondary.
