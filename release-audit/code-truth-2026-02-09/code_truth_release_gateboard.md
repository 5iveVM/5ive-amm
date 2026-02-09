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
| Runtime safety (critical path OOB/access class) | PASS | - | `five-vm-mito/src/handlers/functions.rs:64`, `cargo test -p five-vm-mito --test memory_oob_tests` | Explicit stack-limit guard restored for `CALL`/`CALL_EXTERNAL`; OOB safety regression checks pass. |
| SDK/CLI integration contract correctness | PASS | - | `five-sdk/src/modules/execute.ts:528`, `five-sdk/src/modules/execute.ts:640`, `five-sdk/src/__tests__/unit/execute-on-solana-preflight.test.ts:85`, `five-cli/src/project/__tests__/ProjectLoader.test.ts:1` | On-chain execute now defaults to preflight (`skipPreflight: false`) with explicit opt-out; CLI gate remains passing on Node 20. |
| Frontend boundary compatibility | PASS | - | `five-sdk/src/__tests__/integration/frontend-boundary.test.ts:31`, `five-frontend/src/lib/five-program-client.ts:75` | Executable boundary gate added and passing via SDK Jest harness against real frontend boundary module. |

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
| `cd five-sdk && npm run test:jest -- src/__tests__/unit/execute-on-solana-preflight.test.ts` | PASS (2/2) |
| `cd five-sdk && npm run test:jest -- src/__tests__/integration/frontend-boundary.test.ts` | PASS (2/2) |
| `cd five-cli && source ~/.nvm/nvm.sh && nvm use 20 && npm test -- --ci --watchAll=false --watchman=false --runInBand --runTestsByPath src/project/__tests__/ProjectLoader.test.ts --forceExit --verbose` | PASS (4/4) |

## Go / No-Go

Current decision: **GO**

Conditions satisfied:
1. No open P0/P1 remain under current gate evidence.
2. Requested P2 hardening gates (runtime stack-limit guard, executable frontend boundary gate, SDK preflight-default hardening) are implemented and verified.
