# Session Auth Guide (No New Opcode)

This guide describes account-backed delegated sessions using:

- a dedicated `session-manager` DSL script (normal execute path)
- a sidecar `Session` PDA account
- compiler-lowered `@session(...)` checks using existing VM opcodes

## Attribute

Preferred (keyed) form:
`@session(delegate=..., authority=..., target_program=..., scope_hash=..., bind_account=..., nonce_field=..., current_slot=...)`

Backward-compatible positional form:
`@session(delegate, authority, target_program?, scope_hash?, bind_account?, nonce?, current_slot?)`

Applied on the session account parameter.

## Lowering model

Compiler emits existing checks only:

- `CHECK_SIGNER` for `delegate`
- optional active check (`status == 1`) via `REQUIRE_BATCH_FIELD_EQ_IMM`
- `REQUIRE_OWNER` for session delegate/authority binding
- field equality checks (`target_program`, `scope_hash`, `bind_account`, `nonce`)
- optional expiry gate (`expires_at_slot >= current_slot`) via `REQUIRE_BATCH_FIELD_GTE_PARAM`

No new opcode is introduced.

## Recommended flow

1. User signs once to create session sidecar via `session-manager.create_session`.
2. App sends repeated delegated gameplay txs with `delegate + session`.
3. Program accepts direct authority signer OR valid session path.
4. Revoke with `session-manager.revoke_session`.
