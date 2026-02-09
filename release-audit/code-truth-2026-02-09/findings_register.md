# Findings Register (Code-Truth)

Date: 2026-02-09
Commit: f6dc18b

## P1

1. `CALL_EXTERNAL` length drift inside compiler tooling
- Severity: P1
- Layer: Compiler tooling / analysis
- Evidence:
  - `five-dsl-compiler/src/bytecode_generator/disassembler/inspector.rs:193` uses `CALL_EXTERNAL => 8`.
  - `five-dsl-compiler/src/bytecode_generator/opcodes.rs:481` defines operand size as `4` (opcode total 5 bytes).
  - Runtime/tests assume `opcode + u8 + u16 + u8` (`five-solana/tests/call_external_tests.rs:48`).
- Risk: Bytecode scanners/inspectors can misalign offsets after `CALL_EXTERNAL`, causing false diagnostics or missed real defects.
- Proposed owner: `five-dsl-compiler`
- Unblock: Normalize `CALL_EXTERNAL` size handling in inspector + add regression test that parses mixed opcode streams with CALL_EXTERNAL.

2. External function constraint parsing is effectively stubbed
- Severity: P1
- Layer: VM runtime (`CALL_EXTERNAL`)
- Evidence:
  - `five-vm-mito/src/handlers/functions.rs:233` has `TODO: Parse fixed-width constraint metadata.`
  - `five-vm-mito/src/handlers/functions.rs:234` returns `Ok((0, [0u8; 16]))` when feature bit is set.
- Risk: Constraint metadata in external bytecode is not actually enforced at runtime even when feature signaling exists.
- Proposed owner: `five-vm-mito`
- Unblock: Implement metadata parsing path + add failing-first tests for signer/writable/init constraints sourced from metadata.

3. Call-depth contract drift between protocol and VM
- Severity: P1
- Layer: Protocol vs runtime limits
- Evidence:
  - `five-protocol/src/lib.rs:35` sets `MAX_CALL_DEPTH = 32`.
  - `five-vm-mito/src/lib.rs:133` sets `MAX_CALL_DEPTH = 8`.
- Risk: Producer/consumer assumptions diverge; scripts valid under protocol limits can fail in runtime.
- Proposed owner: protocol + vm
- Unblock: Define one canonical call-depth contract and enforce it consistently (compile-time checks + runtime error messaging).

4. CLI verification surface currently non-executable in this workspace
- Severity: P1
- Layer: CLI verification pipeline
- Evidence:
  - Running `./node_modules/.bin/jest --version` in `five-cli` fails with `ERR_INVALID_PACKAGE_CONFIG` pointing to transitive package configs.
  - `npm test` in `five-cli` fails before test execution.
- Risk: Critical-path CLI behavior cannot be verified in current environment; release confidence gap.
- Proposed owner: CLI/tooling
- Unblock: Rebuild/install dependencies for a reproducible Node version matrix (at least current local Node and CI Node), then rerun CLI gate tests.

## P2

1. Stack limit check intentionally disabled in call handler
- Severity: P2
- Layer: VM call safety
- Evidence: `five-vm-mito/src/handlers/functions.rs:79` comment: `Stack limit check is currently disabled.`
- Risk: Reduced defensive checks in hot path; potential latent stack safety regressions.
- Proposed owner: `five-vm-mito`
- Unblock: Restore explicit stack-bound guard or document hard invariant with dedicated stress tests.

2. Unconditional debug printing in compiler call generation path
- Severity: P2
- Layer: Compiler output behavior
- Evidence: Multiple `println!` in `five-dsl-compiler/src/bytecode_generator/ast_generator/functions.rs:221-255`.
- Risk: Noisy stdout and potential CI/toolchain side effects during normal compilation.
- Proposed owner: `five-dsl-compiler`
- Unblock: Gate under debug feature or remove.

3. Unreachable-pattern warnings in disassembler path
- Severity: P2
- Layer: Compiler disassembler correctness hygiene
- Evidence: `five-dsl-compiler/src/bytecode_generator/disassembler/disasm.rs:492` and `:512` trigger `unreachable_patterns` during test builds.
- Risk: Dead paths may hide intended logic and mask future regressions.
- Proposed owner: `five-dsl-compiler`
- Unblock: Remove/merge duplicate match arms and add snapshot tests for covered opcode branches.

4. Execute module forces `skipPreflight: true`
- Severity: P2
- Layer: SDK execution reliability
- Evidence: `five-sdk/src/modules/execute.ts:639` hardcodes `skipPreflight: true`.
- Risk: Preflight validation bypassed by default; earlier error detection is weakened.
- Proposed owner: `five-sdk`
- Unblock: Make preflight behavior configurable with safe default and explicit opt-out.

## P3

1. Legacy/backup artifacts increase drift risk
- Severity: P3
- Layer: SDK repo hygiene
- Evidence: `five-sdk/src/FiveSDK.ts.bak`, `five-sdk/package.old.json`, `five-sdk/tsconfig.old.json`.
- Risk: Accidental import/reference confusion and stale behavior assumptions.
- Proposed owner: `five-sdk`
- Unblock: Remove or isolate legacy artifacts from active source tree.

