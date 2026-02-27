# AGENTS.md - 5IVE Agent Operating Contract

Read this file first.
Then use `./AGENTS_CHECKLIST.md` as the execution gate and `./AGENTS_REFERENCE.md` for syntax/debug/client details.

## 1) Mission

Deliver production-ready 5IVE contracts in one focused pass when possible, with deterministic compile/test/deploy verification.
No placeholder logic in production paths.
When porting from Anchor, preserve the original security model unless the user explicitly approves a behavioral change.

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

For Anchor ports, add this before coding:
1. Enumerate each Anchor instruction, account constraint, signer rule, and PDA seed flow.
2. Decide the exact 5IVE account/state model and interface/CPI replacements.
3. Preserve auth, range/amount guards, replay protection, and counter semantics before optimizing syntax.

## 4) One-Shot Delivery Policy

1. Start with full-scope design: state, guards, init flows, core instructions, tests, and client integration.
2. Implement in compile-clean increments: state/init first, then each instruction, then tests, then client.
3. If compile fails, do not replace the design with a simplified contract.
4. Keep original scope and fix errors incrementally using compiler output and checklist gates.
5. Only reduce scope if the user explicitly requests reduced scope.

Anchor-porting default:
1. Port all public instructions and required helpers, not just the easiest path.
2. Carry forward security-critical sysvar, signature, and entropy/auth checks.
3. If an Anchor feature does not map directly, implement the platform support or document a release blocker. Do not silently weaken behavior.

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
12. Prefer lowercase authored DSL types (`account`, `pubkey`, `string<N>`). Generated ABI or bundled stdlib surfaces may still show `Account`, but authored source should use the current lowercase form.

## 6) Stdlib and Import Contract (Mandatory)

1. Use module imports, then call via module alias:
- `use std::builtins;` then `builtins::now_seconds()`
- `use std::interfaces::spl_token;` then `spl_token::transfer(...)`
2. Full-path calls are valid:
- `std::interfaces::spl_token::transfer(...)`
- `std::builtins::now_seconds()`
3. Imported stdlib/interface modules should be called with module syntax:
- `spl_token::transfer(...)`
4. Local interfaces declared in the same source file use dot-call syntax:
- `MemoProgram.write(...)`
5. Missing import for alias calls should be fixed by adding `use <module path>;`.

## 6.1) Crypto Capability Contract (Mandatory)

1. Hash builtins use explicit output buffers:
- `sha256(input_bytes, out32)`
- `keccak256(input_bytes, out32)`
- `blake3(input_bytes, out32)`
2. Preferred wrapper names (via `std::builtins`) are:
- `hash_sha256_into(input, out)`
- `hash_keccak256_into(input, out)`
- `hash_blake3_into(input, out)`
3. Byte preimage assembly should use `bytes_concat(left, right)` for deterministic composition.
4. Ed25519 entropy/auth checks should use:
- `verify_ed25519_instruction(instruction_sysvar, expected_pubkey, message, signature) -> bool`
5. For production auth-sensitive randomness, no fallback path is allowed when Ed25519 verification fails.

Notes:
1. `bytes_concat(left, right)` returns a bytes-compatible buffer that can be fed directly into hash builtins.
2. Large fixed `[u8; N]` literals are supported through the raw-bytes lowering path; use them directly for signatures, preimages, and known vectors when the size is static.

## 6.2) Anchor Porting Contract (Mandatory When Migrating)

Map Anchor concepts to 5IVE explicitly:
1. `#[account]` struct -> `account Name { ... }`
2. signer account access -> `account @signer`
3. signer pubkey extraction -> `signer.ctx.key`
4. mutable state -> `State @mut`
5. init flows -> `State @mut @init(...)`
6. Anchor `require!()` guards -> `require(...)`
7. instruction sysvar verification patterns -> explicit `instruction_sysvar: account` parameter plus builtin validation
8. PDA seed/bump logic -> `@seed(...)`, `account.ctx.bump`, and PDA builtins as needed
9. Anchor CPI -> 5IVE interfaces with `@program(...)`, serializer/discriminator selection, and direct account params

Porting rules:
1. Keep instruction names and semantic ordering stable unless the user requests an API change.
2. Preserve counter increments, state transitions, and failure behavior exactly.
3. Do not replace verified randomness/auth paths with counters, placeholders, or simplified arithmetic.
4. If Anchor used Ed25519 instruction-sysvar proofs, the 5IVE port must also verify them before accepting entropy/authenticated input.
5. If Anchor used raw byte hashing, reproduce the byte layout exactly and prove it with a deterministic vector.
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
8. For Anchor ports: explicit mapping summary from Anchor constructs to 5IVE constructs, plus any unresolved parity gaps.

## 9) Where to Look Next

1. `./AGENTS_CHECKLIST.md` for step-by-step gates and failure triage.
2. `./AGENTS_REFERENCE.md` for syntax, CPI rules, testing patterns, and SDK client templates.
