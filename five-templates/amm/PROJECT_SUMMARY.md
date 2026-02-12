# AMM Template - Summary

AMM template with external token settlement via imported token bytecode calls.

## External Import Pattern

`src/main.v` imports token functions with:

```v
use "{TOKEN_BYTECODE_ADDRESS}"::{transfer};
```

Swaps and liquidity operations call `transfer(...)` directly (unqualified), which compiles to `CALL_EXTERNAL`.

## Required Accounts (Token Paths)

- `token_bytecode: account` (explicit runtime binding for imported token bytecode)
- User token accounts and pool token vault accounts
- Signers for user operations and pool authority payouts

## Flow

1. `initialize_pool` initializes pool metadata.
2. `add_liquidity` transfers user tokens into pool vaults then updates pool state.
3. `remove_liquidity` updates pool state then transfers owed assets out.
4. `swap_a_to_b` and `swap_b_to_a` settle in/out legs with external token transfers.
