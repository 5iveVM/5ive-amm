# Session Auth Guide (No New Opcode)

This guide describes account-backed delegated sessions using:

- a dedicated `session-manager` DSL script (normal execute path)
- a sidecar `Session` PDA account
- compiler-lowered `@session(...)` checks using existing VM opcodes

## Attribute

Canonical form on authority/owner parameter:
`authority: account @session`

Advanced keyed overrides are still supported when needed:
`authority: account @session(delegate=..., nonce_field=..., bind_account=..., target_program=..., scope_hash=..., current_slot=...)`

Backward-compatible positional form is still parseable during migration:
`@session(delegate, authority, target_program?, scope_hash?, bind_account?, nonce?, current_slot?)`

Legacy dedicated session parameter form is deprecated:
`session: Session @session(...)`

The compiler injects hidden account slots (`__session`, `__delegate`) so users do not need to declare session/delegate parameters for standard flows. In direct-owner flow, SDKs can alias these implicit slots to the owner/authority account and bypass session-sidecar checks.

## Lowering model

Compiler emits existing checks only:

- owner key-check path OR delegated session path
- delegated path validates session `delegate`/`authority` bindings without requiring a separate delegate signature
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
