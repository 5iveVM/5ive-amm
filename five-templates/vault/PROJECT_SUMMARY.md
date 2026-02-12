# Vault Template - Summary

Vault template implementing share-based deposits/withdrawals and strategy controls with external token settlement.

## External Import Pattern

`src/main.v` imports token functions with:

```v
use "{TOKEN_BYTECODE_ADDRESS}"::{transfer};
```

Deposits, withdrawals, and fee transfers call `transfer(...)` directly.

## Lifecycle Entrypoints

- `init_vault`
- `init_position`
- `deposit`
- `withdraw`
- `rebalance`
- `harvest_yield`

## Required Accounts (Token Paths)

- `token_bytecode: account` on token-moving instructions
- User asset token account
- Vault asset token account
- Fee receiver token account for `harvest_yield`
- Vault authority signer for vault outflows

## Flow

1. Initialize vault and user position.
2. Mint shares on deposit based on total-assets/total-shares ratio.
3. Burn shares on withdraw and transfer proportional assets out.
4. Rebalance strategy target and harvest yield with configurable fee.
