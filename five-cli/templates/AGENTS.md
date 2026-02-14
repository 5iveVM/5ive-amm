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
3. READMEs/examples/docs

## 3) Non-Negotiable Workflow

Always run this sequence:
1. Inspect `five.toml`.
2. Author contract and compile to `.five`.
3. Run tests/local runtime checks.
4. Deploy only with explicit target and program ID resolution.
5. Execute and verify confirmed tx metadata (`meta.err == null`).
6. Record signatures and compute units.

## 4) One-Shot Delivery Policy

1. Target one complete contract pass first: state, guards, init flows, core instructions, tests, and basic client integration.
2. If first compile fails, do not replace the design with a simplified contract.
3. Keep the original scope and fix errors incrementally using compiler output and checklist gates.
4. Only reduce scope if the user explicitly requests reduced scope.

## 5) Hard Authoring Rules

1. Every account field ends with `;`.
2. Use `account @signer` for auth params (not `pubkey @signer`).
3. Use `.key` on `account` values for comparisons/assignments.
4. Functions returning values must declare `-> ReturnType`.
5. `0` and `pubkey(0)` are valid pubkey zero-init/revocation values.
6. `string<N>` is production-safe.
7. `require()` supports `==`, `!=`, `<`, `<=`, `>`, `>=`, `!`, `&&`, `||`.
8. Locals are immutable by default. Use `let mut` if reassigning.
9. No mock timestamps/rates/auth bypasses in production logic.

## 6) Definition of Done

Work is done only when all applicable items are true:
1. `.five` artifact produced.
2. Tests passed with evidence.
3. Deployment confirmed (if in scope).
4. Execution confirmed with `meta.err == null` (if in scope).
5. Signatures and compute units recorded.
6. SDK/frontend integration snippet delivered when requested.

## 7) Where to Look Next

1. `./AGENTS_CHECKLIST.md` for step-by-step gates and failure triage.
2. `./AGENTS_REFERENCE.md` for syntax, CPI rules, testing patterns, and SDK client templates.
