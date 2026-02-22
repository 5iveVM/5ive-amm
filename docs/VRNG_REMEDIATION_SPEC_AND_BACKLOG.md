# VRNG Port Remediation Spec (Report-Only)

## Purpose
Define concrete remediation work without modifying `five-cli init` templates yet.

## A) Misdocumentation Report and Proposed Template Patches (Do Not Apply Yet)

Target files for future patch:
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-cli/templates/AGENTS.md`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-cli/templates/AGENTS_REFERENCE.md`

### Proposed changes
1. Add capability verification rule:
- Require agents to verify feature-support claims (CPI/crypto/sysvar/type features) against compiler tests/source before declaring unsupported.

2. Built-ins section should explicitly list:
- `sha256`, `keccak256`, `blake3`
- `get_clock_sysvar`, `get_epoch_schedule_sysvar`, `get_rent_sysvar`

3. Add “known boundaries” subsection:
- `ed25519` verification path must be explicitly validated; do not assume availability from CPI alone.
- Parser support does not imply codegen/runtime parity (arrays are key example).
- Checked arithmetic syntax uses `+?`, `-?`, `*?`.

4. Relax `pubkey(0)` statement:
- Keep `0` as canonical style.
- Note compatibility constructor behavior may exist in newer checker paths.

## B) Compiler Diagnostics Improvement Spec

## Objective
Reduce false “feature missing” conclusions by producing targeted errors where support is partial or syntax is accepted but codegen/runtime is not.

### B1) Diagnostics to add (P0/P1)
1. Array codegen gap diagnostic (P0)
- Trigger: `Value::Array` path in codegen currently fails generically.
- Required message:
  - “Array literal/type parsed but runtime array value emission is not implemented in this compiler path.”
  - Suggest fixed-size alternatives or scalar flattening.
- Candidate location:
  - `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/bytecode_generator/ast_generator/expressions.rs`

2. Ed25519 capability diagnostic (P0)
- Trigger: unresolved ed25519 builtin name or attempted instruction-sysvar verification pattern that is unsupported.
- Required message:
  - “No direct ed25519 verification builtin detected in current compiler/runtime feature set. Validate CPI/sysvar strategy or use alternative trust model.”
- Candidate locations:
  - function-call type inference + unknown builtin handling paths.

3. Parser-vs-codegen support mismatch warning class (P1)
- Trigger: constructs accepted in parser but unsupported downstream.
- Required message includes phase and fallback guidance.

4. CPI signature mismatch diagnostics (P1)
- Show interface method signature and actual call args/accounts, including mut/signer expectations.

### B2) Error format requirements
1. Include phase (`tokenize`, `parse`, `typecheck`, `codegen`).
2. Include feature tag (`array-runtime`, `ed25519`, `cpi-signature`, `sysvar`).
3. Include one actionable rewrite snippet where possible.

## C) VRNG-Specific Decision Spec

### C1) Port 1:1 components
1. State accounts and authority checks.
2. CPI interface plumbing.
3. SHA-based entropy mix.
4. Clock/sysvar entropy ingestion.

### C2) Adapted components
1. Entropy authenticity:
- If no first-class ed25519 verify path is confirmed, use a trust-shifted model (e.g., precommitted authority seeds) with explicit security tradeoff.

2. Entropy aggregation structure:
- Prefer fixed-layout entropy fields over dynamic vectors.

3. Byte parsing and conversions:
- Prefer compiler-supported serialization paths.

### C3) Hard blocker criteria
Mark production-blocked if both are true:
1. House entropy authenticity requires on-chain ed25519 verification.
2. No verified compiler/runtime path exists for that verification.

## D) Validation Scenarios (Implementation Phase)
1. CPI compile regression with emitted INVOKE opcodes.
2. `get_clock()` + sysvar builtins typecheck and compile.
3. `sha256(...)` compile and runtime path verification.
4. Negative test for ed25519 call pattern with explicit targeted diagnostic.
5. Option/Result constructor and matching tests.
6. Checked arithmetic `+?/-?/*?` overflow behavior tests.
7. Doc snippet compile tests in CI to prevent capability drift.

## E) Prioritized Backlog
1. P0: targeted diagnostics for array runtime gap and ed25519 capability ambiguity.
2. P0: publish corrected capability matrix doc for agent workflows.
3. P1: CPI signature error improvements.
4. P1: parser/codegen phase-mismatch warning framework.
5. P2: runtime feature expansion for dynamic arrays and richer binary parsing ergonomics.
