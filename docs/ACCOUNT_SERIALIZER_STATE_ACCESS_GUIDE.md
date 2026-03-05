# Account Serializer State Access Guide

This guide covers account-state decoding metadata for external accounts (for example SPL Token `Mint`/`TokenAccount`) and how it interacts with CPI.

## Scope

- CPI `@serializer(...)` on an `interface` controls instruction-data encoding.
- Account `@serializer(...)` on `account` definitions and parameters controls account-state field decoding.
- These are related but different surfaces.

## Supported Account Decoding Modes

- `raw`: fixed-layout offsets from byte `0` (best for SPL Token accounts).
- `borsh`: borsh layout-based decode.
- `bincode`: bincode layout-based decode.

## Syntax

Type-level default:

```five
account Mint @serializer("raw") {
    mint_authority_option: u32;
    mint_authority: pubkey;
    supply: u64;
    decimals: u8;
    is_initialized: bool;
    freeze_authority_option: u32;
    freeze_authority: pubkey;
}
```

Parameter-level override:

```five
pub read_supply(mint: Mint @serializer("raw")) -> u64 {
    return mint.supply;
}
```

## Precedence (Deterministic)

Effective account decoding mode resolves as:

1. parameter instance `@serializer(...)`
2. account type `@serializer(...)`
3. contextual default

For SPL accounts, use `raw` explicitly at type or parameter level.

## Typed Account Metadata Access

- Use `acct.ctx.key` for pubkey identity checks.
- Use `acct.ctx.lamports`, `acct.ctx.owner`, `acct.ctx.data` for runtime metadata.
- `acct.key`/`acct.lamports` on typed accounts is invalid and should be migrated to `acct.ctx.*`.

## SPL Guidance

SPL Token accounts are not Anchor accounts. Do not decode SPL state with `anchor`.

Recommended:

```five
use std::interfaces::spl_token;

pub check_balances(
    mint: spl_token::Mint @serializer("raw"),
    token: spl_token::TokenAccount @serializer("raw")
) {
    require(token.mint == mint.ctx.key);
    require(mint.decimals <= 9);
    require(token.amount >= 0);
}
```

## What To Test

Minimum test matrix:

1. default decode mode behavior (`raw` for account-state access)
2. type-level serializer behavior
3. parameter override precedence
4. mixed serializers in one function
5. on-chain assertion path (`require(...)`) reading real external accounts

## On-Chain Validation Pattern

Use a dedicated assertion instruction that only performs `require(...)` checks on decoded fields.
If decode offsets or serializer resolution are wrong, the transaction fails on-chain.
