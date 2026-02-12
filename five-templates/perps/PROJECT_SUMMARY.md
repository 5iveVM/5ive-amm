# Perps Template - Summary

Perps template with position lifecycle and external token-collateral settlement.

## External Import Pattern

`src/main.v` imports token functions with:

```v
use "{TOKEN_BYTECODE_ADDRESS}"::{transfer};
```

Collateral movements use unqualified `transfer(...)` calls.

## Lifecycle Entrypoints

- `init_market`
- `init_position`
- `open_position`
- `close_position`
- `add_collateral`
- `withdraw_collateral`
- `liquidate_position`

## Required Accounts (Token Paths)

- `token_bytecode: account` on collateral-moving instructions
- Trader/liquidator collateral token accounts
- Market collateral vault account
- Market authority signer for vault payouts

## Flow

1. Initialize market and position.
2. Open and manage positions with collateral transfers.
3. Close or liquidate positions with collateral settlement.
