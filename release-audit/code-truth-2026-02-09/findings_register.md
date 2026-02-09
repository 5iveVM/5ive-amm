# Findings Register (Code-Truth)

Date: 2026-02-09
Commit: f6dc18b

## P2

1. Unconditional debug printing in compiler call generation path
- Severity: P2
- Layer: Compiler output behavior
- Evidence: Multiple `println!` in `five-dsl-compiler/src/bytecode_generator/ast_generator/functions.rs:221-255`.
- Risk: Noisy stdout and potential CI/toolchain side effects during normal compilation.
- Proposed owner: `five-dsl-compiler`
- Unblock: Gate under debug feature or remove.

2. Unreachable-pattern warnings in disassembler path
- Severity: P2
- Layer: Compiler disassembler correctness hygiene
- Evidence: `five-dsl-compiler/src/bytecode_generator/disassembler/disasm.rs:492` and `:512` trigger `unreachable_patterns` during test builds.
- Risk: Dead paths may hide intended logic and mask future regressions.
- Proposed owner: `five-dsl-compiler`
- Unblock: Remove/merge duplicate match arms and add snapshot tests for covered opcode branches.

## Resolved In This Pass

1. `CALL_EXTERNAL` length drift inside compiler tooling
- Fixed in: `five-dsl-compiler/src/bytecode_generator/disassembler/inspector.rs:193`
- Regression guard: `five-dsl-compiler/tests/protocol_alignment_tests.rs:100`

2. External function constraint metadata parser stub in VM
- Fixed in: `five-vm-mito/src/handlers/functions.rs:205`
- Regression guards: tests in `five-vm-mito/src/handlers/functions.rs` (`parse_constraints_*`)

3. Call-depth contract drift between protocol and VM
- Fixed in: `five-protocol/src/lib.rs:36` (aligned to VM)
- Regression guard: `five-vm-mito/tests/contract_alignment_tests.rs:1`

4. CLI gate non-executable due to missing Jest TS config and module collision
- Fixed in: `five-cli/jest.config.cjs:1`
- Verification: `npx jest --ci --watchAll=false --watchman=false --runInBand --runTestsByPath src/project/__tests__/ProjectLoader.test.ts --forceExit --verbose` (PASS 4/4 on Node `v20.20.0`)

5. Stack limit guard hardening restored in VM call handlers
- Fixed in: `five-vm-mito/src/handlers/functions.rs:64`
- Verification: `cargo test -p five-vm-mito --test memory_oob_tests` (PASS 2/2)

6. SDK execute preflight default hardened
- Fixed in: `five-sdk/src/modules/execute.ts:528`, `five-sdk/src/modules/execute.ts:640`, `five-sdk/src/FiveSDK.d.ts:425`
- Verification: `cd five-sdk && npm run test:jest -- src/__tests__/unit/execute-on-solana-preflight.test.ts` (PASS 2/2)

7. Frontend executable boundary gate added
- Fixed in: `five-sdk/src/__tests__/integration/frontend-boundary.test.ts:31` (executes `five-frontend/src/lib/five-program-client.ts`)
- Verification: `cd five-sdk && npm run test:jest -- src/__tests__/integration/frontend-boundary.test.ts` (PASS 2/2)

## P3

1. Legacy/backup artifacts increase drift risk
- Severity: P3
- Layer: SDK repo hygiene
- Evidence: `five-sdk/src/FiveSDK.ts.bak`, `five-sdk/package.old.json`, `five-sdk/tsconfig.old.json`.
- Risk: Accidental import/reference confusion and stale behavior assumptions.
- Proposed owner: `five-sdk`
- Unblock: Remove or isolate legacy artifacts from active source tree.
