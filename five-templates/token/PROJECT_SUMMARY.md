# Token Template - Summary

This template is the canonical primitive token bytecode for other Five DeFi templates.

## Role In The DeFi Stack

- Deploy this template first.
- Import its public functions from other templates using:

```v
use "{TOKEN_BYTECODE_ADDRESS}"::{transfer, mint_to, burn, approve, transfer_from};
```

- Call imported functions unqualified (`transfer(...)`) so downstream templates emit `CALL_EXTERNAL`.

## Notes

- Use a valid bytecode account address in place of `{TOKEN_BYTECODE_ADDRESS}`.
- External callers should pass the deployed token bytecode account as an explicit instruction parameter (for example `token_bytecode: account`).
