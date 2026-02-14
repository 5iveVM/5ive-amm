# AGENTS_CHECKLIST.md - 5IVE Delivery Gates

Use this checklist during execution. Do not skip gates.

## A) Scope and Plan Gate

1. Restate requested business behavior before coding.
2. Define required instructions, account models, events, and invariants.
3. Lock units/scales up front:
- time in seconds
- USD/price scale (for example `1e6`)
- rate scale (for example `1e9`)
4. Decide one-shot target: contract + tests + minimal SDK client flow.

## B) Authoring Gate

1. Account schemas:
- every field ends with `;`
- include authority fields where auth exists
- include status/version fields for state machines when needed
2. Init/account attributes:
- canonical order: `Type @mut @init(payer=..., space=...) @signer`
- payer is `account @mut @signer`
3. Auth and guards:
- signer params are `account @signer`
- verify owner/authority with `.key`
- include amount/range/state checks
4. Local variables:
- `let` for immutable
- `let mut` for reassignment
5. No stubs:
- no fake clock, fake rate, fake auth bypass

## C) Compile Gate

1. Run:
```bash
5ive build
```
2. If compile fails, fix in this order:
- missing semicolons in account fields
- wrong attribute stack order
- immutable variable reassignment
- wrong signer type (`pubkey @signer`)
- CPI account type/mutability mismatch
3. Re-run compile until clean.
4. Capture artifact path and byte size changes.

## D) One-Shot Recovery Gate (When Errors Persist)

1. Keep full requested behavior intact.
2. Isolate the smallest failing section (single instruction or account block).
3. Patch only the failing section.
4. Recompile immediately.
5. Repeat until green.
6. Do not rewrite to a toy contract unless user asks.

## E) Test Gate

1. Run sdk/local tests:
```bash
5ive test --sdk-runner
```
2. Run focused tests:
```bash
5ive test --filter "test_*" --verbose
```
3. Add tests for every guard path and at least one happy path per public instruction.
4. Record pass/fail evidence.

## F) Deploy and Execute Gate (If In Scope)

1. Resolve program ID/target explicitly with precedence:
- `--program-id`
- `five.toml [deploy].program_id`
- CLI config target
- `FIVE_PROGRAM_ID`
2. Deploy with explicit target.
3. Execute instruction.
4. Confirm `meta.err == null`.
5. Record signature and compute units.

## G) SDK Client Gate

1. Build a client call for each public instruction needed by the user flow.
2. Ensure `.accounts(...)`, `.args(...)`, signer list, and payer are complete.
3. Add one script that runs the happy path end-to-end.
4. Include error logging that prints transaction signature and on-chain logs.

## H) Final Output Gate

1. Contract source and `.five` artifact ready.
2. Tests and results provided.
3. Deployment/execute proof provided if requested.
4. SDK client snippet or runnable script provided if requested.

## I) Quick Triage Map

1. Parser error near account block:
- check missing `;` first.
2. Attribute/init parse error:
- enforce `Type @mut @init(...) @signer`.
3. Signer/key access errors:
- switch to `account @signer` and use `.key`.
4. CPI failures:
- check interface discriminator/serializer/account types/mutability.
5. Execution failed:
- inspect tx logs, guard conditions, and account ordering.
