# Code-Truth Release Gateboard

Date: 2026-02-09
Commit: `f6dc18b`
Decision rule: no open P0, no unresolved P1.

## Gate Status

| Gate | Status | Severity | Evidence | Notes |
|---|---|---:|---|---|
| Binary contract alignment across layers | PASS | - | `five-dsl-compiler/src/bytecode_generator/disassembler/inspector.rs:193`, `five-dsl-compiler/tests/protocol_alignment_tests.rs:100`, `five-protocol/src/lib.rs:36`, `five-vm-mito/tests/contract_alignment_tests.rs:1` | CALL_EXTERNAL sizing drift fixed; protocol/runtime call-depth limits aligned and test-guarded. |
| Parameter/wire-format alignment | PASS | - | `cargo test -p five-protocol --features test-fixtures --test execute_payload_fixtures`, `cargo test -p five-vm-mito --test execute_payload_alignment_tests`, `cargo test -p five --test parameter_indexing_tests`, `npm run test:jest -- ...execute-wire-format...` | Canonical execute payload and typed param decoding aligned in tested paths. |
| Deploy verification correctness | PASS | - | `cargo test -p five --test deploy_verification_tests` | 15/15 tests pass including parser/verifier shared-fixture alignment and invalid-call rejection. |
| Runtime safety (critical path OOB/access class) | RISK | P2 | `cargo test -p five-vm-mito --test memory_oob_tests`, `five-vm-mito/src/handlers/functions.rs:79` | OOB tests pass, but stack-limit guard is explicitly disabled in CALL handler. |
| SDK/CLI integration contract correctness | PASS (with risk) | P2 | `five-sdk/src/modules/execute.ts:639`, `five-cli/jest.config.cjs:1`, `five-cli/src/project/__tests__/ProjectLoader.test.ts:1` | CLI gate is now executable/passing on Node 20 after adding ts-jest config and module-collision ignore; SDK still defaults to `skipPreflight: true`. |
| Frontend boundary compatibility | RISK | P2 | `five-frontend/src/lib/five-program-client.ts:9`, `five-frontend/src/components/editor/ProjectConfigModal.tsx:8` | Static integration boundary verified; executable frontend test gate still pending. |

## Verification Runs (Executed)

| Command | Result |
|---|---|
| `cargo test -p five-protocol --features test-fixtures --test execute_payload_fixtures` | PASS (3/3) |
| `cargo test -p five-vm-mito --test execute_payload_alignment_tests` | PASS (5/5) |
| `cargo test -p five --test deploy_verification_tests` | PASS (15/15) |
| `cargo test -p five --test parameter_indexing_tests` | PASS (4/4) |
| `cargo test -p five-dsl-compiler --test protocol_alignment_tests` | PASS (3/3) |
| `cargo test -p five-vm-mito --test contract_alignment_tests` | PASS (1/1) |
| `cargo test -p five-vm-mito parse_constraints_` | PASS (3/3) |
| `cargo test -p five-vm-mito --test memory_oob_tests` | PASS (2/2) |
| `cargo test -p five-protocol --test opcode_consistency` | PASS (3/3) |
| `cd five-sdk && npm run test:jest -- src/__tests__/unit/bytecode-encoder-execute.test.ts src/__tests__/unit/parameter-encoder.test.ts src/__tests__/unit/execute-wire-format.test.ts` | PASS (18/18) |
| `cd five-cli && source ~/.nvm/nvm.sh && nvm use 20 && npm test -- --ci --watchAll=false --watchman=false --runInBand --runTestsByPath src/project/__tests__/ProjectLoader.test.ts --forceExit --verbose` | PASS (4/4) |

## Go / No-Go

Current decision: **GO (conditional, with tracked P2 risks)**

Conditions:
1. No open P0/P1 remain under current gate evidence.
2. P2 risks remain tracked for post-MVP hardening (runtime stack-limit guard re-enable, frontend executable boundary gate, SDK preflight-default hardening).
