# Re-Verification Checklist (RC + Final Cut)

## A. Preconditions

- [ ] Confirm target commit SHA and branch frozen for audit rerun.
- [ ] Confirm Node version matrix for JS packages (local + CI) and reinstall dependencies cleanly.
- [ ] Ensure no temporary debug flags alter runtime contracts.

## B. Fix Validation (per finding)

### For `CALL_EXTERNAL` size drift fix
- [ ] Add/adjust tests proving inspector advances correct byte length after CALL_EXTERNAL.
- [ ] Run `cargo test -p five-dsl-compiler --test protocol_alignment_tests`.
- [ ] Run compiler disassembler/inspection tests that include mixed opcodes around CALL_EXTERNAL.

### For constraint-metadata parser implementation
- [ ] Add failing-first tests for signer/writable/init metadata constraints on external calls.
- [ ] Run targeted VM tests for CALL_EXTERNAL constraints.
- [ ] Confirm no behavior regression in existing CPI/CALL_EXTERNAL suites.

### For call-depth contract alignment
- [ ] Decide canonical max depth and enforce same value/behavior across protocol/runtime/compiler checks.
- [ ] Add regression test that compiles and executes a boundary-depth call chain.

### For CLI verification pipeline recovery
- [ ] Restore runnable Jest invocation in `five-cli` (dependency/runtime compatibility).
- [ ] Execute CLI unit suites for command/project flows.

## C. Required Gate Matrix (must all pass)

1. `cargo test -p five-protocol --features test-fixtures --test execute_payload_fixtures`
2. `cargo test -p five-protocol --test opcode_consistency`
3. `cargo test -p five-vm-mito --test execute_payload_alignment_tests`
4. `cargo test -p five-vm-mito --test memory_oob_tests`
5. `cargo test -p five --test deploy_verification_tests`
6. `cargo test -p five --test parameter_indexing_tests`
7. `cargo test -p five-dsl-compiler --test protocol_alignment_tests`
8. `cd /Users/amberjackson/Documents/Development/five-org/five-mono/five-sdk && npm run test:jest -- src/__tests__/unit/bytecode-encoder-execute.test.ts src/__tests__/unit/parameter-encoder.test.ts src/__tests__/unit/execute-wire-format.test.ts`
9. `cd /Users/amberjackson/Documents/Development/five-org/five-mono/five-cli && npm test -- --runInBand src/commands/__tests__/projectFlow.test.ts src/commands/__tests__/configCommand.test.ts src/project/__tests__/ProjectLoader.test.ts`

## D. Release Acceptance

- [ ] No open P0 findings.
- [ ] No open P1 findings, or explicit waiver signed with mitigation + monitoring owner.
- [ ] Gateboard regenerated from fresh command evidence.
- [ ] RC tag only after all required gates are `PASS`.

