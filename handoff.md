# Handoff: MVP Lockdown + Protocol/VM Drift Elimination

Date: 2026-02-10
Workspace: `/Users/amberjackson/Documents/Development/five-org/five-mono`

## Goal Context
The user asked to:
1. Keep disassembler/compiler/runtime aligned to `five-protocol` as source of truth.
2. Remove VLE terminology/paths/config to avoid confusion and drift.
3. Continue a broad drift search and identify what remains to reach MVP lock.

## What Was Completed In This Session

### 1) VLE removal and naming normalization (broad pass)
- Rewired SDK encoder import path from VLE naming to neutral varint naming.
- Removed VLE fallback decode references from active SDK logic.
- Updated wasm JS wrapper surfaces and downstream call sites to neutral names.
- Removed `enable_vle = true` from all template `five.toml` files.
- Updated docs/comments/scripts to remove VLE terminology where still present.

Key files touched in this pass include:
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/src/FiveSDK.js`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/src/wasm/index.js`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/src/wasm/index.d.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/lib/varint-encoder.js` (new)
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/src/assets/wasm/five_vm_wasm_bg.js`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/assets/vm/five_vm_wasm_bg.js`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-frontend/src/app/ide/page.tsx`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/src/__tests__/setup.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-vm-mito/tests/cpi_unit_tests.rs`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/tests/modular_generator_tests.rs`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-templates/*/five.toml` (all template projects)
- Root/docs/script files where wording was stale (`CLAUDE.md`, `CHANGES_MADE.md`, etc.)

### 2) Drift/health validation executed

Rust checks/tests run and passing:
- `cargo check --workspace` -> pass
- `cargo test -p five-protocol --test opcode_tests` -> pass
- `cargo test -p five-dsl-compiler --test protocol_alignment_tests` -> pass
- `cargo test -p five-vm-mito --test cpi_unit_tests` -> pass

SDK checks/tests:
- `node five-sdk/node_modules/typescript/bin/tsc -p five-sdk/tsconfig.json --noEmit` -> pass
- `npm -C five-sdk run test:jest -- --runInBand` -> pass (21 suites)

CLI checks/tests:
- `node five-cli/node_modules/typescript/bin/tsc -p five-cli/tsconfig.json --noEmit` -> pass
- `npm -C five-cli run test -- --runInBand` -> fails (details below)

Frontend checks/build:
- `node five-frontend/node_modules/typescript/bin/tsc -p five-frontend/tsconfig.json --noEmit` -> fails (details below)
- `npm -C five-frontend run build` -> fails (missing dependency module)

VLE grep verification:
- Repo-wide grep for `VLE`, `vle`, `enable_vle`, `encode_execute_vle`, `decode_vle_instruction`, `vle-encoder.js` was driven to no matches in active source/config paths covered.

## Current MVP Blockers (Actionable)

### P0 blocker 1: `five-frontend` build is broken
Command:
- `npm -C five-frontend run build`
Failure:
- `Cannot find module 'baseline-browser-mapping'` from Next/browserslist chain.

Likely fix:
- Repair frontend dependency graph (`npm install` in `five-frontend`, possibly remove lock/node_modules mismatch and reinstall).

Additional frontend TS issues (from noEmit):
- Missing webpack types import in `next.config.ts`.
- Test matcher typing not configured for `@testing-library/jest-dom`.
- Monaco typing mismatches in several files.
- Missing module `./five-lsp-wasm` in `src/lib/lsp-client.ts`.

Representative files:
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-frontend/next.config.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-frontend/src/lib/lsp-client.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-frontend/src/lib/monaco-code-actions.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-frontend/src/lib/monaco-document-symbols.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-frontend/src/lib/monaco-semantic-tokens.ts`

### P0 blocker 2: `five-cli` test suite failing
Command:
- `npm -C five-cli run test -- --runInBand`
Failures observed:
1. Module resolution in test mock:
- `Cannot find module 'five-sdk'` in `projectFlow.test.ts`.

2. ESM/Jest runtime issue:
- `Identifier 'require' has already been declared` in `src/cli.ts` due to `createRequire` usage under current test transform.

3. Chalk mock mismatch:
- `colors.rose is not a function` and related in `cli-ui.test.ts`; mock only provides `chalk.hex` while code uses named chalk methods.

Representative files:
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-cli/src/commands/__tests__/projectFlow.test.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-cli/src/cli.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-cli/src/utils/__tests__/cli-ui.test.ts`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-cli/jest.config.cjs`

### P0 blocker 3: on-chain E2E not validated in this session
- Local validator endpoint `http://127.0.0.1:8899` was unreachable when checked.
- Cannot certify deploy/execute path until localnet is running and templates are executed.

Commands that failed due to no validator:
- `solana cluster-version -u http://127.0.0.1:8899`
- `solana program show --programs -u http://127.0.0.1:8899`

## Important Observations For Next Agent

1. Rust core is in relatively good shape.
- Workspace compiles.
- Protocol alignment tests are green.
- VM CPI unit tests are green.

2. Remaining risk is JS integration/tooling, not opcode protocol core.
- SDK is passing.
- CLI + frontend are the largest remaining release friction points.

3. There are stale artifact files that may cause confusion if left:
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/src/FiveSDK.ts.bak`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk/src/examples/client-integration.ts.bak`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-vm-mito/tests/function_calls.rs.disabled`

These were discovered and not removed in this pass.

## Suggested Next Execution Plan (in order)

1. Fix `five-cli` tests to green.
- Adjust Jest mock strategy for `five-sdk` module in `projectFlow.test.ts`.
- Resolve ESM test parsing for `createRequire` path in `cli.ts` and/or transform config.
- Fix `cli-ui.test.ts` chalk mock to include methods used by `cli-ui.ts`.
- Re-run: `npm -C five-cli run test -- --runInBand`.

2. Fix frontend dependency/build gate.
- Repair dependency install state for `five-frontend`.
- Re-run: `npm -C five-frontend run build`.
- Then address TS noEmit errors and ensure `node .../tsc -p five-frontend/tsconfig.json --noEmit` is green (or intentionally scoped to exclude tests if policy says so).

3. Bring up local validator and run end-to-end smoke.
- Start validator.
- Re-check cluster availability.
- Run template E2E (token/counter minimum) and capture pass/fail with logs.

4. Add a release gate script.
- Include Rust checks/tests + SDK tests + CLI tests + frontend build + localnet smoke.
- Ensure this script is the MVP go/no-go command.

## Commands Reference (used this session)

Core verification:
- `cargo check --workspace`
- `cargo test -p five-protocol --test opcode_tests`
- `cargo test -p five-dsl-compiler --test protocol_alignment_tests`
- `cargo test -p five-vm-mito --test cpi_unit_tests`

Type checks:
- `node five-sdk/node_modules/typescript/bin/tsc -p five-sdk/tsconfig.json --noEmit`
- `node five-cli/node_modules/typescript/bin/tsc -p five-cli/tsconfig.json --noEmit`
- `node five-frontend/node_modules/typescript/bin/tsc -p five-frontend/tsconfig.json --noEmit`

JS tests/build:
- `npm -C five-sdk run test:jest -- --runInBand`
- `npm -C five-cli run test -- --runInBand`
- `npm -C five-frontend run build`

Validator checks:
- `solana cluster-version -u http://127.0.0.1:8899`
- `solana program show --programs -u http://127.0.0.1:8899`

## Current Git State Note
There are many modified files already in the working tree from prior and current sessions. The next agent should:
- avoid reverting unrelated local changes,
- make focused commits by concern (e.g. CLI test fixes, frontend build fixes, E2E gating script).

