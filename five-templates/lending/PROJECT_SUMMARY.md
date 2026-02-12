# Lending Template - Summary

Lending template with explicit market/reserve/obligation lifecycle and external token settlement.

## External Import Pattern

`src/main.v` imports token functions with:

```v
use "{TOKEN_BYTECODE_ADDRESS}"::{transfer};
```

All token movements are unqualified `transfer(...)` calls for `CALL_EXTERNAL` emission.

## Lifecycle Entrypoints

- `init_lending_market`
- `init_reserve`
- `init_obligation`
- `deposit_collateral`
- `borrow`
- `repay`
- `liquidate`

## Required Accounts (Token Paths)

- `token_bytecode: account` on token-moving instructions
- Reserve liquidity/collateral vault token accounts
- User/liquidator token accounts
- Reserve authority signer for vault outflows

## Flow

1. Initialize market and reserve.
2. Create obligation.
3. Deposit collateral, borrow liquidity, repay debt.
4. Liquidate unhealthy obligations with seize transfer settlement.
