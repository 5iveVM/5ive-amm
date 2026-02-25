# AGENTS.md - 5IVE Agent Operating Contract

Read this file first.
Then use `./AGENTS_CHECKLIST.md` as the execution gate and `./AGENTS_REFERENCE.md` for syntax/debug/client details.

## 1) Mission

Deliver production-ready 5IVE contracts in one focused pass when possible, with deterministic compile/test/deploy verification.
No placeholder logic in production paths.

## 2) Source of Truth Order

When docs conflict, follow this order:
1. Local compiler/CLI/SDK source code installed in the workspace
2. Package manifests and command definitions
3. CLI help output and generated project artifacts (`five.toml`, `.five`, ABI)
4. READMEs/examples/docs

Offline-first fallback:
1. If source/docs are unavailable, continue with local CLI behavior and generated artifacts.
2. Never block waiting for external docs when compile/test feedback is available.
3. Treat compiler and runtime output as the immediate truth source.

## 3) Non-Negotiable Workflow

Always run this sequence:
1. Inspect `five.toml`.
2. Author contract and compile to `.five`.
3. Run tests/local runtime checks.
4. Deploy only with explicit target and program ID resolution.
5. Execute and verify confirmed tx metadata (`meta.err == null`).
6. Record signatures and compute units.

## 4) One-Shot Delivery Policy

1. Start with full-scope design: state, guards, init flows, core instructions, tests, and client integration.
2. Implement in compile-clean increments: state/init first, then each instruction, then tests, then client.
3. If compile fails, do not replace the design with a simplified contract.
4. Keep original scope and fix errors incrementally using compiler output and checklist gates.
5. Only reduce scope if the user explicitly requests reduced scope.

## 5) Hard Authoring Rules

1. Every account field ends with `;`.
2. Use `account @signer` for auth params (not `pubkey @signer`).
3. Use `account.ctx.key` for account pubkey access (legacy `account.key` is removed).
4. Use `account.ctx.lamports|owner|data` for runtime account metadata (legacy direct access is removed).
5. For seeded `@init`, use `account.ctx.bump` (legacy `<account>_bump` alias is removed).
6. Functions returning values must declare `-> ReturnType`.
7. Use `0` for pubkey zero-init/revocation values. Do not use `pubkey(0)`.
8. `string<N>` is production-safe.
9. `require()` supports `==`, `!=`, `<`, `<=`, `>`, `>=`, `!`, `&&`, `||`.
10. Locals are immutable by default. Use `let mut` if reassigning.
11. No mock timestamps/rates/auth bypasses in production logic.

## 6) Stdlib and Import Contract (Mandatory)

1. Use module imports, then call via module alias:
- `use std::builtins;` then `builtins::now_seconds()`
- `use std::interfaces::spl_token;` then `spl_token::transfer(...)`
2. Full-path calls are valid:
- `std::interfaces::spl_token::transfer(...)`
- `std::builtins::now_seconds()`
3. Legacy object-style interface calls are invalid:
- `SPLToken.transfer(...)` is a compile error
4. Missing import for alias calls should be fixed by adding `use <module path>;`.
## 7) Definition of Done

Work is done only when all applicable items are true:
1. `.five` artifact produced.
2. Tests passed with evidence.
3. Deployment confirmed (if in scope).
4. Execution confirmed with `meta.err == null` (if in scope).
5. Signatures and compute units recorded.
6. SDK/frontend integration snippet delivered when requested.

## 8) Required Agent Output Format

Unless the user explicitly asks for a different format, final output must include:
1. Scope implemented (what was built).
2. Files changed.
3. Build/test commands run and outcomes.
4. Security checks performed and results.
5. Deploy/execute evidence:
   - target
   - program ID
   - signature(s)
   - `meta.err` result
   - compute units
6. SDK/client usage snippet or runnable command path.
7. Remaining risks and explicit next steps.

## 9) Where to Look Next

1. `./AGENTS_CHECKLIST.md` for step-by-step gates and failure triage.
2. `./AGENTS_REFERENCE.md` for syntax, CPI rules, testing patterns, and SDK client templates.
