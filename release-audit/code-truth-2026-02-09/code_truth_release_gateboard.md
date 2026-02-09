# Code-Truth Release Gateboard

Date: 2026-02-09
Commit: `f6dc18b`
Decision rule: no open P0, no unresolved P1.

## Gate Status

| Gate | Status | Severity | Evidence | Notes |
|---|---|---:|---|---|
| Binary contract alignment across layers | FAIL | P1 | `five-dsl-compiler/src/bytecode_generator/disassembler/inspector.rs:193`, `five-dsl-compiler/src/bytecode_generator/opcodes.rs:481`, `five-protocol/src/lib.rs:35`, `five-vm-mito/src/lib.rs:133` | CALL_EXTERNAL size drift in tooling and MAX_CALL_DEPTH drift between protocol/runtime. |
| Parameter/wire-format alignment | PASS | - | `cargo test -p five-protocol --features test-fixtures --test execute_payload_fixtures`, `cargo test -p five-vm-mito --test execute_payload_alignment_tests`, `cargo test -p five --test parameter_indexing_tests`, `npm run test:jest -- ...execute-wire-format...` | Canonical execute payload and typed param decoding aligned in tested paths. |
| Deploy verification correctness | PASS | - | `cargo test -p five --test deploy_verification_tests` | 15/15 tests pass including parser/verifier shared-fixture alignment and invalid-call rejection. |
| Runtime safety (critical path OOB/access class) | RISK | P2 | `cargo test -p five-vm-mito --test memory_oob_tests`, `five-vm-mito/src/handlers/functions.rs:79` | OOB tests pass, but stack-limit guard is explicitly disabled in CALL handler. |
| SDK/CLI integration contract correctness | FAIL | P1 | `five-sdk/src/modules/execute.ts:639`, `five-cli` test invocation failures (`ERR_INVALID_PACKAGE_CONFIG`) | SDK executes with `skipPreflight: true`; CLI test gate not currently runnable in this environment. |
| Frontend boundary compatibility | RISK | P1 | `five-frontend/src/lib/five-program-client.ts:9`, `five-frontend/src/components/editor/ProjectConfigModal.tsx:8` | Static integration exists; executable frontend gate not completed due local test runner instability. |

## Verification Runs (Executed)

| Command | Result |
|---|---|
| `cargo test -p five-protocol --features test-fixtures --test execute_payload_fixtures` | PASS (3/3) |
| `cargo test -p five-vm-mito --test execute_payload_alignment_tests` | PASS (5/5) |
| `cargo test -p five --test deploy_verification_tests` | PASS (15/15) |
| `cargo test -p five --test parameter_indexing_tests` | PASS (4/4) |
| `cargo test -p five-dsl-compiler --test protocol_alignment_tests` | PASS (3/3) |
| `cargo test -p five-vm-mito --test memory_oob_tests` | PASS (2/2) |
| `cargo test -p five-protocol --test opcode_consistency` | PASS (3/3) |
| `cd five-sdk && npm run test:jest -- src/__tests__/unit/bytecode-encoder-execute.test.ts src/__tests__/unit/parameter-encoder.test.ts src/__tests__/unit/execute-wire-format.test.ts` | PASS (18/18) |
| `cd five-cli && npm test ...` | FAIL (tooling/runtime env error before tests: `ERR_INVALID_PACKAGE_CONFIG`) |

## Go / No-Go

Current decision: **NO-GO**

Blocking reasons:
1. Open P1 code drift in binary contract surfaces (`CALL_EXTERNAL` tooling size + call-depth contract mismatch).
2. Open P1 verification gap in CLI integration gate (test runner not executable in current environment).

