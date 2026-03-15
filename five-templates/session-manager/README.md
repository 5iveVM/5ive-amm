# Session Manager Template

Session sidecar account manager for `@session(...)` delegated auth flows.

## Functions

- `create_session` initializes a deterministic session PDA.
- `revoke_session` disables delegated use.

## Session account fields

- `authority`
- `delegate`
- `target_program`
- `expires_at_slot`
- `scope_hash`
- `nonce`
- `bind_account`
- `status`
- `version`

## Usage notes

- Build and deploy this script once per environment.
- App/game scripts reference session sidecars and use `@session(...)` constraints.
- High-risk actions should remain direct-signer only.

## Examples

- `examples/session-usage.v` minimal delegated action.
- `examples/tictactoe-session.v` tic-tac-toe `play_ttt` sessionized move flow.
